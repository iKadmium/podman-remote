#!/usr/bin/env bash
set -euo pipefail

# Build and push multi-architecture Docker image using pre-compiled binaries
# Usage: ./build-docker.sh [--push] [TAG]

PUSH=false
TAG="${1:-podman-remote:latest}"

if [[ "$1" == "--push" ]]; then
    PUSH=true
    TAG="${2:-podman-remote:latest}"
fi

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}Building multi-architecture Docker image...${NC}"

# Check if binaries exist
if [ ! -d "target/cross-compiled" ] || [ -z "$(ls -A target/cross-compiled 2>/dev/null)" ]; then
    echo -e "${YELLOW}Cross-compiled binaries not found. Running build-cross.sh first...${NC}"
    ./build-cross.sh
fi

# Build the multi-arch image
if [ "$PUSH" = true ]; then
    echo -e "${GREEN}Building and pushing multi-arch image: ${TAG}${NC}"
    docker buildx build \
        --platform linux/amd64,linux/arm64,linux/arm/v7 \
        --tag "$TAG" \
        --file Dockerfile.multi-arch \
        --push \
        .
else
    echo -e "${GREEN}Building multi-arch image: ${TAG}${NC}"
    echo -e "${YELLOW}Note: Building for local use (amd64 only). Use --push to build and push all architectures.${NC}"
    docker buildx build \
        --platform linux/amd64 \
        --tag "$TAG" \
        --file Dockerfile.multi-arch \
        --load \
        .
fi

echo -e "${GREEN}✓ Docker image built successfully!${NC}"
