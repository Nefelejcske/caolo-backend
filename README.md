# Cao-Lo backend

This repository contains the back-end code of the game Cao-Lo.

Code layout:

```txt
|- db/migrations/       # SQL schema migrations
|- protos/              # Protobuf messages used in communications between web and worker services
|- api/                 # Webservice bridging remote clients and the sim
|- rt/                  # Real-time communications service
|- sim/
 |+ cao-storage-derive/ # Derive macro for the storage of the simulation/
 |+ simulation/         # Library for running the game world
 |+ worker/             # Executable code running the simulation and interfacing
```

## Building via Skaffold

### Requirements

-   [Skaffold](https://skaffold.dev/docs/install/)
-   [kubectl](https://kubernetes.io/docs/tasks/tools/)
-   [Docker](https://www.docker.com/)

### Build and push the image

```
skaffold build
```

### Configuring your kubernetes cluster

**TBA**

### Build & Deploy

```
skaffold run
```

## Running tests

```
make test
```
