use crate::prelude::*;

pub async fn role_select(
    ctx: Context<'_>,
    guild_id: serenity::GuildId,
) -> Result<Option<serenity::RoleId>, Error> {
    let buttons = vec![
        serenity::CreateActionRow::SelectMenu(serenity::CreateSelectMenu::new(
            guild_id.to_string() + "-role",
            serenity::CreateSelectMenuKind::Role {
                default_roles: None,
            },
        )),
        serenity::CreateActionRow::Buttons(vec![serenity::CreateButton::new(
            guild_id.to_string() + "-no",
        )
        .label("no")]),
    ];

    let msg = ctx
        .send(
            poise::CreateReply::default()
                .content(
                    "Do you just want to track a specific role?\n
                    You can later create a message for your members to get the role with \"/setup role_button\"",
                )
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
            let role_to_watch = match reaction.data.kind {
                serenity::ComponentInteractionDataKind::Button => None,
                serenity::ComponentInteractionDataKind::RoleSelect { values } => {
                    Some(values.first().copied().unwrap())
                }
                _ => None,
            };

            msg.delete(ctx).await?;

            return Ok(role_to_watch);
        }
    }

    Ok(None)
}

pub async fn bool_select(
    ctx: Context<'_>,
    guild_id: serenity::GuildId,
    text: impl Into<String>,
) -> Result<bool, Error> {
    let text: String = text.into();

    let buttons = vec![serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new(guild_id.to_string() + "-yes").label("yes"),
        serenity::CreateButton::new(guild_id.to_string() + "-no").label("no"),
    ])];

    let msg = ctx
        .send(
            poise::CreateReply::default()
                .content(text)
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
            return Ok(reaction.data.custom_id.ends_with("-yes"));
        }
    }

    Ok(false)
}
