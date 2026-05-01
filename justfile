run *ARGS:
  RUST_LOG=info cargo run {{ ARGS }}

watch *ARGS:
  cargo watch -x run {{ ARGS }}

build:
  cargo build --release

test:
  cargo test

watch-test:
  cargo watch -x test

coverage:
  cargo tarpaulin

fmt:
  cargo fmt

migrate:
  sqlx migrate run --database-url sqlite:instance/iliad.db

docker:
  @nix build .#docker
  @docker load < result
  @docker compose up

cli *ARGS:
  @./iliadctl {{ ARGS }}
