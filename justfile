build database="sqlite":
    just docs
    just styles
    bun i
    bun run static_build.ts
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

styles:
    wget https://codeberg.org/api/packages/hkau/npm/fusion/-/1.0.11/fusion-1.0.11.tgz -O fusion.tgz
    tar -xzf fusion.tgz
    mv ./package/src/css ./static/css
    sed -i -e 's/\"\/utility.css\"/\"\/static\/css\/utility.css\"/' ./static/css/fusion.css
    rm -r ./package
    rm ./fusion.tgz
