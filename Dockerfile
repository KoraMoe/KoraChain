FROM rust:latest as builder

# Install protobuf compiler and other build dependencies
RUN apt-get update && \
    apt-get install -y \
        protobuf-compiler \
        pkg-config \
        libssl-dev \
        build-essential \
        clang \
        && rm -rf /var/lib/apt/lists/*

WORKDIR /polkadot
COPY . /polkadot

ENV RUSTFLAGS="-C target-feature=-crt-static -C link-arg=-Wl,--build-id=none"
ENV SOURCE_DATE_EPOCH=1600000000
ENV CARGO_PROFILE_RELEASE_DEBUG=0
ENV CARGO_PROFILE_PRODUCTION_DEBUG=0
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
ENV CARGO_BUILD_JOBS=1

RUN cargo fetch
RUN cargo build --locked --profile production
RUN strip target/production/kora-chain-node

# Runtime stage with minimal Ubuntu image
FROM ubuntu:22.04

# Install minimal runtime dependencies
RUN apt-get update && \
    apt-get install -y \
        ca-certificates \
        && rm -rf /var/lib/apt/lists/* \
        && apt-get autoremove -y \
        && apt-get clean

COPY --from=builder /polkadot/target/production/kora-chain-node /usr/local/bin

USER root
RUN useradd -m -u 1001 -U -s /bin/sh -d /polkadot polkadot && \
	mkdir -p /data /polkadot/.local/share && \
	chown -R polkadot:polkadot /data && \
	ln -s /data /polkadot/.local/share/polkadot && \
# unclutter and minimize the attack surface
	rm -rf /usr/bin /usr/sbin && \
# check if executable works in this container
	/usr/local/bin/kora-chain-node --version

USER polkadot

EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/kora-chain-node"]
