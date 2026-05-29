# Rusty Auth

[![CI](https://github.com/Okpainmo/rusty-auth/actions/workflows/ci.yml/badge.svg)](https://github.com/Okpainmo/rusty-auth/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Conventional Commits](https://img.shields.io/badge/Conventional%20Commits-1.0.0-yellow.svg)](https://conventionalcommits.org)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](CODE_OF_CONDUCT.md)

A ready-made rust auth service built for direct integration into any microservice project.

## Core Features

- Complete JWT + Cookie Authentication implementation.

- Argon2 password Hashing.

- PostgreSQL + SQLX for database and database operations respectively.

- Dynamic Multi-layer configuration system (TOML + Environment Variables).

- Robust Error Handling and clear API responses.

- Comprehensive Testing with unit tests for utilities and integration tests for API endpoints.

- Middleware Integration for logging, request timeouts, and Cookie management.

## Setup & Execution

### 1. Core Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)(latest stable)

- [Docker](https://www.docker.com/)

- [sqlx-cli](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli)(`cargo install sqlx-cli`)

- [Node.js](https://nodejs.org/en/download/)(and [Bun](https://bun.sh/)) - for contribution
  standards enforcement)

### 2. Integrating into a microservice project

1. Ensure to be on the preferred directory of your microservice project. E.g. `services`.

```bash
cd <microservice-services-dir>
```

2. Clone in the auth service.

```bash
git clone --single-branch --branch main https://github.com/Okpainmo/rusty-auth <preferred-auth-service-name>
```

3. "ungit" the directory(should no longer be a repository)

```bash
cd <preferred-auth-service-name>
rm -rf .git
```

4. Remove all repo-specific files/folders.

```bash
rm -rf .github .husky .codex .vscode CHANGELOG.md CODE_OF_CONDUCT.md commitlint.config.mjs CONTRIBUTING.md commitlint.config.mjs LICENSE SECURITY.md &&  bun pm pkg delete scripts.prepare
```

### 3. Database Setup

Start the local PostgreSQL database:

```shell
docker run -d --name <container-name> -p 5433:5432 -e POSTGRES_USER=<user-name> -e POSTGRES_PASSWORD=<password> -e POSTGRES_DB=<database-name> postgres
```

E.g.

```shell
docker run -d --name rusty-auth-dev-db -p 5433:5432 -e POSTGRES_USER=okpainmo -e POSTGRES_PASSWORD=supersecret -e POSTGRES_DB=rusty-auth-dev-db postgres
```

Initialize the schema to sync with your newly set up database:

```shell
sqlx migrate run --database-url postgres://<user-name>:<password>@localhost:5433/<database-name>
```

E.g.

```shell
sqlx migrate run --database-url postgres://okpainmo:supersecret@localhost:5433/rusty-auth-dev-db
```

#### If Contributing

1. Initialize the migration - to add/register a new migration file.

```shell
sqlx migrate add <migration_name>
```

E.g.

```shell
sqlx migrate add added_new_hello_field_to_users_table
```

2. Edit the migration file to add the SQL schema update.

3. Sync the database with the new migration.

```shell
sqlx migrate run --database-url postgres://<user-name>:<password>@localhost:5433/<database-name>
```

E.g.

```shell
sqlx migrate run --database-url postgres://okpainmo:supersecret@localhost:5433/rusty-auth-dev-db
```

### 4. Running the Server

**Ensure to have installed `cargo-watch`.**

```shell
cargo install cargo-watch
```

To start the server in development mode(auto-reload enabled), simply run:

```shell
cargo dev
```

> `cargo-watch` handles the server/project reloads on-save. See `.config/config.toml` for reference
> on the `dev` command.

**Note:** The `dev` command is an alias for `cargo watch`. If you are on WSL and reload doesn't
trigger, proceed to use the polling command option(also see `.cargo/config.toml` for reference on
that).\*\*

### 5. Setting up to ensure contribution standards

Based on previous research, the Rust ecosystem lacks a very robust/mature and fully integrated
solution for enforcing code contribution standards, such as conventional commits, pre-commit hooks,
and automated linting checks.

By leveraging non-rust packages like `Husky` and `Commitlint` from `Node.js`, the project gains a
comprehensive cross-language workflow that ensures:

- Standardized commit messages across all contributors

- Automatic pre-commit checks (formatting, linting, compilation)

- Pre-push checks enforcement

> The project however stays focused - with its core as pure Rust. The `Node.js` integration only
> introduced the packages needed for enforcing code/contribution standards on the project.
>
> P.S: The preferred `Node.js` package manager is `Bun`.

To integrate the `Husky` and `Commitlint` setup into your current Rust workflow:

1. Ensure to sync with the main branch and pull in all updates:

```shell
git pull origin main
```

2. Install the new packages with Bun:

```shell
bun install
```

## Project Config Setup

The project uses a highly flexible configuration pattern powered by the `config` crate.

### Loading Order(Arranged in increasing order of overriding authority):

1. **Base Config**: `config/base.toml` (Default values).

2. **Environment Config Overrides**: `config/{APP__ENV}.toml` (e.g., `development.toml`,
   `production.toml`).

3. **Local Overrides**: `config/local.toml`

4. **Environment Variables**: Prefixed with `APP__`.

### Mapping Rule for Environment Variables

`__`(double underscore) is used as a separator to map to nested TOML sections.

**Syntax:** `APP__<SECTION>__<FIELD>=value`

**Example:**

To override the server port in `base.toml`:

```toml
[server]
port = 8000
```

Set the environment variable:

`APP__SERVER__PORT=9000`

### Mandatory Sections

The `validate()` method ensures the following sections are correctly populated at startup:

- `app`: Basic metadata.

- `server`: Host, Port, and Request Timeouts.

- `database`: Engine, Connection Pool settings, and Auth.

- `auth`: JWT Secret and Expiration lifetimes.

## Environment Variables Files

The project uses several `.env` files to manage environment-specific configurations. To assist in
setting up your local environment, we provide several **`.sample`** versions within the project
root.

### Setting up your environment

To get started, you should remove the **`.sample`** extension from the end of the sample files to
create the active configuration files.

E.g.

- Duplicate `.env.sample` and rename it to `.env`.
- Duplicate `.env.development.sample` and rename it to `.env.development`.

### Available Files:

- `.env`: The default environment file. Its only current function is controlling environment
  selection for the project.

- `.env.development`: Contains configuration overrides specifically for local development.

- `.env.staging`: Configured for the staging/pre-production environment.

- `.env.production`: Contains sensitive production-only credentials and settings.

> **Note:** Retrieve real secrets through the approved secret-management process and never commit
> real `.env` files to source control.

## Testing

The project maintains high reliability through two layers of testing.

### 1. Unit Tests

Located within the source files (e.g., `src/utils/generate_tokens.rs`). They test isolated logic
like hashing, token generation, and config validation.

**Run unit tests:**

```shell
cargo test --lib
```

### 2. Integration Tests

Located in the `tests` directory. They spin up a real server instance and a test database to verify
end-to-end API flows.

**Available Integration Tests:**

- `login_test.rs`: Successful login, invalid credentials, non-existent users.

- `register_test.rs`: New user creation, duplicate email/phone prevention.

- `logout_test.rs`: Token invalidation and cookie clearing.

**Run integration tests:**

```shell
cargo test --test '*'
```

### 3. How to add new tests

- **Unit Tests**: Add a `#[cfg(test)]` block and the required test(s) - all contained within a
  `mod tests` block - at the end of your module.

E.g.

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_something() {
        assert!(true);
    }
}
```

- **Integration Tests**:
  1. Create a new file in the `tests` directory contained on the root of the project.

  2. Use `common::setup_test_server().await` to get a `TestServer` instance.

  3. Use the request/response structs defined in `tests/common/mod.rs` for type-safe interaction.

## Reliability & Robustness

- **Request Timeouts**: Configurable via `server.request_timeout_secs`.

- **Database Pooling**: Managed via `PgPoolOptions` with configurable `max_connections`.

- **Environment-Aware Cookies**: Cookies are automatically set to `Secure` in production and
  `Lax`/`Insecure` (for HTTP) in development.

## Logging Implementation Layers

The application implements logging through multiple layers to ensure full visibility:

1. **Central Logging Middleware**: A top-level middleware that captures every incoming request and
   outgoing response, logging metadata such as HTTP method, path, status codes, and processing time.

> This prevents the need to manually register logs if no errors/issues are encountered on a request.

2. **In-Process Logic Logs**: Granular logs emitted directly from within the application logic
   (e.g., during errors) to capture specific runtime context.

## Operating System Notes(WSL)

If you are developing on **WSL**, file system events might not trigger `cargo watch`. The project's
`cargo dev` alias is pre-configured to use `--poll` if needed.

## Contributing

Contributions are what make the open-source community such an amazing place to learn, inspire, and
create. Any contributions you make are **greatly appreciated**.

Please check our [Contributing Guidelines](CONTRIBUTING.md) to get started.

### Code of Conduct

We are committed to providing a friendly, safe and welcoming environment for all. Please see our
[Code of Conduct](CODE_OF_CONDUCT.md).

## Security

If you discover any security-related issues, please refer to our [Security Policy](SECURITY.md)
instead of using the issue tracker.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

Cheers!!! 🍻
