#!/usr/bin/env bash
set -euo pipefail

# Cross-compile Rust binary for multiple targets
# This script uses cross to build for different platforms

TARGETS=(
    "x86_64-unknown-linux-musl"
    "aarch64-unknown-linux-musl"
    "armv7-unknown-linux-musleabihf"
)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Starting cross-compilation for multiple targets...${NC}"

# Check if cross is installed
if ! command -v cross &> /dev/null; then
    echo -e "${YELLOW}cross is not installed. Installing...${NC}"
    cargo install cross --git https://github.com/cross-rs/cross
fi

# Create output directory
OUTPUT_DIR="./target/cross-compiled"
mkdir -p "$OUTPUT_DIR"

# Build for each target
for TARGET in "${TARGETS[@]}"; do
    echo -e "\n${GREEN}Building for ${TARGET}...${NC}"
    
    if cross build --release --target "$TARGET"; then
        # Copy the binary to output directory with target name
        BINARY_NAME="podman-remote-${TARGET}"
        cp "target/${TARGET}/release/podman-remote" "${OUTPUT_DIR}/${BINARY_NAME}"
        
        # Strip the binary to reduce size
        echo -e "${GREEN}Stripping ${BINARY_NAME}...${NC}"
        case "$TARGET" in
            x86_64*)
                strip "${OUTPUT_DIR}/${BINARY_NAME}" || true
                ;;
            aarch64*)
                if command -v aarch64-linux-gnu-strip &> /dev/null; then
                    aarch64-linux-gnu-strip "${OUTPUT_DIR}/${BINARY_NAME}" || true
                fi
                ;;
            armv7*)
                if command -v arm-linux-gnueabihf-strip &> /dev/null; then
                    arm-linux-gnueabihf-strip "${OUTPUT_DIR}/${BINARY_NAME}" || true
                fi
                ;;
        esac
        
        # Show file size
        SIZE=$(du -h "${OUTPUT_DIR}/${BINARY_NAME}" | cut -f1)
        echo -e "${GREEN}✓ Built ${BINARY_NAME} (${SIZE})${NC}"
    else
        echo -e "${RED}✗ Failed to build for ${TARGET}${NC}"
        exit 1
    fi
done

echo -e "\n${GREEN}All builds completed successfully!${NC}"
echo -e "${GREEN}Binaries are in: ${OUTPUT_DIR}${NC}"
ls -lh "$OUTPUT_DIR"
