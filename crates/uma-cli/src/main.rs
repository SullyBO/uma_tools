use clap::{Parser, Subcommand};
mod sync;
use sync::{sync_cards, sync_conditions, sync_skills, sync_uma};

/// To run the CLI simply `cargo run -p uma_cli -- {COMMAND} {SUBCOMMAND}`
#[derive(Parser)]
#[command(name = "uma", about = "Umamusume data CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Sync {
        #[command(subcommand)]
        target: SyncTarget,
    },
}

#[derive(Subcommand)]
enum SyncTarget {
    Skills,
    Uma,
    All,
    Cards,
}

#[tokio::main]
async fn main() {
    let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());
    let env_file = match app_env.as_str() {
        "prod" => ".env.prod",
        _ => ".env",
    };
    dotenvy::from_filename(env_file).ok();
    env_logger::init();
    log::info!("Running in {app_env} environment");

    let db = uma_db::db::Db::connect()
        .await
        .expect("failed to connect to database");

    let cli = Cli::parse();

    match cli.command {
        Commands::Sync { target } => match target {
            SyncTarget::Skills => {
                sync_conditions(&db).await;
                sync_skills(&db).await
            }
            SyncTarget::Uma => sync_uma(&db).await,
            SyncTarget::Cards => sync_cards(&db).await,
            SyncTarget::All => {
                sync_skills(&db).await;
                sync_conditions(&db).await;
                sync_uma(&db).await;
                sync_cards(&db).await;
            }
        },
    }
}
