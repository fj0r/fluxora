FROM rust:1 AS chef
# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN set -eux \
  ; cargo install cargo-chef \
  ; cargo install dioxus-cli \
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
