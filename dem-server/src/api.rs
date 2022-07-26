use crate::{Error, Rsp};
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
