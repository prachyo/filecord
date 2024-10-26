use actix_web::{App, HttpServer};
use dotenv::dotenv;
use log::info;
use tokio::task;

mod bot;
mod config;
mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    // Spawn the Discord bot in a separate asynchronous task
    let bot_task = task::spawn(async {
        if let Err(err) = bot::initialize_bot().await {
            println!("Bot error: {:?}", err);
        }
    });

    // Start the Actix-web server
    HttpServer::new(|| {
        App::new()
            .configure(routes::init)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    bot_task.await.unwrap();
    Ok(())
}