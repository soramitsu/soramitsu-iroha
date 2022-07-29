ARG BASE_IMAGE=ubuntu:22.04
FROM $BASE_IMAGE AS rust-base

ENV CARGO_HOME=/cargo_home \
    RUSTUP_HOME=/rustup_home \
    DEBIAN_FRONTEND=noninteractive
ENV PATH="$CARGO_HOME/bin:$PATH"

RUN set -ex; \
    apt-get update  -yq; \
    apt-get install -y --no-install-recommends curl apt-utils; \
    apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        libssl-dev \
        clang \
        pkg-config \
        llvm-dev; \
    rm -rf /var/lib/apt/lists/*

ARG TOOLCHAIN=stable
RUN set -ex; \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs >/tmp/rustup.sh; \
    sh /tmp/rustup.sh -y --no-modify-path --default-toolchain "$TOOLCHAIN"; \
    rm /tmp/*.sh

RUN set -ex; \
    rustup toolchain install --profile default nightly-2022-04-20; \
    rustup target add wasm32-unknown-unknown; \
    rustup component add rust-src


FROM rust-base as builder
ARG PROFILE
WORKDIR /iroha
COPY . .
RUN cargo build $PROFILE --workspace

FROM $BASE_IMAGE
ARG CONFIG_DIR=config
RUN mkdir -p $CONFIG_DIR
ARG BIN=iroha
ARG TARGET_DIR=debug
COPY --from=builder /iroha/target/$TARGET_DIR/$BIN .
RUN apt-get update -yq; \
    apt-get install -y --no-install-recommends libssl-dev; \
    rm -rf /var/lib/apt/lists/*
ENV IROHA_TARGET_BIN=$BIN
ENV IROHA2_CONFIG_PATH=$CONFIG_DIR/config.json
ENV IROHA2_GENESIS_PATH=$CONFIG_DIR/genesis.json
CMD ./$IROHA_TARGET_BIN
