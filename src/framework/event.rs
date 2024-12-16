use serenity::http;

use crate::prelude::*;

#[allow(unused)]
pub async fn events(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    fw_context: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        serenity::FullEvent::CacheReady { guilds } => {
            sync_user_states(ctx, guilds.clone()).await?;
            leaderboards(ctx).await?;
        }
        serenity::FullEvent::VoiceStateUpdate { old, new } => {
            if let Some(guild_id) = new.guild_id {
                if GuildSetup::guild_exists(guild_id).await? {
                    handle_voice_event(new, ctx).await;
                }
            }
        }
        serenity::FullEvent::InteractionCreate { interaction } => {
            if let Some(button_press) = interaction.clone().message_component() {
                handle_role_toggle(button_press, ctx).await?;
            }
        }
        _ => {}
    }
    Ok(())
}

async fn handle_role_toggle(
    button_press: serenity::ComponentInteraction,
    ctx: &serenity::Context,
) -> Result<(), Error> {
    if !button_press.data.custom_id.starts_with("role_toggle-") {
        return Ok(());
    }
    let mut args = button_press.data.custom_id.split("-");
    let _ = args.next().unwrap();
    let guild_id: serenity::GuildId = args.next().unwrap().parse::<u64>().unwrap().into();
    let role_id: serenity::RoleId = args.next().unwrap().parse::<u64>().unwrap().into();

    match button_press
        .user
        .has_role(ctx.http(), guild_id, role_id)
        .await
        .unwrap()
    {
        true => {
            button_press
                .member
                .as_ref()
                .unwrap()
                .remove_role(ctx.http(), role_id)
                .await?;
            button_press
                .create_response(ctx.http(), serenity::CreateInteractionResponse::Acknowledge)
                .await?;
            UserDcEvent::delete(
                doc! {"metadata.guild_id": guild_id.to_string(), "metadata.user_id": button_press.user.id.to_string()},
            ).await?;
        }
        false => {
            button_press
                .member
                .as_ref()
                .unwrap()
                .add_role(ctx.http(), role_id)
                .await?;
            button_press
                .create_response(ctx.http(), serenity::CreateInteractionResponse::Acknowledge)
                .await?;
        }
    }
    Ok(())
}

async fn handle_voice_event(
    new: &serenity::VoiceState,
    ctx: &serenity::Context,
) -> Result<(), Error> {
    let guild_id = new.guild_id.unwrap();
    let user_id = new.user_id;
    let role = GuildSetup::get_data(guild_id).await?;
    let user = new.user_id.to_user(ctx.http()).await?;

    if user.bot {
        return Ok(());
    }

    if let Some(role) = role {
        if !user.has_role(ctx.http(), guild_id, role).await? {
            return Ok(());
        }
    };

    match (
        UserDcEvent::user_is_active(user_id, guild_id).await?,
        new.channel_id.is_some(),
    ) {
        (true, false) => {
            UserDcEvent::new(guild_id, user_id, UserEventType::Left)
                .insert()
                .await?;
        }
        (false, true) => {
            UserDcEvent::new(guild_id, user_id, UserEventType::Joined)
                .insert()
                .await?;
        }
        _ => {}
    }

    Ok(())
}
