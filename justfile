default: lint build
all: lint build test
ci: lint-strict build-release test

###

build:
    cargo build

build-release:
    cargo build --release

clean:
    cargo clean

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
