use crate::prelude::*;

#[command(
    slash_command,
    guild_only,
    subcommands("delete", "role_button", "run", "edit"),
    subcommand_required,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn setup(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Use in channel for leaderboards
///
/// Starts a wizzard to help you setup your server
#[command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR",
    check = "guild_not_setup"
)]
pub async fn run(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    if !bool_select(
        ctx,
        guild_id,
        "Are you sure you want to create the leaderboard in this channel?",
    )
    .await?
    {
        return Ok(());
    }

    let role_to_watch = role_select(ctx, guild_id).await?;

    let msg = ctx
        .channel_id()
        .send_message(
            ctx.http(),
            serenity::CreateMessage::new().content("Leaderboard:\n"),
        )
        .await?;

    if GuildSetup::new(guild_id, ctx.channel_id(), role_to_watch, msg.id)
        .insert()
        .await
        .is_err()
    {
        ctx.reply("Your server is already registerd. Use \"/setup delete\" to remove your server")
            .await?;
    }

    sync_user_states(ctx.serenity_context(), vec![guild_id]).await?;

    Ok(())
}

/// Creates a message with a button where people can get the role
///
/// Requires a role to be set in the setup
#[command(
    slash_command,
    guild_only,
    required_bot_permissions = "MANAGE_ROLES",
    default_member_permissions = "ADMINISTRATOR",
    check = "guild_exists"
)]
async fn role_button(
    ctx: Context<'_>,
    #[description = "Label of the button (can be an emote)"] label: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let guild_setup = GuildSetup::get(doc! {"guild_id": guild_id.to_string()})
        .await?
        .unwrap();

    if guild_setup.role_to_watch.is_none() {
        ctx.reply("Your server did not set a role").await?;
        return Ok(());
    }
    let role = guild_setup.role_to_watch.unwrap();

    let button = vec![serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new(
            "role_toggle-".to_string() + &guild_id.to_string() + "-" + &role.to_string(),
        )
        .label(label),
    ])];
    ctx.channel_id()
        .send_message(
            ctx.http(),
            serenity::CreateMessage::default()
                .content(
                    "Click the button to toggle the ".to_string()
                        + &role.mention().to_string()
                        + " Role\n"
                        + "Removing the role removes all your past data on this server",
                )
                .components(button),
        )
        .await?;

    Ok(())
}

#[command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR",
    check = "guild_exists"
)]
async fn edit(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let buttons = vec![serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new(guild_id.to_string() + "-role").label("role"),
        serenity::CreateButton::new(guild_id.to_string() + "-nothing").label("nothing"),
    ])];

    let msg = ctx
        .send(
            poise::CreateReply::default()
                .content("What do you want to change?\nChanging these does not effect past data.")
                .reply(true)
                .components(buttons),
        )
        .await?;

    let mut reactions = msg
        .message()
        .await?
        .await_component_interactions(&ctx.serenity_context().shard)
        .stream();

    while let Some(reaction) = reactions.next().await {
        if &reaction.user == ctx.author()
            && reaction.data.custom_id.starts_with(&guild_id.to_string())
        {
            msg.delete(ctx).await?;

            if reaction.data.custom_id.ends_with("-role") {
                {
                    let role_to_watch = role_select(ctx, guild_id)
                        .await?
                        .map(|role| role.to_string());

                    GuildSetup::change(
                        doc! {"guild_id": guild_id.to_string()},
                        doc! {"$set": doc! {"role_to_watch": role_to_watch}},
                    )
                    .await?;
                }
            }
        }
    }

    Ok(())
}

/// Removes your Server and all user data
#[command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR",
    check = "guild_exists"
)]
async fn delete(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    if bool_select(
        ctx,
        guild_id,
        "Are you sure you want to remove this server?",
    )
    .await?
    {
        GuildSetup::remove(guild_id).await?;
        UserDcEvent::delete(doc! {"metadata.guild_id": guild_id.to_string()}).await?;
    }
    Ok(())
}
