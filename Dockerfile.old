FROM rust:1.65.0-bullseye AS builder
ARG SERVICE

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY services ./services
COPY hack ./hack

RUN cargo build --bin ${SERVICE} --release


FROM debian:bullseye-slim AS target
ARG SERVICE
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/${SERVICE} /app

RUN chmod a+x /app
CMD /app
