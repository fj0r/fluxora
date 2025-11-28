FROM ghcr.io/fj0r/fluxora:chef AS builder
WORKDIR /app
# Build application
COPY . .
RUN cargo build --release --bin chat


FROM debian:stable-slim AS runtime
WORKDIR /app

COPY --from=builder /app/target/release/chat /app/chat
COPY chat.toml /app
COPY manifest /app/manifest
CMD ["/app/chat"]
