
.PHONY: run clean install test test~% report

export RUST_BACKTRACE=1

TARPAULIN_FLAGS := --output-dir target/tarpaulin --out Stdout --out Html

run: check
	cargo run $(args)

clean:
	cargo clean

check:
	cargo clippy
	cargo fmt --check

test-deps:
	cargo install cargo-tarpaulin

test: check
	cargo tarpaulin $(TARPAULIN_FLAGS)

test~%:
	cargo tarpaulin $(TARPAULIN_FLAGS) -- $(*)

report: test
	open target/tarpaulin/tarpaulin-report.html
