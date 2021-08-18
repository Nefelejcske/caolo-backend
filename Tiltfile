k8s_yaml('k8s/db.yml')
k8s_yaml('k8s/queen.yml')
k8s_yaml('k8s/rt.yml')
k8s_yaml('k8s/web.yml')

docker_build('caolo/caolo-api', '.', dockerfile="api.Dockerfile")
docker_build('caolo/caolo-release', '.', dockerfile="release.Dockerfile")
docker_build('caolo/caolo-sim', '.', dockerfile="sim.Dockerfile")
docker_build('caolo/caolo-rt', '.', dockerfile="rt.Dockerfile")

k8s_resource('web', resource_deps=['web-db', 'queen'])
k8s_resource('rt', resource_deps=['queen'])
