# KoraChain

Sign a Contract with Me!! KoraChain is a zero-knowledge powered application ecosystem.

## Introduction

KoraChain is an application chain developed by Kora Lab. Using zero-knowledge libraries such as circom and zkvm, we make practical and real-world applications possible.

KoraChain is based on Substrate (Polkadot-SDK). It serves as a testground for protocols which requires both zero-knowledge and token-system integration.

## Features

- Zero-knowledge powered application ecosystem (through Pallets and Contracts)
- Complete Governance capabilities
- Substrate-based blockchain implementation (compatible with Polkadot)

## Architecture

### Components

- **Node**: The main blockchain node executable.
- **Runtime**: The blockchain runtime logic (WASM Executable, Upgradable).

### Pallets

- **Contracts**: Execute WebAssembly (WASM) or EVM contracts on-chain to support applications.
- **Verifier**: Verify zero-knowledge proofs on-chain with special pallets which can be used by applications.

## Quick Start

Visit https://koranet.work/ for more information.

## Prerequisites

### System Requirements

- **OS**: Linux, macOS, or Windows
- **Memory**: Minimum 4GB RAM, 8GB+ recommended
- **Storage**: At least 100GB free space for blockchain data and builds
- **CPU**: Multi-core processor (4+ cores recommended)

### Development Dependencies

### Ubuntu/Debian

```bash
sudo apt update
sudo apt install -y git clang curl libssl-dev llvm libudev-dev
```

### Arch Linux

```bash
pacman -Syu --needed --noconfirm curl git clang
```

### Fedora

```bash
sudo dnf update
sudo dnf install clang curl git openssl-devel
```

### OpenSUSE

```bash
sudo zypper install clang curl git openssl-devel llvm-devel libudev-devel
```

### macOS

```bash
brew update
brew install openssl
```

#### Rust Toolchain
```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# The project uses stable Rust with specific targets
rustup default stable
rustup update
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
```

## Compilation

### Quick Start

1. **Clone the repository:**
```bash
git clone https://github.com/KoraMoe/KoraChain.git
cd KoraChain
```

2. **Verify Rust toolchain:**
```bash
rustc --version  # Should show stable version
cargo --version
rustup show      # Verify wasm32-unknown-unknown target is installed
```

3. **Build the project:**
```bash
# Development build (faster compilation, debugging enabled)
cargo build

# Release build (optimized, production-ready)
cargo build --release

# Production build (maximum optimization, deterministic)
cargo build --profile production
```

### Build Components

The workspace includes several key components:

1. **Node Binary** (`node/`): The main blockchain node executable
2. **Runtime** (`runtime/`): The blockchain runtime logic (compiled to WASM)
3. **Pallets** (`pallets/`): Custom blockchain modules

#### Building Specific Components

```bash
# Build only the node binary
cargo build --release -p kora-chain-node

# Build only the runtime
cargo build --release -p kora-chain-runtime

# Build runtime with on-chain release features
cargo build --release -p kora-chain-runtime --features on-chain-release-build
```

### Verification

After building, verify the binary:

```bash
# Check the binary exists and is executable
ls -la target/release/kora-chain-node
./target/release/kora-chain-node --version

# Run basic checks
./target/release/kora-chain-node --help
```

## Testing

### Unit Tests
```bash
# Run all tests
cargo test

# Run tests for a specific package
cargo test -p kora-chain-runtime
cargo test -p pallet-template

# Run tests with output
cargo test -- --nocapture
```

### Integration Tests
```bash
# Run benchmarks (if available)
cargo test --features runtime-benchmarks

# Run with try-runtime features
cargo test --features try-runtime
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Check for common issues
cargo clippy --all-targets --all-features
```

## Docker Deployment

### Building Docker Image

```bash
# Build the Docker image
docker build -t ghcr.io/koramoe/korachain:latest .

# Build with specific tag
docker build -t kora-chain:v1.0.0 .

# Build with build arguments for reproducible builds
docker build \
  --build-arg RUSTFLAGS="-C target-feature=-crt-static" \
  --build-arg SOURCE_DATE_EPOCH=1600000000 \
  -t kora-chain:reproducible .
```

### Running Docker Container

#### Development Setup
```bash
# Run with temporary data (data lost on restart)
docker run -it --rm \
  -p 30333:30333 \
  -p 9933:9933 \
  -p 9944:9944 \
  -p 9615:9615 \
  ghcr.io/koramoe/korachain:latest \
  --dev

# Run with persistent data volume (using host directory)
# First create and set permissions for the data directory
mkdir -p ./data
sudo chown -R 1001:1001 ./data  # or chmod 777 ./data

docker run -d \
  --name kora-chain-dev \
  -p 30333:30333 \
  -p 9933:9933 \
  -p 9944:9944 \
  -p 9615:9615 \
  -v $(pwd)/data:/data \
  ghcr.io/koramoe/korachain:latest \
  --dev \
  --base-path /data
```

#### Production Setup
```bash
# Create and set permissions for the data directory
mkdir -p ./data
sudo chown -R 1001:1001 ./data  # or chmod 777 ./data

# Run production container
docker run -d \
  --name kora-chain-prod \
  --restart unless-stopped \
  -p 30333:30333 \
  -p 9944:9944 \
  -p 9615:9615 \
  -v $(pwd)/data:/data \
  ghcr.io/koramoe/korachain:latest \
  --base-path /data \
  --chain chanto \
  --name "Alice" \
  --rpc-external \
  --rpc-cors all \
  --prometheus-external
```

### Docker Compose Setup

1. **Create the data directory with correct permissions:**
```bash
# Create the data directory
mkdir -p ./data

# Set ownership to uid 1001 (polkadot user in container)
sudo chown -R 1001:1001 ./data

# Alternatively, if you don't have sudo access, make it world-writable
chmod 777 ./data
```

2. **Create `docker-compose.yml`:**

```yaml
version: '3.8'

services:
  kora-chain:
    image: ghcr.io/koramoe/korachain:latest
    container_name: kora-chain
    restart: unless-stopped
    ports:
      - "30333:30333"  # P2P port
      - "9933:9933"    # HTTP RPC
      - "9944:9944"    # WebSocket RPC
      - "9615:9615"    # Prometheus metrics
    volumes:
      - ./data:/data
    command: [
      "--base-path", "/data",
      "--chain", "chanto",
      "--name", "Alice",
      "--rpc-external",
      "--rpc-cors", "all",
      "--prometheus-external"
    ]
    environment:
      - RUST_LOG=info
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:9933/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

3. **Run the container:**
```bash
docker-compose up -d
```

## Manual Deployment

### Binary Deployment

1. **Prepare the system:**
```bash
# Create dedicated user
sudo useradd -m -u 1001 -U -s /bin/bash -d /home/kora kora

# Create data directory
sudo mkdir -p /opt/kora-chain/data
sudo chown kora:kora /opt/kora-chain/data
```

2. **Deploy the binary:**
```bash
# Copy binary to system location
sudo cp target/production/kora-chain-node /usr/local/bin/
sudo chmod +x /usr/local/bin/kora-chain-node

# Verify installation
/usr/local/bin/kora-chain-node --version
```

3. **Create systemd service:**

Create `/etc/systemd/system/kora-chain.service`:

```ini
[Unit]
Description=Kora Chain Node
After=network.target
StartLimitIntervalSec=0

[Service]
Type=simple
Restart=always
RestartSec=1
User=kora
Group=kora
ExecStart=/usr/local/bin/kora-chain-node \
  --base-path /opt/kora-chain/data \
  --chain chanto \
  --name "Alice" \
  --rpc-external \
  --rpc-cors all \
  --prometheus-external
Environment=RUST_LOG=info
WorkingDirectory=/opt/kora-chain
StandardOutput=journal
StandardError=journal
SyslogIdentifier=kora-chain

[Install]
WantedBy=multi-user.target
```

4. **Start the service:**
```bash
# Reload systemd and start service
sudo systemctl daemon-reload
sudo systemctl enable kora-chain
sudo systemctl start kora-chain

# Check status
sudo systemctl status kora-chain

# View logs
sudo journalctl -u kora-chain -f
```

## Network Configuration

### Port Requirements

- **30333**: P2P networking (required for node communication)
- **9933**: HTTP RPC (optional, for RPC calls)
- **9944**: WebSocket RPC (optional, for real-time subscriptions)
- **9615**: Prometheus metrics (optional, for monitoring)

### Firewall Configuration

```bash
# Ubuntu/Debian with UFW
sudo ufw allow 30333/tcp
sudo ufw allow 9944/tcp  # Only if RPC access needed
sudo ufw allow 9615/tcp  # Only for monitoring

# CentOS/RHEL with firewalld
sudo firewall-cmd --permanent --add-port=30333/tcp
sudo firewall-cmd --permanent --add-port=9944/tcp  # Optional
sudo firewall-cmd --permanent --add-port=9615/tcp  # Optional
sudo firewall-cmd --reload
```

## Monitoring and Maintenance

### Health Checks

```bash
# Check node status via RPC
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_health", "params":[]}' \
  http://localhost:9933/

# Check sync status
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_syncState", "params":[]}' \
  http://localhost:9933/
```

### Log Management

```bash
# View recent logs
sudo journalctl -u kora-chain --since "1 hour ago"

# Follow logs in real-time
sudo journalctl -u kora-chain -f

# View logs with specific priority
sudo journalctl -u kora-chain -p err
```

### Database Maintenance

```bash
# Stop the node
sudo systemctl stop kora-chain

# Purge chain data (WARNING: This deletes all blockchain data)
/usr/local/bin/kora-chain-node purge-chain \
  --base-path /opt/kora-chain/data \
  --chain /opt/kora-chain/chain-specs/mainnet.json

# Restart the node
sudo systemctl start kora-chain
```

## Support and Resources

- **GitHub Repository**: https://github.com/KoraMoe/KoraChain.git
- **Homepage**: https://kora.moe
- **License**: Apache-2.0

For additional support, please check the GitHub issues or create a new issue with detailed information about your problem.