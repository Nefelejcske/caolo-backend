.PHONY: test

miri_alloc:
	cd alloc && cargo miri test

miri_ecs:
	cd entity && cargo miri test

test_all:
	cargo test --tests --all --benches
	cargo test --doc


test: test_all miri_alloc miri_ecs

