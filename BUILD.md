# Cross-Compilation and Docker Build

This project uses cross-compilation to build binaries for multiple architectures, then packages them in minimal "from scratch" Docker images.

## Prerequisites

- [cross](https://github.com/cross-rs/cross) - Install with: `cargo install cross --git https://github.com/cross-rs/cross`
- Docker with buildx support for multi-arch builds

## Building

### Cross-Compile Binaries

Build binaries for all supported targets (x86_64, aarch64, armv7):

```bash
./build-cross.sh
```

This will create binaries in `target/cross-compiled/`:
- `podman-remote-x86_64-unknown-linux-musl`
- `podman-remote-aarch64-unknown-linux-musl`
- `podman-remote-armv7-unknown-linux-musleabihf`

### Build Docker Image

Build multi-architecture Docker image locally (amd64 only):

```bash
./build-docker.sh
```

Build and push multi-architecture image to registry:

```bash
./build-docker.sh --push your-registry/podman-remote:latest
```

## GitHub Actions

The project includes a GitHub Actions workflow (`.github/workflows/docker-build.yml`) that:

1. Cross-compiles binaries for all target architectures in parallel
2. Strips binaries to reduce size
3. Builds multi-arch Docker images without QEMU emulation
4. Pushes to GitHub Container Registry (ghcr.io)

The workflow runs on:
- Push to main branch
- Version tags (v*)
- Pull requests (build only, no push)
- Manual trigger (workflow_dispatch)

## Supported Architectures

- `linux/amd64` (x86_64)
- `linux/arm64` (aarch64)
- `linux/arm/v7` (armv7)

## Why This Approach?

Traditional Docker builds with QEMU emulation are extremely slow for Rust projects (often 10-30x slower). By cross-compiling natively and using pre-built binaries, we get:

- ⚡ **Much faster builds** - Native compilation instead of QEMU emulation
- 🪶 **Smaller images** - Using `FROM scratch` with only the binary (~10MB vs 100MB+)
- 🔄 **Better CI/CD** - Parallel builds for all architectures
- 🎯 **Better caching** - Rust compilation caches work properly

## Manual Docker Build

If you want to build for a specific architecture manually:

```bash
# Cross-compile for specific target
cross build --release --target x86_64-unknown-linux-musl

# Build Docker image for that architecture
docker build -f Dockerfile.multi-arch --build-arg TARGETARCH=amd64 -t podman-remote:amd64 .
```
