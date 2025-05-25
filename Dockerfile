FROM docker.io/paritytech/ci-unified:latest as builder

# Install protobuf compiler required by litep2p
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

WORKDIR /polkadot
COPY . /polkadot

ENV RUSTFLAGS="-C target-feature=-crt-static"
ENV SOURCE_DATE_EPOCH=1600000000
ENV CARGO_PROFILE_RELEASE_DEBUG=0
ENV CARGO_PROFILE_PRODUCTION_DEBUG=0

# Fix rustup toolchain issues and ensure we have the right components
RUN rustup toolchain install stable --profile minimal --component rustfmt,clippy
RUN rustup target add wasm32-unknown-unknown --toolchain stable
RUN rustup default stable

RUN cargo fetch
RUN cargo build --locked --profile production

FROM docker.io/parity/base-bin:latest

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
