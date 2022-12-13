# Step -1: Grab mold
FROM lukemathwalker/cargo-chef as rust-base
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends curl clang
ARG MOLD_VERSION=1.7.1
RUN curl -sSL https://github.com/rui314/mold/releases/download/v${MOLD_VERSION}/mold-${MOLD_VERSION}-x86_64-linux.tar.gz | tar xzv && \
    mv mold-${MOLD_VERSION}-x86_64-linux/bin/mold /mold && \
    rm -rf mold-${MOLD_VERSION}-x86_64-linux
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=clang
ENV RUSTFLAGS="-C link-arg=-fuse-ld=/mold"

# Step 1: Compute a recipe file
FROM rust-base as planner
ARG SERVICE
COPY . .
RUN cargo chef prepare --recipe-path recipe.json --bin ${SERVICE}

# Step 2: Cache project dependencies
FROM rust-base as cacher
ARG SERVICE
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --bin ${SERVICE}

# Step 3: Build the binary
FROM rust-base as builder
COPY . .
# Copy over the cached dependencies from above
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
ARG SERVICE
RUN cargo build --release --bin ${SERVICE}

# Step 4:
# Create a tiny output image.
# It only contains our final binary.
FROM debian:bullseye-slim as runtime
ARG SERVICE
COPY --from=builder /app/target/release/${SERVICE} /app
ENTRYPOINT ["/app"]