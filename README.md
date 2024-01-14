# bundlrs

*Bundlrs* is a rewrite of [Bundles](https://codeberg.org/SentryTwo/bundles) in Rust without some of the extra features.

Bundlrs has **not** reached acceptable parity with Bundles *yet*.

## Install

Install styles:

```bash
chmod +x scripts/download_styles.sh && ./scripts/download_styles.sh
```

Build:

```bash
cargo build -r
```

Run:

```bash
chmod +x ./target/release/bundlrs && ./target/release/bundlrs
```

## Configuration

Bundlrs is configured through flags given when running the server. The following flags are available:

- `--port 0000` optional (defaults to `8080`)
- `--db-host "host"` optional (defaults to `localhost`)
- `--db-user "user"` **required**
- `--db-pass "pass"` **required**
- `--db-name "name"` **required**
