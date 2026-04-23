use clap::{Parser, Subcommand};
mod sync;
use sync::{sync_characters, sync_skill_details, sync_skills};

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
    SkillDetails,
    Characters,
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
            SyncTarget::Skills => sync_skills(&db).await,
            SyncTarget::SkillDetails => sync_skill_details(&db).await,
            SyncTarget::Characters => sync_characters(&db).await,
            SyncTarget::All => {
                sync_skills(&db).await;
                sync_skill_details(&db).await;
                sync_characters(&db).await;
            }
        },
    }
}
