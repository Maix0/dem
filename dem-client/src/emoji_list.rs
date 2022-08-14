use stylist::yew::*;
use yew::prelude::*;
use yew_hooks::prelude::*;

use crate::error::{CloneError, self};
use crate::{enable_auto, APIConfig};

#[derive(Clone, PartialEq, Debug, Properties)]
pub struct EmojiListProps {
    pub id: u64,
}

#[styled_component(GuildEmojiList)]
pub fn emoji_list(props: &EmojiListProps) -> Html {
    let api_config = use_context::<APIConfig>().unwrap();
    let emojis = use_async_with_options(
        {
            let id = props.id;
            async move {
                dem_http::apis::default_api::api_get_guild_emojis(&api_config, id)
                    .await
                    .map_err(CloneError::from_error)
            }
        },
        enable_auto(),
    );
    html! {
        {
            if emojis.loading {
                html!{"Loading"}
            }
            else if let Some(emojis) = &emojis.data {
                html! {
                    <div class={css!("display: flex; flex-direction: row; flex-wrap: wrap; justify-content: space-between;")}>
                    {
                        emojis.ok.iter().map(|e| html! {
                            <EmojiListItem inner={e.clone()}/>
                        }).collect::<Html>()
                    }
                    </div>
                }
            } else if let Some(error) = &emojis.error {
                html!{<error::ErrorComponent name={error.catergory()} description={error.detail()} />}
            } else {
                html! {<error::ErrorComponent name={"DEV"} description={"Maix fucked up"}/> }
            }
        }
    }
}

#[derive(Clone, Debug, Properties, PartialEq)]
struct EmojiListItemProps {
    pub inner: dem_http::models::EmojiItem,
}

#[styled_component(EmojiListItem)]
fn emoji_list_item(props: &EmojiListItemProps) -> Html {
    html! {
        <div class={css!("display: flex; flex-direction: column; height: 12rem; width: 10rem; align-items: center; justify-content: space-evenly; color: var(--mdc-theme-on-surface); background-color: var(--mdc-theme-surface); border-radius: 0.5rem; margin: 0.5rem;")}>
            <span class={css!("height: 1rem;")}> {&props.inner.name} </span>
            <img class={css!("width: 9rem; max-height: 9rem;")} src={format!("https://cdn.discordapp.com/emojis/{}.{}",props.inner.id, if props.inner.animated {"gif"} else {"png"})} />
        </div>
    }
}
