use serenity::async_trait;
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::env;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

pub async fn create_user_channel(user_id: u64) -> Result<(), serenity::Error> {
    // Convert user_id to UserId for Serenity
    let discord_user_id = UserId(user_id);
    let guild_id = GuildId(env::var("DISCORD_GUILD_ID").unwrap().parse().unwrap());

    // Channel name based on the user ID
    let user_channel_name = format!("user-{}", discord_user_id);

    // Create the channel in the specified guild/server
    guild_id.create_channel(&discord_user_id.http(), |c| c.name(&user_channel_name)).await?;
    Ok(())
}

pub async fn initialize_bot() -> Result<(), serenity::Error> {
    let config = crate::config::Config::load();
    let mut client = Client::builder(&config.token)
        .event_handler(Handler)
        .await?;

    client.start().await?;
    Ok(())
}