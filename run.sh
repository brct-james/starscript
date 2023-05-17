clear
docker compose up -d --remove-orphans
cargo fmt
DATABASE_URL="postgresql://starscript:fj923ofl23dj89129@localhost:5433/starscript" cargo run