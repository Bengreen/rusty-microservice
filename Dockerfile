# Notes on using cargo chef to build inside docker: https://github.com/LukeMathWalker/cargo-chef
FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release

# We do not need the Rust toolchain to run the binary!
FROM debian:buster-slim AS runtime
WORKDIR app
COPY --from=builder /app/target/release/uservice_run /usr/local/bin/
COPY --from=builder /app/target/release/deps/libuservice.so /app/target/release/deps/libsample01.so /usr/local/lib/
RUN ldconfig
#COPY --from=builder /app/target/release/app /usr/local/bin
ENTRYPOINT ["/usr/local/bin/uservice_run", "-l", "libuservice.so", "start"]
