use dotenv::dotenv;
use std::env;

pub struct Config {
    pub token: String,
    pub guild_id: u64,
}

impl Config {
    pub fn load() -> Self {
        dotenv().ok();
        let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment");
        let guild_id = env::var("DISCORD_GUILD_ID")
            .expect("Expected DISCORD_GUILD_ID in environment")
            .parse()
            .expect("Expected DISCORD_GUILD_ID to be a valid u64");

        Config { token, guild_id }
    }
}