FROM scratch

# Build arguments for target platform
ARG TARGETARCH
ARG TARGETVARIANT

# Copy the pre-built binary based on the target architecture
# The binary should be built using build-cross.sh before building this image
COPY target/cross-compiled/podman-remote-${TARGETARCH}${TARGETVARIANT} /podman-remote

# Expose the default port (adjust if needed)
EXPOSE 3000

# Set the entrypoint to the binary
ENTRYPOINT ["/podman-remote"]
