build database="sqlite":
    just docs
    ./scripts/download_styles.sh
    cargo build -r --no-default-features --features {{database}}

docs:
    cargo doc --no-deps --document-private-items

test:
    just docs
    bun run static_build.ts
    cargo run

run:
    chmod +x ./target/release/bundlrs
    ./target/release/bundlrs
