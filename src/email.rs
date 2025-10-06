//! Email state + rendering + sending
//! Minimal, documented version.

use std::{collections::HashMap, path::PathBuf, time::Duration};

use handlebars::Handlebars;
use lettre::{message::{header, Mailbox, MultiPart, SinglePart}, transport::file::AsyncFileTransport, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use once_cell::sync::OnceCell;
use serde_json::Value;
use thiserror::Error;

static REGISTRY: OnceCell<Handlebars<'static>> = OnceCell::new();

/// Transport selected at runtime (SMTP for prod, FILE for local dev).
#[derive(Clone)]
pub enum Mailer {
    Smtp(AsyncSmtpTransport<Tokio1Executor>),
    File(AsyncFileTransport<Tokio1Executor>),
}

impl Mailer {
    /// Unified `send` so callers don't care which transport we're using.
    /// We normalize errors to String to avoid mixing different transport error types.
    pub async fn send(&self, email: Message) -> Result<(), String> {
        match self {
            Mailer::Smtp(m) => m.send(email).await.map(|_| ()).map_err(|e| e.to_string()),
            Mailer::File(f) => f.send(email).await.map(|_| ()).map_err(|e| e.to_string()),
        }
    }
}

/// Domain errors we surface to the handler layer.
#[derive(Debug, Error)]
pub enum EmailError {
    #[error("template not found: {0}")]
    TemplateNotFound(String),
    #[error("render error: {0}")]
    RenderError(String),
    #[error("smtp error: {0}")]
    SmtpError(String),
    #[error("config error: {0}")]
    Config(String),
}

/// App-wide email state (transport + addressing + templates location).
#[derive(Clone)]
pub struct EmailState {
    pub mailer: Mailer,
    pub from: Mailbox,
    pub reply_to: Option<Mailbox>,
    pub templates_dir: PathBuf,
}

impl EmailState {
    /// Build state from environment variables and initialize the Handlebars registry.
    ///
    /// Required envs (for SMTP mode):
    /// - SMTP_HOST, SMTP_USERNAME, SMTP_PASSWORD, MAIL_FROM
    /// Optional:
    /// - SMTP_PORT (default 587), MAIL_REPLY_TO, TEMPLATES_DIR (default "src/templates")
    /// - MAIL_TRANSPORT = "smtp" (default) | "file"
    /// - MAIL_FILE_DIR (default "outbox/") — only used when MAIL_TRANSPORT=file
    pub fn from_env() -> Result<Self, anyhow::Error> {
        // Common addressing
        let from: Mailbox = std::env::var("MAIL_FROM")?
            .parse()
            .map_err(|e| anyhow::anyhow!(format!("Invalid MAIL_FROM: {e}")))?;
        let reply_to = std::env::var("MAIL_REPLY_TO").ok().and_then(|s| s.parse::<Mailbox>().ok());
        let templates_dir = PathBuf::from(std::env::var("TEMPLATES_DIR").unwrap_or_else(|_| "src/templates".into()));
        // Init HandleBars registry (strict mode, base.hbs partial, etc.)
        init_registry(&templates_dir)?;
        // Choose transport
        let transport = std::env::var("MAIL_TRANSPORT").unwrap_or_else(|_| "smtp".into()).to_lowercase();
        // Set defaults for SMTP
        let host = std::env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".into());
        let port = std::env::var("SMTP_PORT").unwrap_or_else(|_| "587".into()).parse::<u16>()?;
        let username = std::env::var("SMTP_USERNAME").unwrap_or_else(|_| "user".into());
        let password = std::env::var("SMTP_PASSWORD").unwrap_or_else(|_| "password".into());
        // Build transport
        let mailer;
        if transport == "file" {mailer = build_file_mailer()?;}
        else {mailer = build_smtp_mailer(&host, port, &username, &password)?;}
        Ok(Self {
            mailer,
            from,
            reply_to,
            templates_dir,
        })
    }
}

/// Build a STARTTLS SMTP transport with creds and short timeout.
fn build_smtp_mailer(
    host: &str,
    port: u16,
    user: &str,
    pass: &str,
) -> Result<Mailer, anyhow::Error> {
    use lettre::transport::smtp::authentication::Credentials;

    let creds = Credentials::new(user.to_string(), pass.to_string());
    Ok(Mailer::Smtp(
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(host)?
            .port(port)
            .credentials(creds)
            .timeout(Some(Duration::from_secs(15)))
            .build(),
    ))
}
/// Build a file transport (writes `.eml` files), used for local/dev.
fn build_file_mailer() -> Result<Mailer, anyhow::Error> {
    use std::fs;
    use std::path::Path;
    let dir = std::env::var("MAIL_FILE_DIR").unwrap_or_else(|_| "outbox".into());
    fs::create_dir_all(&dir)?;
    let root = Path::new(&dir).to_path_buf();
    Ok(Mailer::File(AsyncFileTransport::new(root)))
}
/// Initialize a global Handlebars registry in strict mode.
/// We pre-register the `base` layout as a **partial** (used by `{{#> base}} ... {{/base}}`).
fn init_registry(dir: &std::path::Path) -> Result<(), anyhow::Error> {
    let mut reg = Handlebars::new();
    reg.set_strict_mode(true);

    let base = dir.join("base.hbs");
    if base.exists() {
        let base_src = std::fs::read_to_string(&base)?;
        reg.register_partial("base", base_src)?;
    }

    let _ = REGISTRY.set(reg); // ignore if already set (idempotent on boot)
    Ok(())
}

/// Render the requested template with `vars`, build a multipart (text+html) message,
/// and send it via SMTP. Returns a pseudo message ID (random nanoid).
pub async fn render_and_send(
    state: &EmailState,
    req: crate::routes::SendRequest,
) -> Result<String, EmailError> {
    // 1) Recipients
    let to_list = parse_recipients(&req.to)
        .map_err(|e| EmailError::Config(format!("invalid recipient: {e}")))?;

    // 2) HTML from Handlebars (strict mode guards missing vars)
    let html = render_template(&state.templates_dir, &req.template, &req.vars)?;

    // 3) Build the email with multipart/alternative (plaintext + html)
    let mut builder = Message::builder().from(state.from.clone()).subject(req.subject);
    if let Some(rt) = &state.reply_to {
        builder = builder.reply_to(rt.clone());
    }
    for mb in to_list {
        builder = builder.to(mb);
    }

    let email = builder
        // `MultiPart::alternative` sets the correct `Content-Type`; no manual header needed.
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_PLAIN)
                        .body(strip_html::strip(&html)),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(html),
                ),
        )
        .map_err(|e| EmailError::Config(format!("message build error: {e}")))?;

    // 4) Send (or write to file, depending on transport)
    state.mailer.send(email).await.map_err(|e| EmailError::SmtpError(e.to_string()))?;
    Ok(nanoid())
}

/// Parse comma-separated recipients into `Mailbox`es.
fn parse_recipients(to: &str) -> Result<Vec<Mailbox>, lettre::address::AddressError> {
    to.split(',').map(|s| s.trim().parse()).collect()
}

/// Generate a compact pseudo message id (22 chars, URL-safe).
fn nanoid() -> String {
    use rand::{distr::Alphanumeric, rng, Rng};
    rng()
        .sample_iter(&Alphanumeric)
        .take(22)
        .map(char::from)
        .collect()
}

/// Load a `.hbs` file and render with the global registry (which already has `base` partial).
fn render_template(
    dir: &std::path::Path,
    name: &str,
    vars: &HashMap<String, Value>,
) -> Result<String, EmailError> {
    let reg = REGISTRY.get().expect("registry not initialized");

    let path = dir.join(format!("{name}.hbs"));
    if !path.exists() {
        return Err(EmailError::TemplateNotFound(name.to_string()));
    }

    let tpl_src =
        std::fs::read_to_string(&path).map_err(|e| EmailError::RenderError(e.to_string()))?;

    // Using `render_template` renders a raw string (not a named template).
    // This works with our pre-registered `base` partial for `{{#> base}}...{{/base}}`.
    reg.render_template(&tpl_src, vars)
        .map_err(|e| EmailError::RenderError(e.to_string()))
}

/// Tiny best-effort HTML→plaintext stripper for the text alternative.
mod strip_html {
    pub fn strip(html: &str) -> String {
        let mut out = String::with_capacity(html.len());
        let mut in_tag = false;
        for c in html.chars() {
            match c {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => out.push(c),
                _ => {}
            }
        }
        out.replace("&nbsp;", " ")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
    }
}
