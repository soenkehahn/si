ci: test clippy

install: ci
  cargo install --path . --force

test:
  cargo test --color=always -- --test-threads=1 --quiet

clippy:
  cargo clippy -- --deny clippy::all
