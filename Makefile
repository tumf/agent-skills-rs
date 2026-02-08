.PHONY: help fetch build release fmt lint test check prek clean

help:
	@printf "Targets:\n"
	@printf "  make fetch    - cargo fetch\n"
	@printf "  make build    - cargo build --all-targets\n"
	@printf "  make release  - cargo release\n"
	@printf "  make fmt      - cargo fmt --all\n"
	@printf "  make lint     - cargo clippy --all-targets -- -D warnings\n"
	@printf "  make test     - cargo test\n"
	@printf "  make check    - fmt + lint + test\n"
	@printf "  make prek     - alias of check\n"
	@printf "  make clean    - cargo clean\n"

fetch:
	cargo fetch

build:
	cargo build --all-targets

release:
	cargo release

fmt:
	cargo fmt --all

lint:
	cargo clippy --all-targets -- -D warnings

test:
	cargo test

check: fmt lint test

prek: check

clean:
	cargo clean
