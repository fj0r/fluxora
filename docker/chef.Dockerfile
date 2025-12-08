FROM rust:1 AS chef
# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN set -eux \
  ; curl -fsSL https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz \
    | tar zxf - -C /usr/local/bin/ \
    ; chmod +x /usr/local/bin/cargo-binstall \
  ; cargo binstall -y cargo-chef dioxus-cli \
  ; apt update \
  ; apt-get install -y --no-install-recommends \
        ripgrep cmake \
  ;

FROM chef AS planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
