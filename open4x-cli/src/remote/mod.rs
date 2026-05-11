//! Remote dispatch — runs every CLI subcommand against an
//! `open4x-server` HTTP API instead of an in-process rules engine.
//!
//! Plan: `book/src/roadmap/cli-server-mode.md`. The local handlers in
//! `crate::handlers` are untouched; this module is a parallel
//! dispatcher selected by the `--server <URL>` global flag.

pub mod action;
pub mod bootstrap;
pub mod client;
pub mod end_turn;
pub mod list;
pub mod session;
pub mod status;
pub mod view;

use std::path::Path;

use crate::cli::Command;

/// Dispatch a parsed CLI command against `server_url` (the `--server`
/// flag). The token file at `token_file` is read by every subcommand
/// except `NewGame`, which writes it.
pub fn dispatch(server_url: &str, token_file: &Path, command: Command) -> Result<(), String> {
    match command {
        Command::NewGame {
            seed,
            width,
            height,
            player,
            ai,
            game_file: _,
            victory: _,
        } => bootstrap::new_game(server_url, token_file, seed, width, height, &player, &ai),

        Command::EndTurn { .. } => end_turn::end_turn(server_url, token_file),
        Command::View { .. } => view::view(server_url, token_file),
        Command::Status { kind, .. } => status::status(server_url, token_file, &kind),
        Command::List { kind, .. } => list::list(server_url, token_file, &kind),
        Command::Action { action, .. } => action::action(server_url, token_file, &action),

        Command::Repl { .. } | Command::Play | Command::Demo | Command::AiDemo { .. } => {
            Err("server mode does not support interactive / demo subcommands".into())
        }
    }
}
