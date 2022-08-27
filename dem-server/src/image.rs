use crate::*;

#[derive(Debug)]
pub struct ImageStore {
    pub base_path: std::path::PathBuf,
    pub temp_image_dir: std::path::PathBuf,
    pub cache: tokio::sync::RwLock<
        lru::LruCache<u64, std::collections::HashMap<uuid::Uuid, ImageData, fxhash::FxBuildHasher>>,
    >,
}

impl ImageStore {
    pub fn from_figment(f: &rocket::figment::Figment) -> Self {
        ImageStore {
            base_path: f
                .extract_inner("dem.image_store")
                .expect("You need to specify the image_store property"),
            temp_image_dir: f
                .extract_inner("dem.temp_image_dir")
                .expect("You need to specify the temp_image_dir property"),
            cache: tokio::sync::RwLock::new(lru::LruCache::new(1024)),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ImageData {
    name: String,
    #[serde(rename = "type")]
    image_type: ImageType,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, JsonSchema)]
pub enum ImageType {
    Gif,
    Png,
}

impl ImageType {
    fn to_content_type(&self) -> rocket::http::ContentType {
        match self {
            Self::Gif => rocket::http::ContentType::GIF,
            Self::Png => rocket::http::ContentType::PNG,
        }
    }
}

#[openapi]
#[post("/upload/<guildid>/store/emoji?<name>", data = "<file>")]
pub async fn upload_emoji_to_store(
    mut file: rocket::fs::TempFile<'_>,
    content_type: &rocket::http::ContentType,
    logic: &rocket::State<crate::discord::Logic>,
    store: &rocket::State<ImageStore>,
    guildid: u64,
    user: crate::auth::User,
    name: &str,
) -> Rsp<String> {
    use tokio::io::AsyncWriteExt;
    if name.len() > 32 || name.len() < 2 || name.chars().any(|c| !c.is_ascii_alphanumeric()) {
        return Rsp::err(
            dem_types::error::Error::InvalidRequest,
            Some("Invalid name".to_string()),
        );
    }
    let uuid = uuid::Uuid::new_v4();
    let file_path = format!("/tmp/dem/{uuid}");
    file.persist_to(&file_path).await.unwrap();

    let image_type = if content_type.is_png() {
        ImageType::Png
    } else if content_type.is_gif() {
        ImageType::Gif
    } else {
        return Rsp::err(
            dem_types::error::Error::InvalidRequest,
            Some("Invalid body".to_string()),
        );
    };

    let rating = match logic.get_image_rating(&file_path).await.map_err(|e| {
        error!("Error with Google SafeSearch :{e}");
        crate::dem_types::error::Error::Internal
    }) {
        Ok(v) => v,
        Err(e) => return Rsp::err(e, Some("Error with Google SafeSearch API".to_string())),
    };
    dbg!(rating);
    if rating < ImageRating::MIN {
        return Rsp::err(
            dem_types::error::Error::InvalidRequest,
            Some("Image rating not valid".to_string()),
        );
    }

    let mut lock = logic.user_cache.write().await;

    let user_permission = lock
        .get(user.token.as_str())
        .and_then(|u| {
            u.guilds
                .get(&guildid)
                .map(|&p| p & ((1 << 30) | (1 << 3)) < 1)
        })
        .unwrap_or(false);
    if !user_permission {
        return Rsp::err(
            dem_types::error::Error::Unauthorized,
            Some("You are not in the guild or don't have permission to do so".to_string()),
        );
    }

    let file_name = format!("{}", uuid.hyphenated());
    let metadata_filename = format!("{file_name}.json",);
    let mut file_path = store.base_path.clone();
    file_path.push(guildid.to_string());
    file_path.push(&file_name);

    if let Err(e) = file.persist_to(&file_path).await {
        error!("Error when persisting image to disk: {e}");
        return Rsp::err(
            dem_types::error::Error::Internal,
            Some("Error when trying to store file".to_string()),
        );
    }

    file_path.pop();
    file_path.push(metadata_filename);

    let metadata = ImageData {
        image_type,
        name: file_name,
    };

    let mut metadata_file = tokio::fs::File::create(file_path).await.unwrap();

    let metadata_bytes = serde_json::to_vec(&metadata).unwrap();

    let amount = metadata_file.write(&metadata_bytes).await.unwrap();
    if amount != metadata_bytes.len() {
        error!("Invalid metadata write");
    }
    let mut cache = store.cache.write().await;

    if let Some(m) = cache.get_mut(&guildid) {
        m.insert(uuid, metadata);
    } else {
        cache.push(guildid, {
            let mut hm: std::collections::HashMap<uuid::Uuid, ImageData, fxhash::FxBuildHasher> =
                Default::default();
            hm.insert(uuid, metadata);
            hm
        });
    }

    Rsp::ok(uuid.hyphenated().to_string())
}

#[get("/<guildid>/<uuid>")]
pub async fn image_serve(
    uuid: uuid::Uuid,
    guildid: u64,
    store: &rocket::State<ImageStore>,
) -> Result<(rocket::http::ContentType, tokio::fs::File), rocket::http::Status> {
    let file = {
        let mut p = store.base_path.clone();
        p.push(guildid.to_string());
        p.push(uuid.hyphenated().to_string());
        tokio::fs::File::open(p).await.map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => rocket::http::Status::NotFound,
            _ => {
                error!("Error when delivering image: {e}");
                rocket::http::Status::InternalServerError
            }
        })?
    };
    let metadata_file = {
        let mut p = store.base_path.clone();
        p.push(guildid.to_string());
        p.push(format!("{}.json", uuid.hyphenated()));
        tokio::fs::File::open(p)
            .await
            .map_err(|e| {
                error!("Error when finding image metadata: {e}");
                rocket::http::Status::InternalServerError
            })?
            .into_std()
            .await
    };

    let metadata: ImageData =
        tokio::task::spawn_blocking(|| serde_json::from_reader(metadata_file))
            .await
            .map_err(|e| {
                error!("Error when reading image's metadata: {e}");
                rocket::http::Status::InternalServerError
            })?
            .map_err(|e| {
                error!("Error when reading image's metadata: {e}");
                rocket::http::Status::InternalServerError
            })?;

    let content_type = metadata.image_type.to_content_type();
    let mut cache = store.cache.write().await;
    if let Some(m) = cache.get_mut(&guildid) {
        m.insert(uuid, metadata);
    } else {
        cache.push(guildid, {
            let mut hm: std::collections::HashMap<uuid::Uuid, ImageData, fxhash::FxBuildHasher> =
                Default::default();
            hm.insert(uuid, metadata);
            hm
        });
    }

    Ok((content_type, file))
}

#[derive(
    Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct ImageRating {
    pub racy: Rating,
    pub adult: Rating,
    pub spoof: Rating,
    pub violance: Rating,
    pub medical: Rating,
}

impl ImageRating {
    const MIN: ImageRating = ImageRating {
        racy: Rating::Unknown,
        adult: Rating::Possible,
        spoof: Rating::Unknown,
        violance: Rating::Possible,
        medical: Rating::Possible,
    };
}

#[derive(
    Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum Rating {
    #[serde(rename = "UNKNOWN")]
    Unknown = 5,
    #[serde(rename = "VERY_UNLIKELY")]
    VeryUnlikely = 4,
    #[serde(rename = "UNLIKELY")]
    Unlikely = 3,
    #[serde(rename = "POSSIBLE")]
    Possible = 2,
    #[serde(rename = "LIKELY")]
    Likely = 1,
    #[serde(rename = "VERY_LIKELY")]
    VeryLikely = 0,
}

#[derive(Clone, Debug, JsonSchema, serde::Deserialize, serde::Serialize)]
pub struct ImageDataApi {
    uuid: String,
    name: String,
    image_type: ImageType,
}

#[openapi]
#[get("/uploaded/<guildid>/emojis")]
pub async fn image_list(
    store: &rocket::State<ImageStore>,
    logic: &rocket::State<crate::discord::Logic>,
    user: crate::auth::User,
    guildid: u64,
) -> Rsp<Vec<ImageDataApi>> {
    if logic
        .user_cache
        .write()
        .await
        .get(&user.token)
        .map(|u| u.guilds.get(&guildid).is_some())
        .unwrap_or_default()
    {
        if let Some(u) = store.cache.write().await.get(&guildid) {
            debug!("Image store cache hit");
            return Rsp::ok(
                u.iter()
                    .map(|(uuid, data)| ImageDataApi {
                        uuid: uuid.to_string(),
                        name: data.name.clone(),
                        image_type: data.image_type.clone(),
                    })
                    .collect::<Vec<_>>(),
            );
        }
        use futures::StreamExt;
        let files = tokio::fs::read_dir({
            let mut p = store.base_path.clone();
            p.push(guildid.to_string());
            p
        })
        .await;
        if let Err(e) = files {
            error!("Reading image store: {e}");
            return Rsp::err(dem_types::error::Error::Internal, None);
        }
        let files = tokio_stream::wrappers::ReadDirStream::new(files.unwrap());
        let emojis = files
            .filter_map(|f| async { f.ok() })
            .filter_map(|f| async { f.file_name().to_str().map(|n| (n.to_string(), f)) })
            .filter_map(|(n, f)| async move {
                n.split('.')
                    .nth_back(0)
                    .and_then(|ext| if ext == "json" { Some(()) } else { None })
                    .map(|_| (n.to_string(), f))
            })
            .filter_map(|(n, f)| async move {
                Some((
                    n,
                    tokio::task::spawn_blocking(move || {
                        let file = std::fs::File::open(f.path()).unwrap();
                        serde_json::from_reader::<_, ImageData>(file).unwrap()
                    })
                    .await
                    .unwrap(),
                ))
            })
            .map(|(n, img)| ImageDataApi {
                name: img.name,
                image_type: img.image_type,
                uuid: n.split('.').next().unwrap().to_string(),
            })
            .collect::<Vec<_>>()
            .await;
        let emoji_for_cache = emojis.clone().into_iter().map(|d| {
            (
                uuid::Uuid::parse_str(&d.uuid).unwrap(),
                ImageData {
                    image_type: d.image_type,
                    name: d.name,
                },
            )
        });
        store.cache.write().await.push(
            guildid,
            emoji_for_cache
                .collect::<std::collections::HashMap<uuid::Uuid, ImageData, fxhash::FxBuildHasher>>(
                ),
        );

        Rsp::ok(emojis)
    } else {
        Rsp::err(dem_types::error::Error::Unauthorized, None)
    }
}
