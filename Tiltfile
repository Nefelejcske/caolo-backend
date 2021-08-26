k8s_yaml('k8s/db.yml')
k8s_yaml('k8s/queen.yml')
k8s_yaml('k8s/rt.yml')
k8s_yaml('k8s/web.yml')

docker_build('caolo/caolo-release', '.', dockerfile="release.Dockerfile")
docker_build('caolo/caolo-api', '.', dockerfile="api.Dockerfile")
docker_build('caolo/caolo-sim', '.', dockerfile="sim.Dockerfile")
docker_build('caolo/caolo-rt', '.', dockerfile="rt.Dockerfile")

k8s_resource('web', resource_deps=['web-db', 'queen'], port_forwards=8000)
k8s_resource('rt', resource_deps=['queen'], port_forwards=8080)
k8s_resource('queen')

allow_k8s_contexts('cloud_okteto_com')

local_resource('sim-tests', cmd='make -C sim test', deps=['./sim/simulation/', './sim/worker/', './sim/Cargo.lock'])
local_resource('rt-tests', cmd='make -C rt test', deps=['./rt'])
