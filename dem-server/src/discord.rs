use dem_types::discord as types;
use rocket::figment::Figment;
use serde_json::json;

const DISCORD_API: &str = "https://discord.com/api/v10";
const DISCORD_WS: &str = "wss://gateway.discord.gg/?v=10&encoding=json";
const INTENTS: u64 = (1 << 0) | (1 << 3);

static mut BOT_AUTH_HEADER: &str = "";

pub fn get_token() -> &'static str {
    unsafe { BOT_AUTH_HEADER }
}

#[derive(Debug)]
pub struct Logic {
    discord_token: String,
    pub guilds: &'static dashmap::DashMap<u64, types::PartialGuild>,
    //user_cache: lru::LruCache<String, LoggedUser>,
    client: reqwest::Client,
}

impl Logic {
    async fn handle_gateway(
        token: String,
        guilds: &'static dashmap::DashMap<u64, types::PartialGuild>,
    ) {
        use futures_util::{sink::SinkExt, stream::StreamExt};
        use rand::{Rng, SeedableRng};

        let (mut client_writer, mut client_reader) =
            match tokio_tungstenite::connect_async(DISCORD_WS).await {
                Err(e) => {
                    error!("Error when connecting to discord gateway: {e}");
                    return;
                }
                Ok((ws, _)) => ws.split(),
            };

        static ACTION_ID: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(-1);
        let mut session_id: Option<String> = None;
        let client_writer = std::sync::Arc::new(tokio::sync::Mutex::new(client_writer));

        let notify = std::sync::Arc::new(tokio::sync::Notify::new());
        let notify2 = notify.clone();

        while let Some(message) = client_reader.next().await {
            match message {
                Err(e) => {
                    error!("Error with the gateway connection: {e}");
                    if let Some(session_id) = &session_id {
                        let mut client_writer_lock = client_writer.lock().await;
                        let (client_writer_new, client_reader_new) =
                            match tokio_tungstenite::connect_async(DISCORD_WS).await {
                                Err(e) => {
                                    error!("Error when connecting to discord gateway: {e}");
                                    return;
                                }
                                Ok((ws, _)) => ws.split(),
                            };
                        client_reader = client_reader_new;
                        *client_writer_lock = client_writer_new;
                        match client_writer_lock
                            .send(tokio_tungstenite::tungstenite::Message::Binary(
                                serde_json::to_vec(&json!({
                                    "op": 6,
                                        "d": {
                                        "token": token.as_str(),
                                        "session_id": session_id.as_str(),
                                        "seq": ACTION_ID.load(std::sync::atomic::Ordering::Relaxed)
                                    }
                                }))
                                .unwrap(),
                            ))
                            .await
                        {
                            Ok(_) => info!("[WS] Sent OPCODE 6"),
                            Err(e) => {
                                error!("[WS] Error when sending gateway: {e}: OPCODE 6")
                            }
                        };
                    }
                }
                Ok(m) => {
                    trace!("[WS] Got message");
                    let m = serde_json::from_str::<serde_json::Value>({
                        let m = m.to_text().unwrap();
                        if !m.is_empty() {
                            m
                        } else {
                            debug!("[WS] Skipped Message");
                            continue;
                        }
                    })
                    .unwrap();

                    match m["op"].as_u64().unwrap_or(0) {
                        0 => {
                            match m["t"].as_str().unwrap() {
                                "READY" => {
                                    info!("Connected to discord gateway!");
                                    session_id =
                                        m["d"]["session_id"].as_str().map(|s| s.to_string());
                                }
                                "GUILD_CREATE" => {
                                    debug!("Got GUILD_CREATE");
                                    let guild = match serde_json::from_value::<types::PartialGuild>(
                                        m["d"].clone(),
                                    ) {
                                        Err(e) => {
                                            error!("Error while parsing GUILD_CREATE event: {e:?}");
                                            continue;
                                        }
                                        Ok(g) => g,
                                    };

                                    guilds.insert(guild.id, guild);
                                }
                                "GUILD_DELETE" => {
                                    debug!("Got GUILD_DELETE");
                                    #[derive(serde::Deserialize)]
                                    struct GuildUnavailable {
                                        #[serde(deserialize_with = "deserialize_str")]
                                        id: u64,
                                        unavailable: Option<bool>,
                                    }
                                    let guild = match serde_json::from_value::<GuildUnavailable>(
                                        m["d"].clone(),
                                    ) {
                                        Err(e) => {
                                            error!("Error while parsing GUILD_DELETE event: {e:?}");
                                            continue;
                                        }
                                        Ok(g) => g,
                                    };
                                    guilds.remove(&guild.id);
                                }
                                "" => {}
                                event_name => trace!("Unhandled event: {event_name}"),
                            };

                            ACTION_ID.store(
                                m["s"].as_i64().unwrap(),
                                std::sync::atomic::Ordering::Relaxed,
                            );
                        }
                        10 => {
                            let token = token.clone();
                            let notify = notify.clone();
                            trace!("[WS] Hello from gateway");
                            let interval = m["d"]["heartbeat_interval"].as_u64().unwrap() as f64;
                            tokio::spawn({
                                let client = client_writer.clone();
                                let mut rng = rand::rngs::StdRng::from_entropy();
                                async move {
                                    let mut client_lock = client.lock().await;
                                    trace!("[WS] Sending first heartbeat");
                                    match client_lock
                                        .send(tokio_tungstenite::tungstenite::Message::Binary(
                                            serde_json::to_vec(&json!({
                                                "op": 1,
                                                "d": null,
                                            }))
                                            .unwrap(),
                                        ))
                                        .await
                                    {
                                        Ok(_) => trace!("[WS] Sent OPCODE 1"),
                                        Err(e) => {
                                            error!("[WS] Error when sending gateway: {e}")
                                        }
                                    };
                                    trace!("[WS] Sending OPCODE 2");
                                    match client_lock
                                        .send(tokio_tungstenite::tungstenite::Message::Binary(
                                            serde_json::to_vec(&json!({
                                                "op": 2,
                                                "d": {
                                                    "token": token,
                                                    "intents": INTENTS,
                                                    "properties": {
                                                        "os": "linux",
                                                        "browser": "dem.maix.me",
                                                        "device": "dem.maix.me"
                                                    }
                                                },
                                            }))
                                            .unwrap(),
                                        ))
                                        .await
                                    {
                                        Ok(_) => debug!("[WS] Sent OPCODE 2"),
                                        Err(e) => {
                                            error!("[WS] Error when sending gateway: {e}: OPCODE 2")
                                        }
                                    };

                                    drop(client_lock);

                                    loop {
                                        tokio::select!(
                                            _ = tokio::time::sleep(std::time::Duration::from_millis(
                                            (interval * rng.gen::<f64>()) as u64,
                                        )) => {},
                                                _ = notify.notified() => {}
                                        );
                                        match client.lock().await
                                                .send(
                                                    tokio_tungstenite::tungstenite::Message::Binary(
                                                        serde_json::to_vec(&json!({
                                                            "op": 1,
                                                            "d": ACTION_ID.load(std::sync::atomic::Ordering::Relaxed),
                                                        }))
                                                        .unwrap(),
                                                    ),
                                                )
                                                .await
                                            {
                                                Ok(_) => {}
                                                Err(e) => {
                                                    error!("[WS] Error when sending gateway: {e}")
                                                }
                                            };
                                    }
                                }
                            });
                        }
                        1 => {
                            trace!("[WS] Force heartbeat");
                            notify2.notify_one();
                        }
                        11 => trace!("[WS] Heartbeat ACK"),

                        n => {
                            debug!("[WS] Got OPCODE '{n}'")
                        }
                    }
                }
            };
        }
    }

    pub async fn from_figment(figment: &Figment) -> Result<Self, rocket::figment::Error> {
        #[derive(serde::Deserialize)]
        struct Config {
            discord_token: String,
            logged_user_cache: usize,
        }
        let config = figment.extract_inner::<Config>("dem")?;
        unsafe {
            BOT_AUTH_HEADER = Box::leak(format!("BOT {}", config.discord_token).into_boxed_str());
        }
        let guilds = Box::leak(Box::new(dashmap::DashMap::with_capacity(1024)));

        tokio::spawn(Self::handle_gateway(config.discord_token.clone(), guilds));
        Ok(Self {
            //user_cache: lru::LruCache::new(config.logged_user_cache),
            guilds,
            discord_token: config.discord_token,
            client: reqwest::Client::new(),
        })
    }

    pub fn get_guild(
        &self,
        guildid: u64,
    ) -> Option<dashmap::mapref::one::Ref<'_, u64, types::PartialGuild>> {
        self.guilds.get(&guildid)
    }

    pub async fn get_guilds_of_client_with_permission(
        &self,
        user_token: &str,
    ) -> Result<std::collections::HashMap<u64, u64>, Box<dyn std::error::Error + Send + Sync>> {
        #[derive(serde::Deserialize)]
        struct PartialGuildFromUser {
            #[serde(deserialize_with = "deserialize_str")]
            id: u64,
            #[serde(deserialize_with = "deserialize_str")]
            permissions: u64,
            owner: bool,
        }
        let response: Vec<PartialGuildFromUser> = self
            .client
            .get(format!("{DISCORD_API}/users/@me/guilds"))
            .header("Authorization", format!("Bearer {user_token}"))
            .send()
            .await?
            .json()
            .await?;
        let mut out = std::collections::HashMap::with_capacity(response.len());
        out.extend(
            response
                .into_iter()
                .map(|g| (g.id, g.permissions | if g.owner { 8 } else { 0 })),
        );
        Ok(out)
    }
}

fn deserialize_str<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let s = <String>::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}
