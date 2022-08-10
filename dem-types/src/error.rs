use rocket::{response::status::BadRequest, serde::json::Json, Responder};
use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

macro_rules! define_error {
    (
    pub enum Error {
        $(
            $variant:ident = { description: $description:literal, code: $code:literal $(,)?}
        ),*
        $(,)?
    }
    ) => {
        #[derive(Debug ,Serialize_repr, Deserialize_repr, Clone, Copy, JsonSchema)]
        #[repr(u16)]
        pub enum Error {
            $(
                $variant = $code,
            )*
        }

        impl Error {
            pub fn into_err_with_desc(self, description: Option<String>) -> ErrResponse {
                match self {
                $(
                    Self::$variant => {
                        let description = description.unwrap_or_else(|| $description.into());
                        ErrResponse {
                            err: UserError {
                                code: self,
                                description,
                            }
                        }
                    }
                )*
                }
            }
            pub fn into_err(self) -> ErrResponse {
                self.into_err_with_desc(None)
            }
        }
    };
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct OkResponse<T> {
    pub ok: T,
}

impl<T> OkResponse<T> {
    pub fn ok(v: T) -> Rsp<T> {
        Rsp::Ok(Json(Self { ok: v }))
    }
}

#[derive(Responder, Debug)]
pub enum Rsp<T> {
    Ok(Json<OkResponse<T>>),
    Err(BadRequest<Json<ErrResponse>>),
}

impl<T> rocket_okapi::response::OpenApiResponderInner for Rsp<T>
where
    T: JsonSchema,
{
    fn responses(
        gen: &mut rocket_okapi::gen::OpenApiGenerator,
    ) -> rocket_okapi::Result<rocket_okapi::okapi::openapi3::Responses> {
        let mut response = rocket_okapi::okapi::openapi3::Responses::default();
        let ok_schema = gen.json_schema::<OkResponse<T>>();
        let err_schema = gen.json_schema::<ErrResponse>();
        rocket_okapi::util::add_schema_response(&mut response, 200, "application/json", ok_schema)?;
        rocket_okapi::util::add_schema_response(
            &mut response,
            400,
            "application/json",
            err_schema,
        )?;
        Ok(response)
    }
}

impl<T> Rsp<T> {
    pub fn ok(v: T) -> Self {
        OkResponse::ok(v)
    }

    pub fn err(e: Error, description: Option<String>) -> Self {
        Self::Err(BadRequest(Some(Json(e.into_err_with_desc(description)))))
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct UserError {
    pub code: Error,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ErrResponse {
    err: UserError,
}

define_error! {
    pub enum Error {
        Internal = {
            description: "internal error",
            code: 0,
        },
        DiscordAPI = {
            description: "discord api errror",
            code: 1,
        },
        Unauthorized = {
            description: "not authorized",
            code: 2,
        }
    }
}
