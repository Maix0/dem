use std::time::SystemTime;

use rocket::{
    http::CookieJar,
    request::{self, FromRequest},
    response::Redirect,
    serde::json::Json,
    Request, State,
};
use rocket_db_pools::deadpool_redis::redis::Cmd;

pub struct Discord;

#[get("/")]
pub async fn login(
    oauth2: rocket_oauth2::OAuth2<Discord>,
    cookies: &CookieJar<'_>,
    _logic: &State<crate::discord::Logic>,
    mut con: crate::Connection<crate::DemDb>,
) -> Result<Redirect, rocket::http::Status> {
    if let Some(token) = cookies.get_private("token") {
        // if logic.user_cache.get(token.value())
        let res = Cmd::get(token.value())
            .query_async::<_, Option<u64>>(&mut *con)
            .await
            .map_err(|e| {
                error!("Error when comunicating with redis db: {e}");
                rocket::http::Status::InternalServerError
            })?;
        if res.is_some() {
            return Ok(Redirect::to("/"));
        }
    }
    Ok(oauth2
        .get_redirect(cookies, &["identify", "guilds"])
        .unwrap())
}

#[get("/callback")]
pub async fn callback(
    token: rocket_oauth2::TokenResponse<Discord>,
    cookies: &CookieJar<'_>,
    mut con: crate::Connection<crate::DemDb>,
    logic: &State<crate::discord::Logic>,
) -> Result<Redirect, rocket::http::Status> {
    let user = logic.get_user(token.access_token()).await.map_err(|e| {
        error!("Error when communicating with the Discord API: {e}");
        rocket::http::Status::InternalServerError
    })?;
    Cmd::set_ex::<_, u64>(
        token.access_token(),
        user.id,
        token.expires_in().unwrap_or(i64::MAX).max(0) as usize,
    )
    .query_async::<_, ()>(&mut *con)
    .await
    .map_err(|e| {
        error!("Error when comunicating with redis db: {e}");
        rocket::http::Status::InternalServerError
    })?;

    let guilds = logic
        .get_guilds_of_client_with_permission(token.access_token())
        .await
        .map_err(|e| {
            error!("Error when communicating with Discord API: {e}");
            rocket::http::Status::InternalServerError
        })?;

    logic
        .user_id_to_token
        .write()
        .await
        .push(user.id, token.access_token().to_string());
    logic.user_cache.write().await.push(
        token.access_token().to_string(),
        crate::discord::LoggedUser {
            expires_at: SystemTime::now()
                + std::time::Duration::from_secs(
                    token.expires_in().map(|s| s.max(0) as u64).unwrap_or(3600),
                ),
            user_id: user.id,
            user_icon: user.avatar.map(|s| {
                if s.starts_with("_a") {
                    s.strip_prefix("_a").unwrap().to_string()
                } else {
                    s
                }
            }),
            username: user.username,
            discriminator: user.discriminator,
            guilds,
        },
    );

    cookies.add_private(
        rocket::http::Cookie::build("token", token.access_token().to_string())
            .same_site(rocket::http::SameSite::Lax)
            .expires(
                Some(
                    SystemTime::now()
                        + std::time::Duration::from_secs(
                            token
                                .expires_in()
                                .map(|s| s.max(0) as u64)
                                .unwrap_or(u64::MAX),
                        ),
                )
                .map(rocket::time::OffsetDateTime::from),
            )
            .finish(),
    );
    Ok(Redirect::to("/"))
}

#[get("/logout")]
pub async fn logout(logic: &State<crate::discord::Logic>, cookies: &CookieJar<'_>) -> Json<bool> {
    if let Some(token) = cookies.get_private("token") {
        let user = logic.user_cache.write().await.pop_entry(token.value());
        if let Some(user) = user {
            logic
                .user_id_to_token
                .write()
                .await
                .pop_entry(&user.1.user_id);
        }
        cookies.remove_private(token);
        return Json(true);
    }
    Json(false)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserAuthError {
    NoToken,
    InvalidToken,
    InternalError,
}

#[derive(Clone, Debug, OpenApiFromRequest)]
pub struct User {
    pub token: String,
}

#[async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = UserAuthError;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let cookies = req.cookies();
        let mut con = req
            .guard::<crate::Connection<crate::DemDb>>()
            .await
            .unwrap();
        let logic = req.guard::<&State<crate::discord::Logic>>().await.unwrap();
        if let Some(token) = cookies.get_private("token") {
            let mut lock = logic.user_cache.write().await;
            let mut lock_id = logic.user_id_to_token.write().await;
            let user = lock.get(token.value());
            if let Some(_user) = user {
                return request::Outcome::Success(Self {
                    token: token.value().to_string(),
                });
            } else {
                let res = match Cmd::get(token.value())
                    .query_async::<_, Option<u64>>(&mut *con)
                    .await
                    .map_err(|e| error!("Error when comunicating with redis db: {e}"))
                {
                    Ok(o) => o,
                    Err(_) => {
                        return request::Outcome::Failure((
                            rocket::http::Status::InternalServerError,
                            UserAuthError::InternalError,
                        ))
                    }
                };
                if res.is_some() {
                    let user = match logic.get_user(token.value()).await.map_err(|e| {
                        error!("Error when communicating with the Discord API: {e}");
                        rocket::http::Status::InternalServerError
                    }) {
                        Ok(o) => o,
                        Err(e) => {
                            error!("Error when comunicating with the Discord API: {e}");
                            return request::Outcome::Failure((
                                rocket::http::Status::InternalServerError,
                                UserAuthError::InternalError,
                            ));
                        }
                    };

                    let guilds = match logic
                        .get_guilds_of_client_with_permission(token.value())
                        .await
                    {
                        Ok(g) => g,
                        Err(e) => {
                            error!("Error when comunicating with Discord API: {e}");
                            return request::Outcome::Failure((
                                rocket::http::Status::InternalServerError,
                                UserAuthError::InternalError,
                            ));
                        }
                    };

                    lock_id.push(user.id, token.value().to_string());
                    lock.push(
                        token.value().to_string(),
                        crate::discord::LoggedUser {
                            expires_at: SystemTime::now()
                                + std::time::Duration::from_secs(
                                    token
                                        .expires_datetime()
                                        .map(std::time::SystemTime::from)
                                        .map(|t| {
                                            match t
                                                .elapsed()
                                                .map(|_| 0)
                                                .map_err(|e| e.duration().as_secs())
                                            {
                                                Ok(e) | Err(e) => e,
                                            }
                                        })
                                        .map(|s| s.max(0) as u64)
                                        .unwrap_or(0),
                                ),
                            user_id: user.id,
                            user_icon: user.avatar.map(|s| {
                                if s.starts_with("_a") {
                                    s.strip_prefix("_a").unwrap().to_string()
                                } else {
                                    s
                                }
                            }),
                            username: user.username,
                            discriminator: user.discriminator,
                            guilds,
                        },
                    );
                    return request::Outcome::Success(Self {
                        token: token.value().to_string(),
                    });
                } else {
                    return request::Outcome::Failure((
                        rocket::http::Status::BadRequest,
                        UserAuthError::InvalidToken,
                    ));
                }
            }
        } else {
            return request::Outcome::Failure((
                rocket::http::Status::Unauthorized,
                UserAuthError::NoToken,
            ));
        }
    }
}
