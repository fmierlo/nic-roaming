
.PHONY: run clean check dev-deps test test~% report

TARPAULIN_FLAGS := --output-dir target/tarpaulin --out Stdout --out Html

run: check
	cargo run $(args)

build: check
	cargo doc
	cargo build

clean:
	cargo clean

check:
	cargo check --all-targets
	cargo fmt --all --check
	cargo clippy

dev-deps:
	cargo install cargo-tarpaulin

test: check
	cargo tarpaulin $(TARPAULIN_FLAGS)

test~%:
	cargo tarpaulin $(TARPAULIN_FLAGS) -- $(*)

report: test
	open target/tarpaulin/tarpaulin-report.html

open: build
	cargo doc --open
