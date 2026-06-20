# Rusty Auth

[![CI](https://github.com/Okpainmo/rusty-auth/actions/workflows/ci.yml/badge.svg)](https://github.com/Okpainmo/rusty-auth/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![Conventional Commits](https://img.shields.io/badge/Conventional%20Commits-1.0.0-yellow.svg)](https://conventionalcommits.org)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](CODE_OF_CONDUCT.md)

`Rusty Auth` is an auth microservice built with Rust, the Axum framework, Tokio, PostgreSQL, and a
lot more.

Built with much love for projects I lead/work on, it is designed to be dropped into a microservice
system as a ready auth boundary, while still being small enough to understand, customize, and
extend. Out of the box it handles user/admin registration, login, logout, session persistence, token
rotation, role/permission management, sub-session audit logs, environment-aware configuration, and
integration tests against real HTTP flows.

## Table Of Contents

- [Static Analysis/Code Standardization](#static-analysiscode-standardization)
- [Why Rusty Auth](#why-rusty-auth)
- [Core Capabilities](#core-capabilities)
- [How Authentication Works](#how-authentication-works)
- [API Reference](#api-reference)
  - [Public Routes](#public-routes)
  - [Protected Routes](#protected-routes)
  - [Request Examples](#request-examples)
- [Project Architecture](#project-architecture)
- [Requirements](#requirements)
- [Setup & Execution](#setup--execution)
  - [Core Prerequisites](#1-core-prerequisites)
  - [Clone The Project](#2-clone-the-project)
  - [Integrating Into A Microservice Project](#3-integrating-into-a-microservice-project)
  - [Environment Files](#4-environment-files)
  - [Database Setup](#5-database-setup)
  - [Running The Server](#6-running-the-server)
  - [Docker And Compose](#7-docker-and-compose)
  - [Creating New Migrations](#8-creating-new-migrations)
  - [Development Checks](#9-development-checks)
- [Postman Collection](#postman-collection)
- [Traffic Simulation](#traffic-simulation)
- [Configuration](#configuration)
- [Rate Limiting](#rate-limiting)
- [Security Model](#security-model)
- [Testing](#testing)
- [Operational Notes](#operational-notes)
- [Contribution Workflow](#contribution-workflow)
- [License](#license)

## Static Analysis/Code Standardization

Rusty Auth is purely a Rust service at its core, but the root build uses JavaScript ecosystem
tooling for repository workflow and code-standardization checks. The Node/Bun layer is intentionally
limited to developer tooling; the service runtime remains Rust-based.

- `package.json` defines the Bun-powered workflow scripts.
- Husky installs Git hooks through the `prepare` script.
- Commitlint enforces Conventional Commit messages in `.husky/commit-msg`.
- Pre-commit checks run `cargo check`, `cargo fmt -- --check`, Clippy with warnings denied, and
  Markdown formatting.
- Pre-push checks run unit tests, conditionally run DB-backed controller tests when PostgreSQL is
  reachable, and verify Markdown formatting.
- Prettier is used for Markdown formatting and documentation consistency.

Install the tooling with:

```bash
bun install
```

## Why Rusty Auth

Rusty Auth exists for teams and builders who want a practical auth service without starting from a
blank Axum project every time. It favors explicit database-backed flows, clear JSON responses, and a
middleware stack that can be reasoned about from request entry to protected route execution.

The project is especially useful when you need:

- A standalone auth service inside a broader microservice architecture.

- JWT access tokens plus refresh/session tokens backed by PostgreSQL.

- HTTP-only auth cookies with development/production-aware security settings.

- Session renewal, revocation, expiry tracking, and activity history.

- Role and permission primitives that can grow into a richer authorization layer.

- A Rust-first service with modern contribution tooling around formatting, commits, and CI.

## Core Capabilities

### Authentication

- User registration through `POST /api/v1/auth/register`.

- Admin registration through `POST /api/v1/auth/register/admin`.

- Login through `POST /api/v1/auth/login`.

- Logout through `POST /api/v1/auth/logout`.

- Argon2 password hashing and verification.

- JWT access token generation.

- JWT refresh/session token generation.

- One-time-password token generation utility support.

- HTTP-only `auth_cookie` deployment.

- Token-kind claims to distinguish access, refresh, and one-time-password tokens.

### Sessions

- PostgreSQL-backed session records.

- Refresh/session token hashes stored in the database.

- Session status tracking, including active and revoked states.

- Session expiry management.

- Session renewal inside protected-route middleware.

- Session listing by user or globally.

- Session lookup by ID.

- Session expiry updates.

- Logout-driven session revocation.

### Audit Logs And Sub-Sessions

- Granular sub-session records for important auth and management actions.

- Activity type and activity description tracking.

- Request method and request path tracking.

- IP address and User-Agent extraction through request metadata.

- Ordered sub-session history per session.

### Roles And Permissions

- Role creation, update, and listing.

- Permission creation, update, and listing.

- User-role assignment and removal.

- Role-permission assignment and deletion.

- User role listing.

- User permission listing.

- Automatic role assignment during registration based on user type, such as `user` or `admin`.

### Middleware

- Logging middleware for request timing and request metadata capture.

- Request timeout middleware.

- Fixed-window IP-based rate limiting middleware.

- Session middleware for protected route validation and token renewal.

- Access middleware for access token validation.

- Cookie manager integration through `tower-cookies`.

### Configuration And Reliability

- TOML configuration layered by environment.

- Environment variable overrides with `APP__` prefixes.

- Startup config validation.

- PostgreSQL connection pooling.

- JSON tracing subscriber setup.

- Unit tests for isolated utilities.

- Integration tests for auth, session, role, and permission API flows.

## How Authentication Works

The public auth flow starts with registration or login. When the credentials are accepted, Rusty
Auth:

1. Creates or resolves the user.

2. Hashes sensitive token material before persistence.

3. Creates a database session.

4. Creates a sub-session audit entry for the activity.

5. Generates an access token and refresh/session token.

6. Deploys an HTTP-only `auth_cookie`.

7. Returns the user profile, `session_id`, access token, and refresh token in the response.

Protected routes pass through the session and access middlewares. A protected request is expected to
include:

```http
Authorization: Bearer <access_token>
user_id: <user_id>
session_token: <refresh_token>
session_id: <session_id>
Cookie: auth_cookie=<auth_cookie_value>
```

The session middleware verifies the user, cookie, session ID, refresh/session token, stored
refresh-token hash, session status, and token claims. When the session is valid, it renews the
session and issues fresh tokens for the current protected-route response.

The access middleware then verifies that the access token belongs to the resolved user and has the
expected access-token kind.

Because protected routes rotate the session token, clients should treat `access_token`,
`refresh_token`, and `session_id` from successful protected responses as the next credentials to
use. Reusing an older refresh/session token after a protected response has rotated it will be
rejected.

## API Reference

All routes are nested under:

```text
/api/v1/auth
```

### Public Routes

| Method | Path              | Purpose                       |
| ------ | ----------------- | ----------------------------- |
| POST   | `/register`       | Register a standard user      |
| POST   | `/register/admin` | Register an admin user        |
| POST   | `/login`          | Authenticate an existing user |

### Protected Routes

Protected routes require the auth headers and cookie described in
[How Authentication Works](#how-authentication-works).

| Method | Path                           | Purpose                                         |
| ------ | ------------------------------ | ----------------------------------------------- |
| POST   | `/logout`                      | Revoke the current session and clear auth state |
| GET    | `/sessions`                    | List sessions                                   |
| GET    | `/sessions/user/{user_id}`     | List sessions for a user                        |
| GET    | `/sessions/{session_id}`       | Get a session with sub-session history          |
| PATCH  | `/sessions/{session_id}`       | Update a session expiry                         |
| GET    | `/roles`                       | List roles                                      |
| POST   | `/roles`                       | Create a role                                   |
| PATCH  | `/roles/{role_id}`             | Update a role                                   |
| POST   | `/roles/permissions`           | Assign a permission to a role                   |
| DELETE | `/roles/permissions`           | Remove a permission from a role                 |
| POST   | `/roles/user/assign`           | Assign a role to a user                         |
| POST   | `/roles/user/remove`           | Remove a role from a user                       |
| GET    | `/roles/user/{user_id}`        | List roles for a user                           |
| GET    | `/permissions`                 | List permissions                                |
| POST   | `/permissions`                 | Create a permission                             |
| PATCH  | `/permissions/{permission_id}` | Update a permission                             |
| GET    | `/permissions/user/{user_id}`  | List effective permissions for a user           |

### Request Examples

Register a user:

```bash
curl -X POST http://127.0.0.1:8000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "first_name": "Ada",
    "last_name": "Lovelace",
    "email": "ada@example.com",
    "password": "password123",
    "country": "United Kingdom",
    "country_code": "GB",
    "phone_number": "1234567890"
  }'
```

Login:

```bash
curl -X POST http://127.0.0.1:8000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "ada@example.com",
    "password": "password123"
  }'
```

Call a protected route:

```bash
curl http://127.0.0.1:8000/api/v1/auth/roles \
  -H "Authorization: Bearer <access-token>" \
  -H "user_id: <user-id>" \
  -H "session_token: <refresh-token>" \
  -H "session_id: <session-id>" \
  -H "Cookie: auth_cookie=<auth-cookie-value>"
```

Create a role:

```bash
curl -X POST http://127.0.0.1:8000/api/v1/auth/roles \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <access-token>" \
  -H "user_id: <user-id>" \
  -H "session_token: <refresh-token>" \
  -H "session_id: <session-id>" \
  -H "Cookie: auth_cookie=<auth-cookie-value>" \
  -d '{
    "name": "moderator",
    "description": "Can review and moderate user content"
  }'
```

Assign a permission to a role:

```bash
curl -X POST http://127.0.0.1:8000/api/v1/auth/roles/permissions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <access-token>" \
  -H "user_id: <user-id>" \
  -H "session_token: <refresh-token>" \
  -H "session_id: <session-id>" \
  -H "Cookie: auth_cookie=<auth-cookie-value>" \
  -d '{
    "role_id": "<role-id>",
    "permission_id": "<permission-id>"
  }'
```

## Project Architecture

```text
src/
  core/
    controllers/     HTTP handlers for auth, sessions, roles, and permissions
    services/        Database and business operations
    structs/         Shared response and database models
    router.rs        Route registration under the auth module
  db/                PostgreSQL connection setup
  middlewares/       Access, session, logging, and timeout middleware
  utils/             Config, env, hashing, token, cookie, and time helpers
  lib.rs             App state and Axum app construction
  main.rs            Binary entry point

config/              Base and environment-specific TOML configuration
migrations/          SQLx database migrations
postman/             Importable Postman collection and usage notes
tests/               Integration tests for API flows
Dockerfile           Multi-stage production image build
compose.yaml         Local app plus PostgreSQL Compose stack
```

The application is created in `src/lib.rs`, where auth routes are nested under `/api/v1/auth` and
global middleware is applied. Runtime startup lives in `src/main.rs`, which loads environment files,
loads and validates configuration, connects to PostgreSQL, initializes app state, and starts the
Axum server.

## Requirements

- Rust `1.85+`
- Cargo
- PostgreSQL `15+`, or Docker for running PostgreSQL locally
- `sqlx-cli` for applying database migrations
- `cargo-watch`, optional, for the `cargo dev` workflow
- Node.js and Bun, optional, for repository contribution tooling
- Postman, optional, for importing and running the API collection

## Setup & Execution

This section is intentionally detailed. The commands below are meant to make local development and
microservice integration predictable from a fresh clone.

### 1. Core Prerequisites

Install the required tooling:

- [Rust](https://www.rust-lang.org/tools/install), version `1.85` or newer.
- [Docker](https://www.docker.com/) for running PostgreSQL locally.
- [sqlx-cli](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli) for database migrations.
- [cargo-watch](https://crates.io/crates/cargo-watch) for the `cargo dev` workflow.
- [Node.js](https://nodejs.org/en/download/) and [Bun](https://bun.sh/) for contribution tooling.

Install Rust helpers:

```bash
cargo install sqlx-cli
cargo install cargo-watch
```

Install contribution tooling:

```bash
bun install
```

`bun install` installs Husky, Commitlint, and Prettier. These are used for repository workflow
checks; the service runtime remains Rust-based.

### 2. Clone The Project

For standalone development:

```bash
git clone https://github.com/Okpainmo/rusty-auth.git
cd rusty-auth
```

Install the Node/Bun-powered contribution hooks:

```bash
bun install
```

### 3. Integrating Into A Microservice Project

If you want Rusty Auth inside an existing microservice workspace, add it as a standalone auth
service inside your parent repository's services or microservices directory.

Move into the preferred services directory:

```bash
cd <microservice-services-dir>
```

Clone the auth service:

```bash
git clone --single-branch --branch main https://github.com/Okpainmo/rusty-auth <preferred-auth-service-name>
```

Move into the service directory:

```bash
cd <preferred-auth-service-name>
```

Remove the Git history so the service becomes part of your parent project:

```bash
rm -rf .git
```

Keep the runtime files that make Rusty Auth a service:

```text
src/
config/
migrations/
Cargo.toml
Cargo.lock
schema.sql
.cargo/config.toml
.env.sample
.env.development.sample
.env.staging.sample
.env.production.sample
```

Keep `postman/` if you want the parent project to retain the importable API collection for manual
testing and onboarding.

Remove repository-specific community and contribution files if your parent project already owns
those concerns:

```bash
rm -rf \
  .github \
  .husky \
  .codex \
  .vscode \
  CHANGELOG.md \
  CODE_OF_CONDUCT.md \
  CONTRIBUTING.md \
  SECURITY.md \
  commitlint.config.mjs \
  package.json \
  bun.lock \
  prettier.config.mjs
```

You can also remove `README.md` and `LICENSE` if the parent repository already provides project-wide
documentation and licensing. Keep them if the auth service should remain documented as its own
standalone unit inside the parent repo.

If your parent repository also uses Bun/Node tooling and you choose to keep `package.json`, remove
the package `prepare` script if you are not using this repository's Husky setup:

```bash
bun pm pkg delete scripts.prepare
```

Even inside a broader microservice system, configure Rusty Auth as an independent service. Give it
its own database, environment file, JWT secret, cookie policy, and deployment lifecycle. Other
services should treat it as the authentication boundary and consume the tokens/session data it
issues through explicit API contracts.

### 4. Environment Files

The project uses dotenv files plus layered TOML configuration.

Create active environment files from the samples:

```bash
cp .env.sample .env
cp .env.development.sample .env.development
```

For staging or production work, create the matching files:

```bash
cp .env.staging.sample .env.staging
cp .env.production.sample .env.production
```

The root `.env` file selects which environment-specific dotenv file is loaded by `load_env()`:

```dotenv
APP__DEPLOY__ENV=development
```

The active environment-specific file should also set `APP__ENV`, which selects the matching TOML
configuration layer:

```dotenv
APP__ENV=development
```

For example, `APP__DEPLOY__ENV=development` loads `.env.development`, and `APP__ENV=development`
loads `config/development.toml`.

Your `.env.development` should include values like:

```dotenv
# Environment
APP__ENV=development

# Server
APP__SERVER__PORT=8000

# Database
APP__DATABASE__ENGINE=postgres
APP__DATABASE__HOST=localhost
APP__DATABASE__NAME=rusty-auth-dev-db
APP__DATABASE__PASSWORD=supersecret
APP__DATABASE__PORT=5433
APP__DATABASE__USER=okpainmo

# JWT
APP__AUTH__JWT_ACCESS_EXPIRATION_TIME_IN_HOURS=1
APP__AUTH__JWT_REFRESH_EXPIRATION_TIME_IN_HOURS=24
APP__AUTH__JWT_ONE_TIME_PASSWORD_LIFETIME_IN_MINUTES=5
APP__AUTH__JWT_SECRET="generate-a-strong-random-secret"

# Rate Limit
APP__CLIENT_INTEGRATIONS__ALLOW_RATE_LIMIT_MIDDLEWARE=false
APP__RATE_LIMIT__ENABLED=true
APP__RATE_LIMIT__REQUESTS_PER_WINDOW=60
APP__RATE_LIMIT__WINDOW_SECS=60
```

Real secrets should never be committed. Keep local and production `.env` files out of source
control.

### 5. Database Setup

The standard local database path is Compose, because `compose.yaml` already matches the app's
expected PostgreSQL version and initializes the schema from `schema.sql`.

Start only PostgreSQL with development-style credentials:

```bash
POSTGRES_USER=okpainmo \
POSTGRES_PASSWORD=supersecret \
POSTGRES_DB=rusty-auth-dev-db \
POSTGRES_PORT=5433 \
docker compose up -d db
```

Check that the database is healthy:

```bash
docker compose ps
```

If you prefer a one-off raw Docker container, use equivalent values:

```bash
docker run -d \
  --name <container-name> \
  -p 5433:5432 \
  -e POSTGRES_USER=<user-name> \
  -e POSTGRES_PASSWORD=<password> \
  -e POSTGRES_DB=<database-name> \
  postgres
```

Example:

```bash
docker run -d \
  --name rusty-auth-dev-db \
  -p 5433:5432 \
  -e POSTGRES_USER=okpainmo \
  -e POSTGRES_PASSWORD=supersecret \
  -e POSTGRES_DB=rusty-auth-dev-db \
  postgres
```

Check that the container is running:

```bash
docker ps
```

Run the SQLx migrations:

```bash
sqlx migrate run --database-url postgres://<user-name>:<password>@localhost:5433/<database-name>
```

Example:

```bash
sqlx migrate run --database-url postgres://okpainmo:supersecret@localhost:5433/rusty-auth-dev-db
```

If you need to stop and start the local database later:

```bash
docker compose stop db
docker compose start db
```

If you need to inspect migration status:

```bash
sqlx migrate info --database-url postgres://okpainmo:supersecret@localhost:5433/rusty-auth-dev-db
```

### 6. Running The Server

Run the server once:

```bash
cargo run
```

Run the server in development mode with auto-reload:

```bash
cargo dev
```

The `cargo dev` command is defined in `.cargo/config.toml` as a `cargo-watch` alias. It watches
`src` and reruns the server when source files change.

If you are developing on WSL and file changes do not trigger reloads, switch the alias in
`.cargo/config.toml` to the polling version:

```toml
[alias]
dev = ["watch", "--poll", "-c", "-w", "src", "-x", "run -- --config config.yaml"]
```

When the server starts successfully, it binds to the configured host and port. With the default
development configuration, the API is available at:

```text
http://127.0.0.1:8000/api/v1/auth
```

Smoke-check the API by registering a disposable user:

```bash
curl -X POST http://127.0.0.1:8000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "first_name": "Smoke",
    "last_name": "User",
    "email": "smoke@example.com",
    "password": "password123",
    "country": "Testland",
    "country_code": "TL",
    "phone_number": "1000000001"
  }'
```

Expected successful responses include the project-wide shape:

```json
{
  "response_message": "Registration successful",
  "response": {
    "session_id": "<session-id>",
    "access_token": "<access-token>",
    "refresh_token": "<refresh-token>"
  },
  "error": null
}
```

### 7. Docker And Compose

Build the application image:

```bash
docker compose build app
```

Start the app and PostgreSQL together:

```bash
docker compose up
```

Run them in the background:

```bash
docker compose up -d
```

By default, Compose exposes:

```text
Rusty Auth API: http://127.0.0.1:8000/api/v1/auth
PostgreSQL:     127.0.0.1:5433
```

The Compose database uses these local defaults:

```text
POSTGRES_USER=rusty_auth
POSTGRES_PASSWORD=rusty_auth
POSTGRES_DB=rusty_auth
```

Override them from your shell or a local environment file when needed. For development tests, use
the same values shown in [Database Setup](#5-database-setup) so `.env.development` and the Compose
database agree.

```bash
POSTGRES_USER=auth_user \
POSTGRES_PASSWORD=supersecret \
POSTGRES_DB=auth_db \
APP__AUTH__JWT_SECRET=replace-with-a-strong-secret \
docker compose up -d
```

The app image is built with a multi-stage `Dockerfile` and `cargo-chef` dependency caching, so
dependency layers can be reused across source-only rebuilds. The runtime image only carries the
compiled `auth` binary and the `config/` directory required by the config loader.

On first database initialization, Compose mounts `schema.sql` into Postgres'
`/docker-entrypoint-initdb.d` directory so the local database starts with the current consolidated
schema. The app container itself uses the `db` service hostname and port `5432`; the host machine
uses `127.0.0.1:5433` by default.

If the named database volume already exists, Postgres will not replay initialization files. Reset
the local Compose database only when you intentionally want a fresh database:

```bash
docker compose down -v
```

Then start it again:

```bash
docker compose up -d
```

### 8. Creating New Migrations

Create a new migration file:

```bash
sqlx migrate add <migration_name>
```

Example:

```bash
sqlx migrate add add_last_login_to_users
```

Edit the generated SQL file in `migrations/`, then apply it:

```bash
sqlx migrate run --database-url postgres://<user-name>:<password>@localhost:5433/<database-name>
```

Example:

```bash
sqlx migrate run --database-url postgres://okpainmo:supersecret@localhost:5433/rusty-auth-dev-db
```

### 9. Development Checks

Check that the Rust code compiles:

```bash
cargo check
```

Format Rust code:

```bash
cargo fmt
```

Verify Rust formatting without changing files:

```bash
cargo fmt --check
```

Run Clippy:

```bash
cargo clippy
```

Format Markdown files:

```bash
bun run format
```

Check Markdown formatting:

```bash
bun run format:check
```

## Postman Collection

Postman collection files are available in the [postman](postman) directory. Import the collection
when you want to exercise the auth API without rebuilding requests by hand.

| Collection       | File                                                                                     | Folders                                                     |
| ---------------- | ---------------------------------------------------------------------------------------- | ----------------------------------------------------------- |
| `Rusty Auth API` | [postman/rusty-auth.postman_collection.json](postman/rusty-auth.postman_collection.json) | `Public Auth`, `Sessions`, `Roles`, `Permissions`, `Logout` |

The collection includes requests for:

- Public registration and login.
- Session listing, lookup, and expiry update.
- Role creation, update, assignment, removal, and listing.
- Permission creation, update, and listing.
- Logout and session revocation.

Before running protected requests, confirm or update these collection variables:

| Variable         | Purpose                                      | Default                                |
| ---------------- | -------------------------------------------- | -------------------------------------- |
| `base_url`       | Rusty Auth API base URL                      | `http://127.0.0.1:8000/api/v1/auth`    |
| `access_token`   | Bearer token captured from register or login | empty                                  |
| `refresh_token`  | Session token captured from register/login   | empty                                  |
| `session_id`     | Backing database session ID                  | empty                                  |
| `user_id`        | Authenticated user ID                        | empty                                  |
| `user_email`     | Email used for login/logout examples         | `ada@example.com`                      |
| `auth_cookie`    | Value of the `auth_cookie` cookie            | empty                                  |
| `role_id`        | Role used by role/permission examples        | `00000000-0000-0000-0000-000000000001` |
| `permission_id`  | Permission used by role-permission examples  | empty                                  |
| `target_user_id` | User target for role/session examples        | empty                                  |

Run `Public Auth / Register User`, `Public Auth / Register Admin`, or `Public Auth / Login` first so
Postman can capture the auth variables required by protected routes. The collection also updates
`access_token`, `refresh_token`, and `session_id` from protected-route responses, which keeps the
collection aligned with Rusty Auth's session-token rotation behavior. Run
`Logout / Logout Current Session` last when manually testing a flow, because it revokes the current
backing session.

See [postman/README.md](postman/README.md) for import and usage notes.

## Traffic Simulation

Rusty Auth includes a k6 traffic simulation script at:

```text
k6-traffic-simulation/main.js
```

The script creates one authenticated session per virtual user, then rotates through protected
session, role, and permission routes while preserving fresh tokens returned by protected responses.
Use [k6-traffic-simulation/.env.sample](k6-traffic-simulation/.env.sample) as the template for local
k6-only environment values.

Create a local k6 environment file from the template:

```bash
cp k6-traffic-simulation/.env.sample k6-traffic-simulation/.env
```

Edit `k6-traffic-simulation/.env`, then source it and run the simulation:

```bash
set -a
. ./k6-traffic-simulation/.env
set +a
k6 run k6-traffic-simulation/main.js
```

See [k6-traffic-simulation/README.md](k6-traffic-simulation/README.md) for environment-loading
details. The k6 script does not automatically read the app's root `.env`; source the dedicated
`k6-traffic-simulation/.env` file before running it.

## Configuration

The project uses the `config` crate and a layered configuration model.

All dotenv files are loaded before configuration is deserialized:

1. `.env`, which selects the deployment environment with `APP__DEPLOY__ENV`.
2. `.env.{APP__DEPLOY__ENV}`, for example `.env.development`.

The config loader then resolves application configuration in this order, from lowest to highest
priority:

1. `config/base.toml`
2. `config/{APP__ENV}.toml`, such as `config/development.toml`
3. `config/local.toml`, if present
4. Environment variables prefixed with `APP__`

Environment variables use double underscores to map to nested TOML fields.

```text
APP__<SECTION>__<FIELD>=value
```

Examples:

```bash
APP__ENV=development
APP__SERVER__PORT=9000
APP__DATABASE__HOST=localhost
APP__DATABASE__PORT=5433
APP__DATABASE__USER=okpainmo
APP__DATABASE__PASSWORD=supersecret
APP__DATABASE__NAME=rusty-auth-dev-db
APP__AUTH__JWT_SECRET=replace-with-a-real-secret
APP__CLIENT_INTEGRATIONS__ALLOW_RATE_LIMIT_MIDDLEWARE=true
APP__RATE_LIMIT__REQUESTS_PER_WINDOW=60
APP__RATE_LIMIT__WINDOW_SECS=60
```

This TOML:

```toml
[server]
port = 8000
```

Can be overridden with:

```bash
APP__SERVER__PORT=9000
```

Common runtime variables:

| Variable                                                | Purpose                                | Default/Source        |
| ------------------------------------------------------- | -------------------------------------- | --------------------- |
| `APP__DEPLOY__ENV`                                      | Selects which `.env.*` file to load    | `development`         |
| `APP__ENV`                                              | Selects the TOML config environment    | required by loader    |
| `APP__SERVER__HOST`                                     | Server bind host                       | `127.0.0.1`           |
| `APP__SERVER__PORT`                                     | Server bind port                       | `8000`                |
| `APP__SERVER__REQUEST_TIMEOUT_SECS`                     | Request timeout in seconds             | `60`                  |
| `APP__DATABASE__ENGINE`                                 | Database engine                        | `postgres`            |
| `APP__DATABASE__HOST`                                   | PostgreSQL host                        | environment-specific  |
| `APP__DATABASE__PORT`                                   | PostgreSQL port                        | `5433` in development |
| `APP__DATABASE__USER`                                   | PostgreSQL user                        | environment-specific  |
| `APP__DATABASE__PASSWORD`                               | PostgreSQL password                    | environment-specific  |
| `APP__DATABASE__NAME`                                   | PostgreSQL database name               | environment-specific  |
| `APP__DATABASE__MAX_CONNECTIONS`                        | PostgreSQL pool size                   | `20`                  |
| `APP__DATABASE__CONNECT_TIMEOUT_SECS`                   | PostgreSQL connection timeout          | `5`                   |
| `APP__AUTH__JWT_SECRET`                                 | JWT signing secret                     | environment override  |
| `APP__AUTH__JWT_ACCESS_EXPIRATION_TIME_IN_HOURS`        | Access-token lifetime                  | `1`                   |
| `APP__AUTH__JWT_REFRESH_EXPIRATION_TIME_IN_HOURS`       | Refresh/session-token lifetime         | `24`                  |
| `APP__AUTH__JWT_ONE_TIME_PASSWORD_LIFETIME_IN_MINUTES`  | OTP token lifetime                     | `5`                   |
| `APP__CLIENT_INTEGRATIONS__ALLOW_RATE_LIMIT_MIDDLEWARE` | Enables rate-limit middleware          | `false`               |
| `APP__RATE_LIMIT__ENABLED`                              | Enables limiter when middleware runs   | `true`                |
| `APP__RATE_LIMIT__REQUESTS_PER_WINDOW`                  | Allowed requests per rate-limit window | `60`                  |
| `APP__RATE_LIMIT__WINDOW_SECS`                          | Rate-limit window length               | `60`                  |

### Required Configuration Sections

Startup validation requires:

- `app.name`
- `server.host`
- `server.port`
- `server.request_timeout_secs`
- `database.engine`
- `database.host`
- `database.port`
- `database.user`
- `database.password`
- `database.name`
- `database.max_connections`
- `database.connect_timeout_secs`
- `auth.jwt_secret`
- `auth.jwt_access_expiration_time_in_hours`
- `auth.jwt_refresh_expiration_time_in_hours`
- `auth.jwt_one_time_password_lifetime_in_minutes`

## Rate Limiting

Rusty Auth includes an in-process fixed-window rate limiter. It is disabled by default through the
middleware feature flag, but its defaults are defined in `config/base.toml`.

```toml
[client_integrations]
allow_rate_limit_middleware = true

[rate_limit]
enabled = true
requests_per_window = 60
window_secs = 60
```

The limiter identifies clients by IP address. It checks request metadata in this order:

1. `x-forwarded-for`
2. `x-real-ip`
3. Axum `ConnectInfo<SocketAddr>`
4. `unknown`

When a client exceeds the configured limit, the service returns:

```http
HTTP/1.1 429 Too Many Requests
```

```json
{
  "error": "Too Many Requests",
  "response_message": "Rate limit exceeded. Please try again later."
}
```

Rate-limit responses include:

- `retry-after`
- `x-ratelimit-limit`
- `x-ratelimit-remaining`
- `x-ratelimit-reset`

Allowed responses also include the `x-ratelimit-*` headers.

This first implementation is local to a single service instance. If Rusty Auth is deployed behind
multiple replicas, use a shared backend such as Redis for distributed rate limiting.

## Security Model

Rusty Auth includes several security-oriented defaults and checks:

- Passwords are hashed with Argon2.

- Access and refresh tokens are JWTs with explicit token-kind claims.

- Refresh/session tokens are hashed before database storage.

- Submitted refresh/session tokens are verified against the stored session hash before protected
  routes execute.

- Bearer access tokens are verified from the request and must match the resolved user and access
  token kind.

- Auth cookies are HTTP-only.

- Auth cookies are marked `Secure` outside development.

- Cookies use `SameSite=Lax`.

- Protected routes require the auth cookie, access token, refresh/session token, user ID, and
  session ID.

- Rate limiting can reject excessive requests before they reach protected route logic.

- Inactive users are blocked by session middleware.

- Revoked sessions are rejected.

- Logout clears the cookie and revokes the backing session.

- Production secrets are expected to come from environment variables or a secret manager.

## Testing

Run all tests:

```bash
cargo test
```

Run library/unit tests:

```bash
cargo test --lib
```

Run the integration test target:

```bash
cargo test --test controllers
```

For the most predictable local run against one shared development database, run the controller
target with one test thread:

```bash
cargo test --test controllers -- --test-threads=1
```

The integration tests live in `tests/controllers` and cover flows across:

- User registration.

- Admin registration.

- Login.

- Logout.

- Session listing, lookup, and update.

- Role creation, update, listing, assignment, and removal.

- Permission creation, update, listing, and user permission lookup.

- Role-permission assignment and deletion.

Integration tests use `tests/common/shared.rs` to create an Axum `TestServer`, connect to the
configured PostgreSQL database, and provide helpers for authenticated requests.

Before running integration tests, make sure:

- PostgreSQL is running.

- `.env.development` exists.

- Database variables point to the test/development database.

- The schema has been initialized through Compose's `schema.sql` mount or migrations have been
  applied manually.

The repository pre-push hook always runs `cargo test --lib`. It runs the controller integration
target only when the configured local database port is reachable, so contributors without a running
PostgreSQL instance can still push docs or unit-only changes.

## Operational Notes

- **Rusty Auth is stateful**. User accounts, role mappings, permissions, sessions, and sub-session
  audit records live in PostgreSQL. Treat database availability, backups, migrations, and connection
  pool sizing as part of the service deployment.

- **Run one logical auth service against one authoritative auth database** unless you have planned
  database-level consistency, migration coordination, and secret rotation across replicas.

- **JWT secrets are deployment-critical**. Rotating `APP__AUTH__JWT_SECRET` invalidates tokens
  signed with the previous value unless you add explicit multi-key rotation support.

- **Protected routes require both token and session context**. Callers must send the access token,
  refresh/session token, session ID, user ID, and `auth_cookie` value together.

- **Session rotation is response-driven**. Protected-route responses can return fresh access and
  refresh tokens. Client applications should update their stored tokens from successful protected
  responses when using this flow.

- **Rate limiting is in-process**. It is suitable as a first local guard. For multiple replicas, use
  a shared backend such as Redis or move rate limiting to an API gateway.

- **Keep auth service network access narrow**. In a microservice deployment, expose public auth
  routes through the project gateway and restrict direct database access to the auth service.

- **The Postman collection is an operator aid**. It is useful for onboarding and manual smoke tests,
  but automated confidence should come from the integration tests and CI workflow.

## Contribution Workflow

The project uses Husky, Commitlint, and Prettier through Bun to keep contribution standards
consistent.

Install the tooling:

```bash
bun install
```

Commit messages should follow Conventional Commits:

```bash
git commit -m "feat: add permission deletion endpoint"
git commit -m "fix: renew session expiry correctly"
git commit -m "test: cover role permission removal"
```

The pre-commit flow checks that Rust code compiles and is formatted. If a commit fails because of
formatting, run:

```bash
cargo fmt
cargo check
```

Then retry the commit.

## Operating System Notes

On WSL, filesystem events may not always trigger `cargo watch`. If `cargo dev` does not restart the
server after edits, use the polling alias documented in [Running The Server](#6-running-the-server).

## Contributing

Contributions are welcome. Please read the [Contributing Guidelines](CONTRIBUTING.md) before opening
a pull request.

We are committed to a friendly and safe project environment. Please review the
[Code of Conduct](CODE_OF_CONDUCT.md).

## Security

If you discover a security-related issue, please follow the [Security Policy](SECURITY.md) instead
of opening a public issue.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
