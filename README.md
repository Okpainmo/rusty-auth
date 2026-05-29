# Rusty Auth

[![CI](https://github.com/Okpainmo/rusty-auth/actions/workflows/ci.yml/badge.svg)](https://github.com/Okpainmo/rusty-auth/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![Conventional Commits](https://img.shields.io/badge/Conventional%20Commits-1.0.0-yellow.svg)](https://conventionalcommits.org)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](CODE_OF_CONDUCT.md)

Rusty Auth is a production-minded authentication service built with Rust, Axum, PostgreSQL, SQLx,
JWTs, secure cookies, and role-based access-control primitives.

It is designed to be dropped into a microservice system as a ready auth boundary, while still being
small enough to understand, customize, and extend. Out of the box it handles user/admin
registration, login, logout, session persistence, token rotation, role/permission management,
sub-session audit logs, environment-aware configuration, and integration tests against real HTTP
flows.

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

The session middleware verifies the user, cookie, session ID, refresh/session token, session status,
and token claims. When the session is valid, it renews the session and issues fresh tokens for the
current protected-route response.

The access middleware then verifies that the access token belongs to the resolved user and has the
expected access-token kind.

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
tests/               Integration tests for API flows
```

The application is created in `src/lib.rs`, where auth routes are nested under `/api/v1/auth` and
global middleware is applied. Runtime startup lives in `src/main.rs`, which loads environment files,
loads and validates configuration, connects to PostgreSQL, initializes app state, and starts the
Axum server.

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

If you want Rusty Auth inside an existing microservice workspace, clone it into your services
directory.

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

Remove repository-specific community and contribution files if your parent project already owns
those concerns:

```bash
rm -rf .github .husky .codex .vscode CHANGELOG.md CODE_OF_CONDUCT.md CONTRIBUTING.md commitlint.config.mjs LICENSE SECURITY.md
```

Remove the package `prepare` script if you are not using this repository's Husky setup:

```bash
bun pm pkg delete scripts.prepare
```

If your parent repository also uses Bun/Node tooling, review `package.json`, `bun.lock`,
`prettier.config.mjs`, and the remaining scripts before deleting them.

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

Set the active application environment with `APP__ENV`:

```bash
APP__ENV=development
```

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

Start a local PostgreSQL database with Docker:

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
docker stop rusty-auth-dev-db
docker start rusty-auth-dev-db
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

### 7. Creating New Migrations

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

### 8. Development Checks

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

## Configuration

The project uses the `config` crate and a layered configuration model.

Configuration loading order, from lowest to highest priority:

1. `config/base.toml`
2. `config/{APP__ENV}.toml`, such as `config/development.toml`

3. `config/local.toml`

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

The integration tests live in `tests/controllers` and cover flows across:

- User registration.

- Admin registration.

- Login.

- Logout.

- Session listing, lookup, and update.

- Role creation, update, listing, assignment, and removal.

- Permission creation, update, listing, and user permission lookup.

- Role-permission assignment and deletion.

Integration tests use `tests/common/mod.rs` to create an Axum `TestServer`, connect to the
configured PostgreSQL database, and provide helpers for authenticated requests.

Before running integration tests, make sure:

- PostgreSQL is running.

- `.env.development` exists.

- Database variables point to the test/development database.

- Migrations have been applied.

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
