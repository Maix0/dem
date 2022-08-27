use discord::Logic;

#[macro_use]
extern crate rocket_okapi;
#[macro_use]
extern crate rocket;
extern crate dem_types;
extern crate okapi;
extern crate reqwest;
extern crate rocket_db_pools;
extern crate schemars;
extern crate tokio;
extern crate fxhash;

mod api;
mod auth;
mod discord;
mod image;
mod retry_middleware;

pub use dem_types::error::{Error, Rsp};
use rocket_db_pools::{deadpool_redis::Pool, Connection, Database};

#[derive(Database)]
#[database("dem_db")]
pub struct DemDb(Pool);

#[rocket::launch]
async fn launch() -> _ {
    let tmp_dir: String = rocket::Config::figment()
        .extract_inner("dem.temp_image_dir")
        .expect("You need to specity a custom tmp dir");
    match tokio::fs::create_dir_all(&tmp_dir).await {
        Ok(_) => Ok(()),
        Err(e) => match e.kind() {
            std::io::ErrorKind::AlreadyExists => Ok(()),
            _ => Err(e),
        },
    }
    .expect("Unable to create temp_dir");
    let img_store: String = rocket::Config::figment()
        .extract_inner("dem.image_store")
        .expect("You need to specity a image_store location");
    match tokio::fs::create_dir(&img_store).await {
        Ok(_) => Ok(()),
        Err(e) => match e.kind() {
            std::io::ErrorKind::AlreadyExists => Ok(()),
            _ => Err(e),
        },
    }
    .expect("Unable to create image store directory");
    rocket::build()
        .mount("/dev", routes![get_emojis, get_stickers, get_user])
        .mount("/store", routes![image::image_serve])
        .mount(
            "/swagger-ui",
            rocket_okapi::swagger_ui::make_swagger_ui(&rocket_okapi::swagger_ui::SwaggerUIConfig {
                url: "/api/openapi.json".to_string(),
                ..Default::default()
            }),
        )
        .mount(
            "/api",
            openapi_get_routes![
                api::get_overlapping_guilds,
                api::get_current_user,
                api::get_guild_emojis,
                api::get_guild_stickers,
                image::upload_emoji_to_store,
                image::image_list,
            ],
        )
        .mount(
            "/api/auth",
            routes![auth::login, auth::callback, auth::logout],
        )
        .mount(
            "/",
            rocket::fs::FileServer::from(
                rocket::Config::figment()
                    .extract_inner::<std::path::PathBuf>("dem.static_page")
                    .map_err(|e| error!("You must include the static_page filed: {e}"))
                    .unwrap(),
            ),
        )
        .manage(
            Logic::from_figment(&rocket::Config::figment())
                .await
                .map_err(|e| error!("Error when parsing dem config: {e}"))
                .unwrap(),
        )
        .manage(image::ImageStore::from_figment(&rocket::Config::figment()))
        .attach(rocket_oauth2::OAuth2::<auth::Discord>::fairing("discord"))
        .attach(DemDb::init())
}

#[get("/get_emojis?<guildid>")]
async fn get_emojis(
    logic: &rocket::State<Logic>,
    guildid: u64,
) -> rocket::serde::json::Json<Vec<dem_types::discord::EmojiItem>> {
    rocket::serde::json::Json(
        logic
            .get_guild(guildid)
            .map(|kv| (*kv).emojis.clone())
            .unwrap_or_else(Vec::new),
    )
}

#[get("/user")]
async fn get_user(user: auth::User) -> String {
    user.token
}

#[get("/get_stickers?<guildid>")]
async fn get_stickers(
    logic: &rocket::State<Logic>,
    guildid: u64,
) -> rocket::serde::json::Json<Vec<dem_types::discord::StickerItem>> {
    rocket::serde::json::Json(
        logic
            .get_guild(guildid)
            .map(|kv| (*kv).stickers.clone())
            .unwrap_or_else(Vec::new),
    )
}
