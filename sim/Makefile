.PHONY: test

test:
	cd storage && cargo miri test
	cargo test --tests --all --benches
	cargo test --doc

