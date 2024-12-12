
.PHONY: run clean install test test~% report

export RUST_BACKTRACE=1

TARPAULIN_FLAGS := --output-dir target/tarpaulin --out Stdout --out Html

run:
	cargo run $(if)

clean:
	rm -rf target

test-deps:
	cargo install cargo-tarpaulin

test:
	cargo tarpaulin $(TARPAULIN_FLAGS)

test~%:
	cargo tarpaulin $(TARPAULIN_FLAGS) -- $(*)

report: test
	open target/tarpaulin/tarpaulin-report.html
