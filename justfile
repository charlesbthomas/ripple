default: check

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

lint:
    cargo clippy --all-targets -- -D warnings

test:
    cargo test

build:
    cargo build --release

check: fmt-check lint test

docs-build:
    cd docs && zola build

docs-serve:
    cd docs && zola serve
