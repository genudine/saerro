FROM rust:1.65.0-alpine AS builder
ARG SERVICE

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY services ./services

RUN cargo build --bin ${SERVICE} --release


FROM alpine AS target
ARG SERVICE

COPY --from=builder /app/target/release/${SERVICE} /app

RUN chmod a+x /app
CMD /app
