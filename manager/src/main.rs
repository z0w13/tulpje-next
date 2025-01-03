#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // load .env into environment vars, ignore if not found
    match dotenvy::dotenv().map(|_| ()) {
        Err(err) if err.not_found() => eprintln!("warn: no .env file found"),
        Err(err) => eprintln!("warn: error loading env vars: {}", err),
        Ok(()) => (),
    };

    let token = match std::env::var("DISCORD_TOKEN") {
        Ok(token) => token,
        Err(std::env::VarError::NotPresent) => {
            eprintln!("DISCORD_TOKEN not set");
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("error reading DISCORD_TOKEN: {}", err);
            std::process::exit(1);
        }
    };

    let client = twilight_http::Client::builder().token(token).build();
    let connection_info = client.gateway().authed().await?.model().await?;

    println!("{}", connection_info.shards);

    Ok(())
}
