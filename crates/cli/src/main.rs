use anyhow::*;
use clap::{Parser, Subcommand};
use tracing_subscriber::{EnvFilter, fmt};

#[derive(Parser, Debug)]
#[command(name = "filecord", version, about)]
struct Args {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Initialize .filecord and write config
    Init {
        #[arg(long)] guild: String,
        #[arg(long, name="root-channel")] root_channel: String,
    },
    /// Show basic info (stub)
    Info,
}

#[tokio::main]
async fn main() -> Result<()> {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();
    let args = Args::parse();
    match args.cmd {
        Cmd::Init { guild, root_channel } => cmd_init(&guild, &root_channel).await?,
        Cmd::Info => println!("stub: info"),
    }
    Ok(())
}

async fn cmd_init(guild: &str, root_channel: &str) -> Result<()> {
    use filecord_fsindex::{Repo, RepoConfig};
    std::fs::create_dir_all(".filecord")?;
    let repo = Repo::new(".filecord");
    repo.ensure_layout()?;
    // Load from env as convenience
    let bot_token = std::env::var("DISCORD_BOT_TOKEN").unwrap_or_default();
    let application_id = std::env::var("DISCORD_APP_ID").unwrap_or_default();
    let public_key = std::env::var("DISCORD_PUBLIC_KEY").unwrap_or_default();
    let cfg = RepoConfig {
        bot_token,
        application_id,
        guild_id: guild.to_string(),
        root_channel_id: root_channel.to_string(),
        public_key,
        ..Default::default()
    };
    repo.save_config(&cfg)?;
    println!("Initialized .filecord with guild={} root_channel={}", guild, root_channel);
    Ok(())
}
