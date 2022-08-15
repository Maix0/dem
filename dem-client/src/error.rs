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
impl<T: Clone> CloneError<T> {
    pub fn from_error(err: dem_http::apis::Error<T>) -> Self {
        err.into()
    }
}

impl<T> CloneError<T> {}

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
        )*
    };
}
use dem_http::apis::default_api::*;
gen_error_name!(
    ApiGetCurrentUserError,
    ApiGetGuildEmojisError,
    ApiGetGuildStickersError,
    ApiGetOverlappingGuildsError
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
