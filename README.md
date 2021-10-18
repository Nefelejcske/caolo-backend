# Cao-Lo backend

This repository contains the back-end code of the game Cao-Lo.

Code layout:

```txt
|- k8s/                 # Kubernetes manifests
|- db/migrations/       # SQL schema migrations
|- protos/              # Protobuf messages used in communications between web and worker services
|- api/                 # Webservice bridging remote clients and the sim
|- rt/                  # Real-time communications service
|- sim/
 |+ cao-storage-derive/ # Derive macro for the storage of the simulation/
 |+ simulation/         # Library for running the game world
 |+ worker/             # Executable code running the simulation and interfacing
```

## Deploying via Tilt

### Requirements

-   [Tilt](https://tilt.dev)
-   [Helm](https://helm.sh/)
-   [kubectl](https://kubernetes.io/docs/tasks/tools/)
-   [Docker](https://www.docker.com/)

### Configuring your kubernetes cluster

**TBA**

### Local development via [Kind](https://kind.sigs.k8s.io/)

-   Install [Kind](https://kind.sigs.k8s.io/docs/user/quick-start/#installation)
-   Install [ctlptl](https://github.com/tilt-dev/ctlptl#kind-with-a-built-in-registry-at-a-random-port) (Optional, but recommended)

```
ctlptl create cluster kind --registry=ctlptl-registry
tilt up
```

## Running tests

```
make test
```
