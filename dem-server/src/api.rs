use crate::{discord::Logic, Error, Rsp};
use std::collections::HashMap;

#[openapi]
#[get("/overlapping_guilds")]
pub async fn get_overlapping_guilds(
    user: crate::auth::User,
    logic: &rocket::State<crate::discord::Logic>,
) -> Rsp<Vec<dem_types::discord::PartialGuildWithPermission>> {
    let user_guilds: HashMap<u64, u64> = match logic
        .get_guilds_of_client_with_permission(&user.token)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            error!("Error when requesting user's guilds: {e}");
            return Rsp::err(Error::DiscordAPI, Some(format!("{e}")));
        }
    };
    let overlapping = {
        let mut guilds = Vec::with_capacity(200);
        for entry in logic.guilds.iter() {
            if user_guilds.contains_key(entry.key()) {
                guilds.push(
                    dem_types::discord::PartialGuildWithPermission::from_partial_guild(
                        entry.value().clone(),
                        user_guilds[entry.key()],
                    ),
                )
            }
        }
        guilds
    };
    Rsp::ok(overlapping)
}

#[openapi]
#[get("/user")]
pub async fn get_current_user(
    user: Result<crate::auth::User, crate::auth::UserAuthError>,
    logic: &rocket::State<crate::discord::Logic>,
) -> Rsp<Option<crate::dem_types::api::UserLogin>> {
    match user {
        Ok(crate::auth::User { token }) => match logic.get_user(&token).await {
            Ok(u) => Rsp::ok(Some(dem_types::api::UserLogin {
                username: u.username,
                avatar: u.avatar,
                id: u.id,
                discriminator: u.discriminator,
            })),
            Err(e) => {
                error!("Internal Error when fetching user from cache: {e}");
                Rsp::err(crate::Error::Internal, None)
            }
        },
        Err(crate::auth::UserAuthError::InvalidToken | crate::auth::UserAuthError::NoToken) => {
            Rsp::ok(None)
        }
        Err(crate::auth::UserAuthError::InternalError) => Rsp::err(crate::Error::Internal, None),
    }
}

#[openapi]
#[get("/guild/<id>/emojis")]
pub async fn get_guild_emojis(
    user: crate::auth::User,
    logic: &rocket::State<crate::discord::Logic>,
    id: u64,
) -> Rsp<Vec<dem_types::discord::EmojiItem>> {
    if let Some(u) = logic.user_cache.write().await.get(&user.token) {
        if u.guilds.contains_key(&id) {
            Rsp::ok(
                match logic.get_guild(id).map(|kv| (*kv).emojis.clone()) {
                    Some(o) => o,
                    None => {
                        return Rsp::err(Error::Internal, None);
                    }
                },
            )
        } else {
            Rsp::err(Error::Unauthorized, "Not in the guild".to_string().into())
        }
    } else {
        Rsp::err(Error::Unauthorized, None)
    }
}


#[openapi]
#[get("/guild/<id>/stickers")]
pub async fn get_guild_stickers(
    user: crate::auth::User,
    logic: &rocket::State<crate::discord::Logic>,
    id: u64,
) -> Rsp<Vec<dem_types::discord::StickerItem>> {
    if let Some(u) = logic.user_cache.write().await.get(&user.token) {
        if u.guilds.contains_key(&id) {
            Rsp::ok(
                match logic.get_guild(id).map(|kv| (*kv).stickers.clone()) {
                    Some(o) => o,
                    None => {
                        return Rsp::err(Error::Internal, None);
                    }
                },
            )
        } else {
            Rsp::err(Error::Unauthorized, "Not in the guild".to_string().into())
        }
    } else {
        Rsp::err(Error::Unauthorized, None)
    }
}
