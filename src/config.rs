//! Configuration module for the email sending API.

/// Struct containing all configuration options.
#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub log_to_file: bool,
    pub log_to_stdout: bool,
    pub log_level: String,
    pub log_dir: String,
    pub log_file: String,
    pub templates_dir: String,
    pub outbox_dir: String,
    pub listen_addr: String,
    pub listen_port: u16,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub mail_from: String,
    pub mail_reply_to: String,
    pub transport: String,
}
/// # get_defaults()
/// Returns an `ApiConfig` struct populated with default values for all configuration options.
/// These defaults can be overridden by environment variables in the main application.
/// > Env vars are case-insensitive.
/// # Environment Variables:
/// |Variable|Description|
/// |:------:|:---------:|
/// |`LOG_LEVEL`|Log level (DEBUG, INFO, WARN, ERROR)|
/// |`LOG_TO_FILE`|Whether to log to file (true/false)|
/// |`LOG_TO_STDOUT`|Whether to log to stdout (true/false)|
/// |`LOG_DIR`|Directory to log to (relative to executable)|
/// |`LOG_FILE`|File to log to (relative to `LOG_DIR`)|
/// |`LISTEN_ADDR`|Address to bind to (e.g. `127.0.0.1`)|
/// |`LISTEN_PORT`|Port to bind to (e.g. `8080`)|
/// |`TEMPLATES_DIR`|Directory containing email templates|
/// |`SMTP_HOST`|SMTP server hostname (e.g. `smtp.example.com`)|
/// |`SMTP_PORT`|SMTP server port (e.g. `587`)|
/// |`SMTP_USERNAME`|SMTP username for authentication|
/// |`SMTP_PASSWORD`|SMTP password for authentication|
/// |`MAIL_FROM`|Default "from" email address (e.g. `test@localhost.com`)|
/// |`MAIL_REPLY_TO`|Default "reply-to" email address (e.g. `test@localhost.com`)|
/// |`TRANSPORT`|Email transport method (`smtp` or `file`)|
/// |`OUTBOX_DIR`|Directory to store emails when using `file` transport|
///
/// --------------------------------------------------------------------
/// ## Log defaults:
/// |`log_file`|`log_dir` |`log_to_file`|`log_to_stdout`|`log_level`|
/// |:--------:|:--------:|:-----------:|:-------------:|:---------:|
/// |`out.log` |`logs`    |`true`       |`true`         |`DEBUG`    |
/// --------------------------------------------------------------------
/// ## App defaults:
/// | `templates_dir` | `listen_addr`|`listen_port`|
/// |:---------------:|:------------:|:-----------:|
/// | `src/templates` |`127.0.0.1`   |`8080`       |
/// --------------------------------------------------------------------
/// ## SMTP defaults:
/// | `smtp_host`| `smtp_port`| `smtp_username`| `smtp_password`|
/// |:----------:|:----------:|:--------------:|:--------------:|
/// | `localhost`|`587`       |`user`          |`password`      |
/// --------------------------------------------------------------------
/// ## Mail defaults:
/// |         `mail_from`|     `mail_reply_to`|`transport`|`outbox_dir`|
/// |:------------------:|:------------------:|:---------:|:----------:|
/// |`test@localhost.com`|`test@localhost.com`|     `file`|    `outbox`|
/// --------------------------------------------------------------------
pub fn get_defaults() -> ApiConfig {
    let config = ApiConfig{
        log_file: "out.log".parse().unwrap(),
        log_dir: "logs".parse().unwrap(),
        log_to_file: true,
        log_to_stdout: true,
        templates_dir: "src/templates".parse().unwrap(),
        outbox_dir: "outbox".parse().unwrap(),
        listen_addr: "127.0.0.1".parse().unwrap(),
        listen_port: 8080,
        smtp_host: "localhost".parse().unwrap(),
        smtp_port: 587,
        smtp_username: "user".parse().unwrap(),
        smtp_password: "password".parse().unwrap(),
        mail_from: "test@localhost.com".parse().unwrap(),
        mail_reply_to: "test@localhost.com".parse().unwrap(),
        transport: "file".parse().unwrap(),
        log_level: "DEBUG".parse().unwrap()
    };
    config
}