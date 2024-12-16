use crate::prelude::*;

pub async fn sync_user_states(
    ctx: &serenity::Context,
    cached_guilds: Vec<serenity::GuildId>,
) -> Result<(), Error> {
    let guilds: Vec<GuildSetup> = GuildSetup::get_guilds()
        .await?
        .iter()
        .filter(|&guild| cached_guilds.contains(&guild.guild_id))
        .cloned()
        .collect();

    for guild in guilds {
        let db_active = UserDcEvent::active_users(guild.guild_id).await?;
        let dc_active = get_dc_active_users(ctx, &guild.guild_id).await?;

        for db_active_user in db_active.clone() {
            if !dc_active.contains(&db_active_user) {
                UserDcEvent::new(guild.guild_id, db_active_user, UserEventType::Left)
                    .insert()
                    .await?;
            }
        }

        for dc_active_user in dc_active {
            if !db_active.contains(&dc_active_user) {
                let user = dc_active_user.to_user(ctx.http()).await?;
                if user.bot {
                    continue;
                }
                match guild.role_to_watch {
                    Some(role) => {
                        if user.has_role(ctx.http(), &guild.guild_id, role).await? {
                            UserDcEvent::new(guild.guild_id, dc_active_user, UserEventType::Joined)
                                .insert()
                                .await?;
                        }
                    }
                    None => {
                        UserDcEvent::new(guild.guild_id, dc_active_user, UserEventType::Joined)
                            .insert()
                            .await?;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn get_dc_active_users(
    ctx: &serenity::Context,
    guild_id: &serenity::GuildId,
) -> Result<Vec<serenity::UserId>, Error> {
    let out = vec![];

    if let Some(cached_guild) = guild_id.to_guild_cached(ctx.cache().unwrap()) {
        return Ok(cached_guild
            .voice_states
            .iter()
            .filter(|(_, state)| state.channel_id.is_some())
            .map(|(user, _)| *user)
            .collect());
    };

    Ok(out)
}
