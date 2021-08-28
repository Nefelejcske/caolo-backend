.PHONY: test

test:
	cd alloc && cargo miri test
	cargo test --tests --all --benches
	cargo test --doc

