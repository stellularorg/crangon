# 🗃️ bundlrs

*Bundlrs* is a rewrite of [Bundles](https://codeberg.org/SentryTwo/bundles) in Rust without some of the extra features.

Bundlrs is a *super* lightweight and [anonymous](#user-accounts) markdown pastebin featuring custom URLs, quick and full deletion, easy editing, live preview, advanced styling, and [much more](#features)!

## Install

Build:

```bash
bun run build
# release
bun run build:release
```

Run:

```bash
chmod +x ./target/debug/bundlrs && ./target/debug/bundlrs
# release
chmod +x ./target/release/bundlrs && ./target/release/bundlrs
```

## Configuration

Bundlrs is configured through flags given when running the server. The following flags are available:

- `--port 0000` optional (defaults to `8080`)
- `--static-dir "/path/to/dir` optional (defaults to `./static`)
- `--db-type "type"` optional (defaults to `sqlite`)

Environment variables:

- `INFO` optional (defaults to `/pub/info`)
- `BODY_EMBED` optional (defaults to nothing)
- `PSQL_HOST "host"` optional (defaults to `localhost`) (only if `--db-type` is not `sqlite`)
- `PSQL_USER "user"` **required** (only if `--db-type` is not `sqlite`)
- `PSQL_PASS "pass"` **required** (only if `--db-type` is not `sqlite`)
- `PSQL_NAME "name"` **required** (only if `--db-type` is not `sqlite`)

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
