use clap::{Parser, Subcommand};
mod sync;
use sync::{sync_conditions, sync_skills, sync_uma};

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
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::builder()
        .target(env_logger::Target::Stderr)
        .init();

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
            SyncTarget::All => {
                sync_skills(&db).await;
                sync_conditions(&db).await;
                sync_uma(&db).await;
            }
        },
    }
}
