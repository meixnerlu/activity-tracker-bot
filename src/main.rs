mod commands;
mod framework;
mod prelude;
mod utils;

use commands::*;
use prelude::*;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::all();
    let _ = UserDcEvent::setup_collection().await;
    let _ = GuildSetup::setup_collection().await;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![setup()],
            on_error: |error| {
                Box::pin(async move {
                    if let Err(e) = framework::on_error(error).await {
                        log::error!("{e:#?}");
                    }
                })
            },
            event_handler: |ctx, event, framework, data| {
                Box::pin(crate::framework::events(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(&ctx.http, &framework.options().commands)
                    .await
                    .unwrap();
                Ok(Data::new().await)
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
