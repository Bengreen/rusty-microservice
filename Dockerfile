FROM rust:1.60.0 AS chef
RUN cargo install cargo-chef
# FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR app

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS build
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release

# We do not need the Rust toolchain to run the binary!
FROM debian:buster-slim AS runtime
WORKDIR app
COPY --from=build /app/target/release/uservice_run /usr/local/bin/
COPY --from=build /app/target/release/deps/libuservice.so /app/target/release/deps/libsample01.so /usr/local/lib/
RUN ldconfig
#COPY --from=builder /app/target/release/app /usr/local/bin
# ENTRYPOINT ["/usr/local/bin/uservice_run", "-l", "libuservice.so", "start"]
ENTRYPOINT ["/usr/local/bin/uservice_run"]
CMD ["-l", "sample01", "start"]
