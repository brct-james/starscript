clear
source surreal_secrets.env; docker compose up -d --remove-orphans --build
cargo fmt
cargo run