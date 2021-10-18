load("ext://helm_remote", "helm_remote")
helm_remote(
    "postgresql",
    repo_url="https://charts.bitnami.com/bitnami",
    set=[
        "postgresqlUsername=postgres",
        "postgresqlPassword=poggers",
        "postgresqlDatabase=caolodb",
    ],
)

k8s_yaml("k8s/queen.yml")
k8s_yaml("k8s/rt.yml")
k8s_yaml("k8s/web.yml")

docker_build("caolo/caolo-release", ".", dockerfile="release.Dockerfile")
docker_build("caolo/caolo-api", ".", dockerfile="api.Dockerfile")
docker_build("caolo/caolo-sim", ".", dockerfile="sim.Dockerfile")
docker_build("caolo/caolo-rt", ".", dockerfile="rt.Dockerfile")

local_resource(
    "sim-tests",
    cmd="make -C sim test",
    deps=[
        "./sim/simulation/",
        "./sim/worker/",
        "./sim/Cargo.lock",
        "./protos",
        "./sim/Makefile",
    ],
    allow_parallel=True,
    auto_init=False,
)
local_resource(
    "rt-tests",
    cmd="make -C rt test",
    deps=["./rt"],
    allow_parallel=True,
    auto_init=False,
)
local_resource(
    "api-tests",
    cmd="make -C api test",
    deps=[
        "./api/caoloapi/",
        "./api/test/",
        "./api/setup.py",
        "./api/poetry.lock",
        "./api/pyproject.toml",
        "./api/Makefile",
    ],
    allow_parallel=True,
    auto_init=False,
)

k8s_resource("caolo-queen")
k8s_resource("caolo-web", resource_deps=["caolo-queen"], port_forwards=8000)
k8s_resource("caolo-rt", resource_deps=["caolo-queen"], port_forwards=8080)
