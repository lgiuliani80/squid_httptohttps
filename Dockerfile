# Stage 1: Build the Rust URL rewriter
FROM rust:1.77-bookworm AS builder
WORKDIR /build
COPY Cargo.toml ./
COPY src/ ./src/
RUN cargo build --release

# Stage 2: Squid proxy with the compiled rewriter
FROM ubuntu/squid:latest
COPY --from=builder /build/target/release/https-to-http /https-to-http
RUN chmod +x /https-to-http
COPY squid.conf /etc/squid/squid.conf
EXPOSE 3128
