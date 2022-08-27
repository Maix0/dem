use material_yew::{MatIconButton, MatSnackbar};
use yew::prelude::*;

#[derive(Clone, Debug)]
pub enum CloneError<T> {
    ResponseError(dem_http::apis::ResponseContent<T>),
    Io(String),
    Reqwest(String),
    Serde(String),
}

impl<T: Clone> std::convert::From<dem_http::apis::Error<T>> for CloneError<T> {
    fn from(c: dem_http::apis::Error<T>) -> Self {
        match c {
            dem_http::apis::Error::Io(e) => Self::Io(e.to_string()),
            dem_http::apis::Error::Serde(e) => Self::Serde(e.to_string()),
            dem_http::apis::Error::Reqwest(e) => Self::Reqwest(e.to_string()),
            dem_http::apis::Error::ResponseError(r) => Self::ResponseError(r),
        }
    }
}

macro_rules! gen_error_name {
    ($($err:tt),*) => {
        $(
        #[allow(dead_code)]
        impl CloneError<$err> {
            pub fn catergory(&self) -> &'static str {
                match self {
                    CloneError::Io(_) => "Io",
                    CloneError::Serde(_) => "Api Deserialization",
                    CloneError::Reqwest(_) => "Http",
                    CloneError::ResponseError(r) => {
                        if let Some($err::Status400(err_response)) = &r.entity {
                            match err_response.err.code {
                                dem_http::models::error::Error::Internal => {
                                    "Internal"
                                }
                                dem_http::models::error::Error::DiscordAPI => {
                                    "Discord API"
                                }
                                dem_http::models::error::Error::Unauthorized => {
                                    "Unauthorize"
                                }
                                dem_http::models::error::Error::InvalidRequest => {
                                    "Invalid Request"
                                }
                            }
                        } else {
                            "Unknown"
                        }
                    }
                }
            }
            pub fn detail(&self) -> String {
                match self {
                    CloneError::Io(e) => e.to_string(),
                    CloneError::Serde(e) => e.to_string(),
                    CloneError::Reqwest(e) => e.to_string(),
                    CloneError::ResponseError(ref r) => {
                        if let Some($err::Status400(err_response)) = &r.entity {
                            err_response.err.description.clone()
                        } else {
                            "No description".to_string()
                        }

                    }
                }
            }
        }
    impl ::std::fmt::Display for CloneError<$err> {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                CloneError::Io(e) | CloneError::Serde(e) | CloneError::Reqwest(e) => ::std::fmt::Display::fmt(&e, f),
                CloneError::ResponseError(ref r) =>
                    ::std::fmt::Display::fmt(
                        if let Some($err::Status400(err_response)) = &r.entity {
                            err_response.err.description.as_str()
                        } else {
                            "No description"
                        },
                        f
                    ),
            }
        }
    }
    impl PartialEq for CloneError<$err> {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (CloneError::Io(e1), CloneError::Io(e2)) => e1 == e2,
                (CloneError::Serde(e1), CloneError::Serde(e2)) => e1 == e2,
                (CloneError::Reqwest(e1), CloneError::Reqwest(e2)) => e1 == e2,
                (CloneError::ResponseError(e1), CloneError::ResponseError(e2)) => {
                    e1.status == e2.status
                        && e1.content == e2.content
                        && match (&e1.entity, &e2.entity) {
                            (
                                &Some($err::Status400(ref e1)),
                                &Some($err::Status400(ref e2)),
                            ) => e1 == e2,
                            (
                                &Some($err::UnknownValue(ref e1)),
                                &Some($err::UnknownValue(ref e2)),
                            ) => e1 == e2,
                            _ => false,
                        }
                }
                _ => false,
            }
        }
    }

    impl std::error::Error for CloneError<$err> {}
    )*


    };
}
use dem_http::apis::default_api::*;
gen_error_name!(
    ApiGetCurrentUserError,
    ApiGetGuildEmojisError,
    ApiGetGuildStickersError,
    ApiGetOverlappingGuildsError,
    ImageUploadEmojiToStoreError,
    ImageImageListError
);

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct ErrorComponentProps {
    pub name: String,
    pub description: String,
}

pub struct ErrorComponent {
    link: material_yew::WeakComponentLink<MatSnackbar>,
}

impl Component for ErrorComponent {
    type Message = ();
    type Properties = ErrorComponentProps;

    fn create(_: &Context<Self>) -> Self {
        ErrorComponent {
            link: Default::default(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let ErrorComponentProps { name, description } = ctx.props();
        html! {
            <MatSnackbar label_text={format!("{} Error: {}", name, description)} snackbar_link={ self.link.clone() }>
                <span class="snackbar-dismiss-slot" slot="dismiss">
                    <MatIconButton icon="close" />
                </span>
            </MatSnackbar>
        }
    }

    fn rendered(&mut self, _: &Context<Self>, _: bool) {
        self.link.show();
    }
}
