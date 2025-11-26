FROM ghcr.io/fj0r/edap:chef AS builder
WORKDIR /app
# Build application
COPY . .
RUN cargo build --release --bin gateway
RUN set -eux \
  ; cd crates/ui \
  ; cat index.html | rg --passthru 'data-host=".+"' -r '' > index.html \
  ; dx build --web --release \
  \
  ; mkdir -p assets \
  ; cd assets \
  ; wget https://cdnjs.cloudflare.com/ajax/libs/mermaid/11.5.0/mermaid.min.js \
  ; wget https://cdnjs.cloudflare.com/ajax/libs/apexcharts/4.5.0/apexcharts.min.js \
  ; wget https://cdnjs.cloudflare.com/ajax/libs/apexcharts/4.5.0/apexcharts.min.css \
  ;

# We do not need the Rust toolchain to run the binary!
FROM debian:stable-slim AS runtime
WORKDIR /app

COPY --from=builder /app/target/release/gateway /app/gateway
COPY gateway.toml /app
COPY manifest /app/manifest

COPY --from=builder /app/target/dx/ui/release/web/public/ /app/static
COPY --from=builder /app/crates/ui/assets /app/static/assets
CMD ["/app/gateway"]
