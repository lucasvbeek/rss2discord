FROM clux/muslrust:stable AS chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
WORKDIR /app
COPY . /app
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
WORKDIR /app
COPY --from=planner /app/recipe.json /app/recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . /app
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM gcr.io/distroless/static:latest
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/rss2discord /usr/local/bin/
CMD ["/usr/local/bin/rss2discord", "-c", "/etc/rss2discord/config.yaml"]