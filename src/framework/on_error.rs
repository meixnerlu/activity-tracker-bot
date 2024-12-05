use crate::prelude::*;

pub async fn on_error(error: poise::FrameworkError<'_, Data, Error>) -> Result<(), Error> {
    match error {
        poise::FrameworkError::Command { error, ctx, .. } => {
            ctx.reply("Error while executing your command").await?;
            log::error!("{error:?}");
        }
        poise::FrameworkError::MissingBotPermissions {
            missing_permissions,
            ctx,
            ..
        } => {
            log::error!(
                "In {:?} was missing permission {:?}",
                ctx.guild_id(),
                missing_permissions
            )
        }
        _ => {}
    };
    Ok(())
}
