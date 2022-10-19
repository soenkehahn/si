ci: test clippy

install: ci
  cargo install --path . --force --locked

test:
  cargo test --color=always -- --test-threads=1

clippy:
  cargo clippy -- --deny clippy::all
