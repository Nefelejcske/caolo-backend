# ============= planner ============================================================
# later stages may use these cached layers
FROM lukemathwalker/cargo-chef:latest AS planner
RUN rustup update
RUN rustup self update

WORKDIR /caolo
COPY sim/.cargo/ sim/.cargo/
COPY sim/rust-toolchain.toml sim/rust-toolchain.toml
COPY ./protos/ ./protos/
COPY ./sim/ ./sim/

WORKDIR /caolo/sim
RUN cargo chef prepare --recipe-path recipe.json

# ============= cache dependencies ============================================================

FROM lukemathwalker/cargo-chef:latest AS deps
RUN rustup update
RUN rustup self update

RUN apt-get update
RUN apt-get install lld clang libc-dev pkgconf -y

WORKDIR /caolo
COPY sim/.cargo/ sim/.cargo/
COPY sim/rust-toolchain.toml sim/rust-toolchain.toml
# NOTE that chef cook and cargo build have to be executed from the same working directory!
WORKDIR /caolo/sim
COPY --from=planner $CARGO_HOME $CARGO_HOME
COPY --from=planner /caolo/sim/recipe.json recipe.json
# cache the toolchain
RUN cargo --version 
RUN cargo chef cook --release --no-default-features --recipe-path recipe.json

# ==============================================================================================

# note: we don't use cargo-chef in this image, just making sure we use the same rust compiler version
FROM lukemathwalker/cargo-chef:latest AS build
RUN rustup update
RUN rustup self update

RUN apt-get update
RUN apt-get install lld clang libc-dev pkgconf protobuf-compiler -y

WORKDIR /caolo

# copy the cache
COPY --from=deps $CARGO_HOME $CARGO_HOME
COPY --from=deps /caolo/sim/target ./sim/target
COPY --from=deps /caolo/sim/Cargo.lock ./sim/Cargo.lock

COPY sim/.cargo/ sim/.cargo/
COPY sim/rust-toolchain.toml sim/rust-toolchain.toml
# cache the version
RUN cargo --version 
RUN protoc --version

COPY ./protos/ ./protos/
COPY ./sim/ ./sim/

WORKDIR /caolo/sim
RUN cargo build --release --no-default-features

# ========== Copy the built binary to a new container, to minimize the image size ==========

FROM ubuntu:18.04
WORKDIR /caolo

RUN apt-get update -y
RUN apt-get install openssl -y

COPY --from=build /caolo/sim/target/release/caolo-worker ./caolo-worker

ENTRYPOINT [ "./caolo-worker" ]
