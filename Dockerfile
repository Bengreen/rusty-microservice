
FROM rust:1.60.0 AS build
# COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
# RUN cargo chef cook --release --recipe-path recipe.json

RUN cargo new --bin uservice_run && \
  mv uservice_run/* . && \
  rmdir uservice_run

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
# COPY ./uservice ./uservice
# COPY ./sample01 ./sample01
# COPY ./ffi-log2 ./ffi-log2
# COPY ./src ./src


# Build application
COPY . .
RUN rm ./target/release/deps/uservice_run*
RUN cargo build --release




# We do not need the Rust toolchain to run the binary!
FROM debian:buster-slim AS runtime
WORKDIR app
COPY --from=build /app/target/release/uservice_run /usr/local/bin/
COPY --from=build /app/target/release/deps/libuservice.so /app/target/release/deps/libsample01.so /usr/local/lib/
RUN ldconfig
#COPY --from=builder /app/target/release/app /usr/local/bin
ENTRYPOINT ["/usr/local/bin/uservice_run", "-l", "libuservice.so", "start"]
