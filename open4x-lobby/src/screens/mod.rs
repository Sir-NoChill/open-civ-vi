//! Top-level screens. Each screen mirrors a JSX file under
//! `docs/open4x-landing/project/hifi/`.

pub mod landing;
pub mod login;
pub mod menu;
pub mod newgame;
pub mod ongoing;
pub mod profile;

pub use landing::Landing;
pub use login::Login;
pub use menu::{MenuShell, MenuTab};
pub use newgame::NewGame;
pub use ongoing::OngoingGames;
pub use profile::Profile;
