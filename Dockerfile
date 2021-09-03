FROM rust:latest as build
WORKDIR /usr/src

RUN rustup target add x86_64-unknown-linux-musl

ARG APP_NAME=hello
RUN USER=root cargo new ${APP_NAME} && \
  touch ${APP_NAME}/src/lib.rs

WORKDIR /usr/src/${APP_NAME}
COPY Cargo.toml Cargo.lock ./
RUN cargo update && \
    cargo build --release --target x86_64-unknown-linux-musl

COPY src ./src/
COPY benches ./benches/
COPY examples ./examples/

RUN touch src/*

RUN cargo build --release --target x86_64-unknown-linux-musl
RUN strip target/x86_64-unknown-linux-musl/release/${APP_NAME}


FROM scratch
ARG APP_NAME=hello
COPY --from=build /usr/src/${APP_NAME}/target/x86_64-unknown-linux-musl/release/${APP_NAME} .

USER 1000

CMD ["/hello", "start"]

