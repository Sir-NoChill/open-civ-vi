//! Pluggable mail-transport interface for the email magic-link flow.
//!
//! Phase 2.2 of `book/src/roadmap/accounts-and-login.md`. The default
//! [`LogMailer`] just writes the magic-link to stderr — invaluable in
//! dev / CI / self-host without an SMTP server. Production deploys
//! enable the `mailer-smtp` feature and use [`SmtpMailer`] (TODO; the
//! shape is reserved here so the trait surface is stable).

#![cfg(feature = "persistence")]

use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MailerError {
    #[error("transport error: {0}")]
    Transport(String),
    #[error("invalid recipient: {0}")]
    Recipient(String),
    #[error("backend not configured (build with --features mailer-smtp)")]
    NotConfigured,
}

/// Mail transport. The lobby owns one `Arc<dyn Mailer>` and hands it
/// to the email-auth handler.
#[async_trait]
pub trait Mailer: Send + Sync {
    /// Deliver a magic-link sign-in email to `email`. The `link` is the
    /// fully-rendered URL the user clicks (e.g.
    /// `https://lobby.example/api/v1/auth/email/verify?token=…`).
    async fn send_magic_link(&self, email: &str, link: &str) -> Result<(), MailerError>;

    /// Generic send. Reserved for invite emails / verification flows —
    /// default impl forwards to [`Mailer::send_magic_link`] when the
    /// subject contains "magic link", otherwise errors out so backends
    /// that don't support generic mail fail loudly.
    async fn send_raw(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), MailerError> {
        let _ = (to, subject, body);
        Err(MailerError::NotConfigured)
    }
}

// ───────────────────────────── LogMailer ─────────────────────────────────────

/// Default mailer for dev: writes the magic link to stderr in a format
/// that's grep-friendly and copy/pasteable.
pub struct LogMailer;

#[async_trait]
impl Mailer for LogMailer {
    async fn send_magic_link(&self, email: &str, link: &str) -> Result<(), MailerError> {
        eprintln!("[magic-link] to={email} link={link}");
        Ok(())
    }

    async fn send_raw(&self, to: &str, subject: &str, body: &str) -> Result<(), MailerError> {
        eprintln!("[mail] to={to} subject={subject:?}");
        for line in body.lines() {
            eprintln!("[mail]   {line}");
        }
        Ok(())
    }
}

// ───────────────────────────── SmtpMailer (stub) ──────────────────────────────

/// SMTP-backed mailer. Behind the `mailer-smtp` feature; today this is
/// a stub that always returns `MailerError::NotConfigured`. The real
/// `lettre`-backed implementation lands when self-host deploys need
/// it (Phase 6 of the roadmap).
#[cfg(feature = "mailer-smtp")]
pub struct SmtpMailer;

#[cfg(feature = "mailer-smtp")]
#[async_trait]
impl Mailer for SmtpMailer {
    async fn send_magic_link(&self, _email: &str, _link: &str) -> Result<(), MailerError> {
        Err(MailerError::NotConfigured)
    }
}

// ───────────────────────────── Tests ──────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn log_mailer_send_magic_link_does_not_error() {
        let m = LogMailer;
        m.send_magic_link("alice@example.com", "https://lobby.test/verify?t=abc")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn log_mailer_send_raw_does_not_error() {
        let m = LogMailer;
        m.send_raw("alice@example.com", "Welcome", "Hi!\nPlease sign in.")
            .await
            .unwrap();
    }
}
