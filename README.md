# rust-mail-composing-api
production-leaning email microservice written in Rust. It exposes a single HTTP endpoint that renders Handlebars templates and sends transactional emails via SMTP. It uses Axum, Lettre, and Handlebars, with structured logging via tracing. Templates live on disk (hot-reloaded on each render), and HTML is auto-converted to a plaintext alternative.
