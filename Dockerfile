# Multi-stage Dockerfile for HOOP
# Builder stage: compile Rust + build UI assets
FROM debian:bookworm-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl build-essential pkg-config libssl-dev ca-certificates \
    nodejs npm && rm -rf /var/lib/apt/lists/* \
    && npm install -g pnpm

# Install Rust via rustup with minimal profile for smaller image
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
ENV PATH="/root/.cargo/bin:${PATH}"
ENV CARGO_TERM_COLOR="always"
# Limit parallel jobs to reduce memory usage during builds
ENV CARGO_BUILD_JOBS=2

WORKDIR /workspace

# Copy manifests first for better layer caching
COPY Cargo.toml Cargo.lock ./
COPY hoop-cli/Cargo.toml hoop-cli/
COPY hoop-daemon/Cargo.toml hoop-daemon/
COPY hoop-schema/Cargo.toml hoop-schema/
COPY hoop-ui/Cargo.toml hoop-ui/
COPY hoop-mcp/Cargo.toml hoop-mcp/

# Create dummy src files for dependency caching
RUN mkdir -p hoop-cli/src hoop-daemon/src hoop-schema/src hoop-ui/src hoop-mcp/src
RUN echo "fn main() {}" > hoop-cli/src/main.rs
RUN echo "fn main() {}" > hoop-daemon/src/lib.rs
RUN echo "fn main() {}" > hoop-schema/src/lib.rs
RUN echo "fn main() {}" > hoop-ui/src/lib.rs
RUN echo "fn main() {}" > hoop-mcp/src/main.rs

# Build dependencies (cached layer)
RUN cargo build --release --bins && rm -rf target/release/deps/hoop* target/release/*.hoop*

# Copy actual source code
COPY hoop-ui/web hoop-ui/web
COPY hoop-ui/static hoop-ui/static
COPY hoop-cli/src hoop-cli/src
COPY hoop-daemon/src hoop-daemon/src
COPY hoop-schema/src hoop-schema/src
COPY hoop-schema/schemas hoop-schema/schemas
COPY hoop-ui/src hoop-ui/src
COPY hoop-mcp/src hoop-mcp/src

# Build UI assets
RUN cd hoop-ui/web && CI=true pnpm install --frozen-lockfile && pnpm run build

# Build binaries
RUN cargo build --release --bin hoop --bin hoop-mcp

# Runtime stage: distroless for minimal size
FROM gcr.io/distroless/cc-debian12

# Copy binaries
COPY --from=builder /workspace/target/release/hoop /hoop
COPY --from=builder /workspace/target/release/hoop-mcp /hoop-mcp

# Expose default port
EXPOSE 3000

# Set data volume for persistent state
VOLUME ["/root/.hoop"]

# Note: Healthcheck via HTTP /healthz endpoint (orchestration platforms should use this)
# Distroless doesn't include curl/wget, so container-level HEALTHCHECK is omitted.
# Kubernetes/other orchestrators can probe http://container:3000/healthz directly.

ENTRYPOINT ["/hoop"]
CMD ["serve", "--addr", "0.0.0.0:3000"]
