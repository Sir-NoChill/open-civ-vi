//! `open4x-accounts` — CLI for ops tasks against the lobby's
//! accounts database. Today: `dump-audit`. Future: `delete-account`,
//! `prune-old-sessions`, `mint-session`.

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use open4x_accounts::audit::{AuditStore, SqliteAuditStore};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

#[derive(Parser)]
#[command(
    name = "open4x-accounts",
    version,
    about = "Ops CLI for the open4x-lobby accounts database"
)]
struct Cli {
    /// Path to the sqlite db. Defaults to ./data/lobby/accounts.sqlite
    /// (matches AppState::boot's default location).
    #[arg(long, default_value = "./data/lobby/accounts.sqlite")]
    db: PathBuf,

    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print the most recent audit_events rows, newest first.
    DumpAudit {
        /// How many rows to print.
        #[arg(short, long, default_value_t = 100)]
        limit: u32,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = Cli::parse();
    let opts = SqliteConnectOptions::new().filename(&cli.db);
    let pool = SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(opts)
        .await
        .unwrap_or_else(|e| {
            eprintln!("open: {e}");
            std::process::exit(2);
        });

    match cli.cmd {
        Command::DumpAudit { limit } => {
            let store = SqliteAuditStore::from_pool(pool);
            let rows = store.list_recent(limit).await.unwrap_or_else(|e| {
                eprintln!("query: {e}");
                std::process::exit(2);
            });
            // TSV: ts \t kind \t player_id_or_dash \t ip_or_dash \t detail
            println!("ts\tkind\tplayer_id\tip\tdetail");
            for r in rows {
                println!(
                    "{}\t{}\t{}\t{}\t{}",
                    r.ts,
                    r.kind.as_str(),
                    r.player_id.map(|p| p.display()).unwrap_or_else(|| "-".into()),
                    r.ip.unwrap_or_else(|| "-".into()),
                    r.detail.replace('\t', " ").replace('\n', " "),
                );
            }
        }
    }
}
