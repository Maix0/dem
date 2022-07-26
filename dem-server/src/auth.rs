use rocket::{
    http::CookieJar,
    request::{self, FromRequest},
    response::Redirect,
    Request, Route,
};
use rocket_okapi::OpenApiFromRequest;

pub struct Discord;

#[get("/")]
pub async fn login(oauth2: rocket_oauth2::OAuth2<Discord>, cookies: &CookieJar<'_>) -> Redirect {
    oauth2
        .get_redirect(cookies, &["identify", "guilds"])
        .unwrap()
}

#[get("/callback")]
pub fn callback(token: rocket_oauth2::TokenResponse<Discord>, cookies: &CookieJar<'_>) -> Redirect {
    cookies.add_private(
        rocket::http::Cookie::build("token", token.access_token().to_string())
            .same_site(rocket::http::SameSite::Lax)
            .finish(),
    );
    Redirect::to("/")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserAuthError {
    NoToken,
}

#[derive(Clone, Debug, OpenApiFromRequest)]
pub struct User {
    pub token: String,
}

#[async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = UserAuthError;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match req.cookies().get_private("token") {
            None => request::Outcome::Failure((
                rocket::http::Status::Unauthorized,
                UserAuthError::NoToken,
            )),
            Some(token) => request::Outcome::Success(User {
                token: token.value().to_string(),
            }),
        }
    }
}
