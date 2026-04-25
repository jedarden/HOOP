FROM debian:bookworm AS builder

RUN apt-get update && apt-get install -y \
    curl build-essential pkg-config libssl-dev \
    nodejs npm && rm -rf /var/lib/apt/lists/* \
    && npm install -g pnpm

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /workspace
COPY . .

RUN cd hoop-ui/web && pnpm install --frozen-lockfile && pnpm run build

RUN cargo build --release --bin hoop --bin hoop-mcp

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /workspace/target/release/hoop /usr/local/bin/hoop
COPY --from=builder /workspace/target/release/hoop-mcp /usr/local/bin/hoop-mcp

EXPOSE 3000

ENTRYPOINT ["/usr/local/bin/hoop"]
