build database="sqlite":
    just style
    cargo build -r --no-default-features --features {{database}}

test:
    just style
    cargo run

link:
    ./helpers/static.sh

style:
    just link
    bunx tailwindcss -i ./crates/crangon/static/input.css -o ./crates/crangon/static/style.css
