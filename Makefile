.DEFAULT_GOAL := all
.PHONY: api sim rt

test-sim:
	@${MAKE} -C sim test

test-rt:
	@${MAKE} -C rt test

test-api:
	@${MAKE} -C api test


test: test-sim test-rt test-api

start:
	docker-compose up -d
	docker-compose logs -f --tail=100

rt:
	docker build -t caolo/caolo-rt:bleeding -f ./rt.Dockerfile .

api:
	docker build -t caolo/caolo-api:bleeding -f ./api.Dockerfile .

sim:
	docker build -t caolo/caolo-sim:bleeding -f ./sim.Dockerfile .

release:
	docker build -t caolo/caolo-release:bleeding -f release.Dockerfile .

all: api sim release rt

push_api:api
	docker push caolo/caolo-api:bleeding

push_sim:sim
	docker push caolo/caolo-sim:bleeding

push_rt:rt
	docker push caolo/caolo-rt:bleeding

push_release:release
	docker push caolo/caolo-release:bleeding

push: push_api push_release push_rt push_sim
