.DEFAULT_GOAL := all
.PHONY: api sim rt

init:
	@${MAKE} -C api init
	@${MAKE} -C rt init

test-sim:
	@${MAKE} -C sim test

test-rt:
	@${MAKE} -C rt test

test-api:
	@${MAKE} -C api test


test: test-sim test-rt test-api
