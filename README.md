# Templar

A minimal Rust microservice to send transactional emails from JSON payloads using **SMTP + Handlebars templates**.

* **Axum** web server (async, Tokio)
* **Lettre** SMTP client (TLS)
* **Handlebars** templates with base/partial support
* **HTML + auto–plaintext** multipart emails
* **Structured logs** via `tracing` (console + file)
* Simple `.env` configuration

> ⚠️ Security: This repo ships as a thin demo service. Protect it behind an API gateway / firewall and add authentication before internet exposure.

---

## Quick start

### Prerequisites

* Rust (stable) & Cargo
* An SMTP account (e.g., Mailgun / Postmark / SES SMTP)
* Linux/macOS/Windows

### Configure environment

Copy `.env` (already included) and set your values:

```env
# HTTP
API_KEY=dev-secret-token
API_KEY_CURRENT_REQUEST=dev-secret-token
LISTEN_ADDR=127.0.0.1
LISTEN_PORT=3000

# Mail (SMTP)
SMTP_HOST=smtp.mailgun.org
SMTP_PORT=587
SMTP_USERNAME=notifications@domain.com
SMTP_PASSWORD=secret-password
MAIL_FROM=notifications@domain.com
MAIL_REPLY_TO=notifications@domain.com

# Templates directory (relative to project root or absolute)
TEMPLATES_DIR=src/templates
```

### Run

```bash
cargo run
```

You should see a log line like:

```
Starting server on 127.0.0.1:3000
```

---

## API

### `POST /send`

Sends an email rendered from a Handlebars template.

**Request body (JSON)**

```json
{
  "to": "alice@example.com,bob@example.com",
  "subject": "Welcome to the service",
  "template": "welcome",
  "vars": {
    "name": "Alice",
    "product": "Awesome SAAS service",
    "verify_url": "https://example.com/verify?token=..."
  }
}
```

* `to`: a single email or **comma-separated** list
* `subject`: subject line
* `template`: template file **without** extension (e.g., `welcome` → `templates/welcome.hbs`)
* `vars`: key/value map injected into the Handlebars template

**Responses**

* `200 OK` → `{"status":"ok","id":"<smtp-message-id>"}`
* `404 Not Found` if the template doesn’t exist
* `422 Unprocessable Entity` if rendering fails
* `500 Internal Server Error` for other failures

**Example**

```bash
curl -X POST http://127.0.0.1:3000/send \
  -H "Content-Type: application/json" \
  -d '{
    "to": "alice@example.com",
    "subject": "Welcome to our product",
    "template": "welcome",
    "vars": {
      "name": "Alice",
      "product": "Awesome SAAS service",
      "verify_url": "https://example.com/verify?token=abc"
    }
  }'
```

---

## Templates

Default directory: `templates` (configurable via `TEMPLATES_DIR`).

Included demo templates:

* `base.hbs` — a simple layout partial
* `welcome.hbs` — demonstrates a block partial using `base`

`welcome.hbs`:

```hbs
{{#> base title="Welcome!"}}
  <h1 style="margin-top:0">¡Hi, {{name}}!</h1>
  <p>Thank you for your trust in <strong>{{product}}</strong>.</p>
  {{#if verify_url}}
    <p>
      <a class="btn" style="background:#2563eb;color:#fff" href="{{verify_url}}">
        Confirm email
      </a>
    </p>
  {{/if}}
{{/base}}
```

`base.hbs`:

```hbs
<!doctype html>
<html>
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>{{title}}</title>
  <style>
    body{margin:0;font-family:Arial,Helvetica,sans-serif;background:#f6f7fb;color:#222}
    .container{max-width:640px;margin:0 auto;padding:24px}
    .card{background:#fff;border-radius:12px;padding:24px;box-shadow:0 2px 8px rgba(0,0,0,.06)}
    .btn{display:inline-block;padding:12px 16px;border-radius:8px;text-decoration:none}
  </style>
</head>
<body>
  <div class="container">
    <div class="card">
      {{> @partial-block }}
      <hr style="border:none;border-top:1px solid #eee;margin:24px 0"/>
      <p style="font-size:12px;color:#666">
        Please, do not reply directly to this email.
      </p>
    </div>
  </div>
</body>
</html>
```

> The service builds a **multipart/alternative** message with the HTML you render and an auto-generated plaintext part (basic tag stripping + entity decoding).

---

## How it works

* `axum` hosts `/send` with JSON input (`routes.rs`)
* `EmailState` is created from environment (`email.rs`)
* `handlebars` registry is initialized on boot and templates/partials are registered
* On each request:

  1. recipients are parsed (`to` → `Mailbox` list)
  2. template is rendered with `vars`
  3. plaintext is derived from HTML
  4. an SMTP message is built & sent via `lettre`

---

## Configuration reference

Environment variables:

| Name          | Required | Default         | Description                          |
| ------------- | -------- | --------------- | ------------------------------------ |
| LISTEN_ADDR   | ✅        | —               | e.g., `0.0.0.0`                      |
| LISTEN_PORT   | ✅        | —               | e.g., `3000`                         |
| SMTP_HOST     | ✅        | —               | SMTP server hostname                 |
| SMTP_PORT     | ❌        | `587`           | SMTP port                            |
| SMTP_USERNAME | ✅        | —               | SMTP username                        |
| SMTP_PASSWORD | ✅        | —               | SMTP password                        |
| MAIL_FROM     | ✅        | —               | RFC-5322 address for the From header |
| MAIL_REPLY_TO | ❌        | —               | Optional Reply-To address            |
| TEMPLATES_DIR | ❌        | `src/templates` | Directory containing `.hbs` files    |

---

## Logging

`tracing` is configured for structured logs with levels. You can tune via standard `RUST_LOG` env or by modifying the subscriber in `main.rs`.

---

## Deployment notes

* Run behind a reverse proxy (NGINX, Caddy, Traefik)
* Add **authentication** (API key, mTLS, or JWT) and **rate limits**
* Keep SMTP credentials secret (env, Vault, or container secrets)
* Monitor delivery via your SMTP provider logs & webhooks (if applicable)

---

## Troubleshooting

* **Template not found (404):** ensure `TEMPLATES_DIR` points to the folder and `template` matches a file without the `.hbs` suffix.
* **Render error (422):** verify all placeholders used in the template exist in `vars`.
* **SMTP failures:** double-check host, port, username, password, and from/reply-to addresses.

---

## Tech stack

* Rust 2024 edition
* axum, tokio
* lettre (smtp-transport, native-tls)
* handlebars
* tracing / tracing-subscriber
* dotenvy

---

## License

MIT

---
