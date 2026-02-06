# Stage 1: Build frontend
FROM node:22-alpine AS frontend-builder
WORKDIR /app

# Install pnpm
RUN corepack enable && corepack prepare pnpm@latest --activate

# Copy package files
COPY package.json pnpm-lock.yaml ./

# Install dependencies
RUN pnpm install --frozen-lockfile

# Copy frontend source
COPY src/ src/
COPY index.html vite.config.ts tsconfig.json tsconfig.node.json ./

# Build for web mode (not Tauri)
RUN pnpm build

# Stage 2: Build Rust server
FROM rust:bookworm AS rust-builder
WORKDIR /app

# Install dependencies for SQLite and OpenSSL
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/

# Create dummy src-tauri to satisfy workspace
RUN mkdir -p src-tauri/src && \
    echo '[package]\nname = "boke"\nversion = "0.1.0"\nedition = "2021"\n\n[lib]\nname = "boke_lib"\ncrate-type = ["rlib"]\n\n[dependencies]\nboke-core = { path = "../crates/boke-core", features = ["sqlite"] }' > src-tauri/Cargo.toml && \
    echo 'pub fn dummy() {}' > src-tauri/src/lib.rs

# Build release binary
RUN cargo build --release --package boke-server

# Stage 3: Runtime image
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy built artifacts
COPY --from=rust-builder /app/target/release/boke-server /app/boke-server
COPY --from=frontend-builder /app/dist /app/static

# Create non-root user
RUN useradd -r -s /bin/false boke && \
    mkdir -p /data && \
    chown -R boke:boke /app /data
USER boke

# Environment variables
ENV BIND_ADDRESS=0.0.0.0:8080
ENV STATIC_DIR=/app/static
ENV DATABASE_URL=sqlite:///data/boke.db
ENV RUST_LOG=info

# Volume for SQLite data persistence
VOLUME ["/data"]

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/feeds || exit 1

CMD ["/app/boke-server"]
