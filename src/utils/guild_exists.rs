use crate::prelude::*;

pub async fn guild_exists(ctx: Context<'_>) -> Result<bool, Error> {
    let guild_id = ctx.guild_id().unwrap();

    if !GuildSetup::guild_exists(guild_id).await? {
        ctx.reply("Your server is not registerd. Use \"/setup run\" to add your server")
            .await?;

        return Ok(false);
    }

    Ok(true)
}

pub async fn guild_not_setup(ctx: Context<'_>) -> Result<bool, Error> {
    let guild_id = ctx.guild_id().unwrap();

    if GuildSetup::guild_exists(guild_id).await? {
        ctx.reply("Your server is already registerd. Use \"/setup delete\" to remove your server")
            .await?;

        return Ok(false);
    }

    Ok(true)
}
