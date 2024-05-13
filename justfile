# Lint + Build
default: lint build
# Lint + Build + Test
all: lint build test
# Shadows CI checks as closely as possible
ci: lint-strict build-release test udeps deny

###

build:
    cargo build

build-release:
    cargo build --release

clean:
    cargo clean

deny:
    cargo deny check

docs:
    cargo doc

docs-open:
    cargo doc --open

example file:
    cargo run --example {{file}}

lint:
    cargo clippy

lint-strict:
    cargo clippy -- -D warnings

test:
    cargo test

udeps:
    cargo +nightly udeps
