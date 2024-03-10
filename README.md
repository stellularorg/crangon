# ☄️ bundlrs

*Bundlrs* is a rewrite of [Bundles](https://codeberg.org/SentryTwo/bundles) in Rust without some of the extra features.

Bundlrs is a *super* lightweight and [anonymous](#user-accounts) social markdown platform featuring pastes with custom URLs, quick and full deletion, easy editing, live preview, advanced styling, and [much more](#features)!

For migration from Bundles, please see [#3](https://code.stellular.org/stellular/bundlrs/issues/3).

## Install

Build:

```bash
bun run build
# release
bun run build:release
# release (mysql)
bun run build:release:mysql
# release (postgres)
bun run build:release:postgres
```

Run:

```bash
chmod +x ./target/debug/bundlrs && ./target/debug/bundlrs
# release
chmod +x ./target/release/bundlrs && ./target/release/bundlrs
```

Bundlrs supports the features `sqlite`, `postgres`, and `mysql`. These features dictate which database types will be used.

## Configuration

Bundlrs is configured through flags given when running the server. The following flags are available:

- `--port 0000` optional (defaults to `8080`)
- `--static-dir "/path/to/dir` optional (defaults to `./static`)
- `--db-type "type"` optional (defaults to `sqlite`)

Environment variables:

- `INFO` optional (defaults to `/pub/info`)
- `BODY_EMBED` optional (defaults to nothing)
- `DB_HOST "host"` optional (defaults to `localhost`) (only if `--db-type` is `postgres` or `mysql`)
- `DB_USER "user"` **required** (only if `--db-type` is `postgres` or `mysql`)
- `DB_PASS "pass"` **required** (only if `--db-type` is `postgres` or `mysql`)
- `DB_NAME "name"` **required** (only if `--db-type` is `postgres` or `mysql`)
- `SITE_NAME "name"` optional (defaults to `Bundlrs`)
- `AUTH_REQUIRED` optional (defaults to `(None)`), disables creating pastes without an account
- `REGISTRATION_DISABLED` optional (defaults to `(None)`)

## Features

- [Bundlrs Info Page](https://stellular.net/pub/info)
- [Markdown Info Page](https://stellular.net/pub/markdown)
- [API Docs](https://stellular.net/api/docs/bundlrs/index.html)

## Boards

Boards are simple forum-like rooms that can be created by users. Boards can be made private by changing `is_private` to `yes` in their configuration page; however this will not stop [users](#user-accounts) with the `Staff` role from viewing and posting on the board.

Boards can be given tags in their configuration. All tags must start with `+` and must be space separated.

## User Accounts

Users can register for an account with just a username. They are given a unique ID which will be used as their password. This ID is stored hashed on the server and cannot be reset.
