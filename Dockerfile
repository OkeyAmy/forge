# ============================================================
# Stage 1 — Builder
# ============================================================
FROM rust:slim-bookworm AS builder

# System deps needed to compile (openssl for reqwest, pkg-config)
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy the whole workspace
COPY . .

# Build the forge binary in release mode
RUN cargo build --release -p forge

# ============================================================
# Stage 2 — Runtime (minimal)
# ============================================================
FROM debian:bookworm-slim AS runtime

# Runtime deps: ca-certificates (for HTTPS), libssl (for reqwest TLS)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    git \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m -u 1000 forge

# Copy the compiled binary
COPY --from=builder /build/target/release/forge /usr/local/bin/forge

# Trajectories will be written here — mount a volume over it
RUN mkdir -p /trajectories && chown forge:forge /trajectories

USER forge
WORKDIR /home/forge

ENTRYPOINT ["forge"]
CMD ["--help"]
