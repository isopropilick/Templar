# Contributing to Templar

Thanks for your interest in contributing! 🎉
This project is a Rust-based email microservice using Axum, Lettre, and Handlebars. We welcome bug reports, feature requests, and pull requests.

---

## Code of Conduct

By participating, you agree to uphold a respectful, inclusive, and collaborative environment. Please be kind and constructive in all interactions.

---

## Getting Started

### Prerequisites

* [Rust](https://www.rust-lang.org/tools/install) (latest stable recommended)
* [Cargo](https://doc.rust-lang.org/cargo/) (comes with Rust)
* An SMTP account (Mailgun, Postmark, SES, Gmail SMTP, etc.)
* Docker (optional, for container builds)

### Setup

1. Fork and clone the repository:

   ```bash
   git clone https://github.com/<your-username>/Templar.git
   cd Templar
   ```
2. Copy the environment template: (TO-DO)

   ```bash
   cp .env.example .env
   ```

   Update with your own values.
3. Run the service:

   ```bash
   cargo run
   ```
4. Test the `/send` endpoint with curl or Postman.

---

## Development Workflow

### Branching

* **main** → stable, production-ready.
* **feature/*** → new features.
* **fix/*** → bug fixes.
* **docs/*** → documentation updates.

### Commit Messages

Use clear, descriptive commits:

```
feat: add support for SMTPS (465)
fix: handle missing template error more gracefully
docs: update README with Bearer auth example
```

### Pull Requests

1. Sync with `main` before opening a PR.
2. Ensure code is formatted and linted:

   ```bash
   cargo fmt --all
   cargo clippy --all-targets --all-features -- -D warnings
   ```
3. Run tests: (TO-DO)

   ```bash
   cargo test
   ```
4. Open a PR with a clear description of what you changed and why.

---

## Testing (TO-DO)

* Unit tests live next to modules (`mod tests`).
* To run all tests:

  ```bash
  cargo test --all
  ```
* If adding features, write tests covering:

  * Template rendering (missing vars, strict mode).
  * Recipient parsing.
  * HTML → plaintext stripping.

---

## Style Guide

* Use **Rust 2021/2024 edition** features where possible.
* Keep functions small and focused.
* Document modules and public functions with `///`.
* Prefer `anyhow::Error` at edges, `thiserror::Error` for domain errors.
* Handle configuration via environment variables.

---

## Documentation

* Update `README.md` when adding features or changing behavior.
* Add examples for new templates or config options.
* Include `curl` examples in API docs when relevant.

---

## Docker & Deployment (TO-DO)

* Build locally:

  ```bash
  docker build -t templar .
  docker run -p 3000:3000 --env-file .env templar
  ```
* If adding deployment scripts (e.g., Kubernetes manifests), place them under `/deploy`.

---

## Security (TO-DO)

* Do not commit `.env` files or secrets.
* Use `API_KEY` + Bearer authentication for production.
* When in doubt, open an issue and discuss before submitting security-sensitive changes.

---

## Questions?

* Open a [GitHub Issue](../../issues) with your question.
* Or start a [Discussion](../../discussions) if enabled.
