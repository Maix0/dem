use bounce::query::*;
use stylist::yew::*;
use yew::prelude::*;

use crate::error;

#[derive(Clone, PartialEq, Debug, Properties)]
pub struct EmojiListProps {
    pub id: u64,
}

#[styled_component(GuildEmojiList)]
pub fn emoji_list(props: &EmojiListProps) -> Html {
    let emojis = use_query_value::<crate::query::GuildEmoteQuery>(props.id.into());

    match emojis.result() {
        None => html! {"Loading"},
        Some(Err(e)) => {
            html! {<error::ErrorComponent name={e.catergory()} description={e.detail()} />}
        }
        Some(Ok(emojis)) => html! {
            <div class={css!("display: flex; flex-direction: row; flex-wrap: wrap; justify-content: space-between;")}>
                {
                    emojis.iter().map(|e| html! {
                        <EmojiListItem inner={e.clone()}/>
                    }).collect::<Html>()
                }
            </div>
        },
    }
}

#[derive(Clone, Debug, Properties, PartialEq)]
struct EmojiListItemProps {
    pub inner: dem_http::models::EmojiItem,
}

#[styled_component(EmojiListItem)]
fn emoji_list_item(props: &EmojiListItemProps) -> Html {
    html! {
        <div class={css!("cursor: grab; display: flex; flex-direction: column; height: 12rem; width: 10rem; align-items: center; justify-content: space-evenly; color: var(--mdc-theme-on-surface); background-color: var(--mdc-theme-surface); border-radius: 0.5rem; margin: 0.5rem;")}>
            <span class={css!("height: 1rem;")}> {&props.inner.name} </span>
            <img class={css!("width: 9rem; max-height: 9rem;")} src={format!("https://cdn.discordapp.com/emojis/{}.{}",props.inner.id, if props.inner.animated {"gif"} else {"png"})} />
        </div>
    }
}

#[derive(Clone, PartialEq, Debug, Properties)]
pub struct UploadedEmojiListProps {
    pub id: u64,
}

#[styled_component(UploadedEmojiList)]
pub fn uploaded_emoji_list(props: &UploadedEmojiListProps) -> Html {
    let emojis = use_query_value::<crate::query::GetUploadedEmojisQuery>(props.id.into());
    html! {
        <div>
        <h2 class={css!("color: var(--mdc-theme-on-surface); border-bottom-color: var(--mdc-theme-on-surface); border-bottom-style: solid; border-bottom-width: 5px;")}>
            {"Uploaded Emojis"}
        </h2>
        <div class={css!("display: flex; flex-direction: row; flex-wrap: wrap; justify-content: space-between;")}>
            {
                match emojis.result() {
                    None => Html::default(),
                    Some(Ok(emojis)) => emojis.iter().map(|v| html!{
                        <UploadedEmojiListItem name={v.name.clone()} uuid={v.uuid.clone()} imagetype={v.image_type} guildid={props.id}/>
                    }).collect::<Html>(),
                    Some(Err(e)) => html! {<error::ErrorComponent name={e.catergory()} description={e.detail()} />},
                }
            }
        </div>
        </div>
    }
}

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct UploadedEmojiListItemProps {
    name: String,
    uuid: String,
    imagetype: dem_http::models::ImageType,
    guildid: u64,
}

#[styled_component(UploadedEmojiListItem)]
fn uploaded_emoji_list_item(props: &UploadedEmojiListItemProps) -> Html {
    html! {
        <div class={css!("cursor: grab;display: flex; flex-direction: column; height: 12rem; width: 10rem; align-items: center; justify-content: space-evenly; color: var(--mdc-theme-on-surface); background-color: var(--mdc-theme-surface); border-radius: 0.5rem; margin: 0.5rem;")}>
            <span class={css!("height: 1rem;")}> {&props.name} </span>
            <img class={css!("width: 9rem; max-height: 9rem;")} src={format!("/store/{}/{}",props.guildid, props.uuid, 
        //match props.imagetype {
        //    dem_http::models::ImageType::Png => "png",
        //    dem_http::models::ImageType::Gif => "gif",
        //}
    )} />
        </div>
    }
}
