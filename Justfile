ci: test clippy

test:
  cargo test --color=always -- --test-threads=1 --quiet

clippy:
  cargo clippy -- --deny clippy::all
