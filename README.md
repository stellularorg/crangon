# 🗃️ bundlrs

*Bundlrs* is a rewrite of [Bundles](https://codeberg.org/SentryTwo/bundles) in Rust without some of the extra features.

Bundlrs is a *super* lightweight and [anonymous](#user-accounts) markdown pastebin featuring custom URLs, quick and full deletion, easy editing, live preview, advanced styling, and [much more](#features)!

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

## Features

Bundlrs supports all [Bundles features](https://bundles.cc/what#features) with some minor modifications. These are listed below with their reasons:

- Bundlrs does **not** support comments
    - Comments were the cause of many bugs in the original version of Bundles, as well as being one of the least used features. The amount of time taken to get comments working with the correct configuration was not worth the output.
- Bundles does **not** support writer mode
    - *\*this could change*
- Bundles does **not** support paste media
    - *\*this could change*

## User Accounts

Users can register for an account with just a username. They are given a unique ID which will be used as their password. This ID is stored hashed on the server and cannot be reset.
