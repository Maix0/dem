use discord::Logic;

#[macro_use]
extern crate rocket_okapi;
#[macro_use]
extern crate rocket;
extern crate dem_types;
extern crate okapi;
extern crate reqwest;
extern crate schemars;
extern crate tokio;

mod api;
mod auth;
mod discord;

pub use dem_types::error::{Error, OkResponse, Rsp};

#[rocket::launch]
async fn launch() -> _ {
    rocket::build()
        .mount("/dev", routes![get_emojis, get_stickers])
        .mount(
            "/swagger-ui",
            rocket_okapi::swagger_ui::make_swagger_ui(&rocket_okapi::swagger_ui::SwaggerUIConfig {
                url: "/api/openapi.json".to_string(),
                ..Default::default()
            }),
        )
        .mount("/api", openapi_get_routes![api::get_overlapping_guilds])
        .mount("/api/auth", routes![auth::login, auth::callback])
        .mount(
            "/",
            rocket::fs::FileServer::from({
                #[derive(serde::Deserialize)]
                struct Config {
                    r#static: String,
                }
                let dem = rocket::Config::figment().extract_inner::<Config>("dem");
                let dem = dem
                    .map_err(|e| error!("Error when using config: {e}"))
                    .unwrap();
                dem.r#static
            }),
        )
        .manage(
            Logic::from_figment(&rocket::Config::figment())
                .await
                .map_err(|e| error!("Error when parsing dem config: {e}"))
                .unwrap(),
        )
        .attach(rocket_oauth2::OAuth2::<auth::Discord>::fairing("discord"))
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
