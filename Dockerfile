FROM rust:latest as build
WORKDIR /usr/src

RUN rustup target add x86_64-unknown-linux-musl

ARG APP_NAME=uservice_run
RUN USER=root cargo new ${APP_NAME} && \
  touch ${APP_NAME}/src/lib.rs

WORKDIR /usr/src/${APP_NAME}
COPY Cargo.toml Cargo.lock ./
RUN cargo update && \
    cargo build --release --target x86_64-unknown-linux-musl

COPY benches ./benches/
COPY ffi-log2 ./ffi-log2/
COPY hello ./hello/
COPY sample01 ./sample01/
COPY src ./src/
COPY uservice ./uservice/

RUN touch src/*

RUN cargo build --release --target x86_64-unknown-linux-musl
RUN strip target/x86_64-unknown-linux-musl/release/${APP_NAME}


FROM scratch
ARG APP_NAME=hello
COPY --from=build /usr/src/${APP_NAME}/target/x86_64-unknown-linux-musl/release/${APP_NAME} .

USER 1000

CMD ["/hello", "start"]
