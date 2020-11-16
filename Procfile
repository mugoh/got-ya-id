release: cargo install diesel_cli --no-default-features --features postgres
release: diesel setup
release: diesel migration run --migration-dir=src/diesel_cfg/migrations/
web ./target/release/got-ya-id
