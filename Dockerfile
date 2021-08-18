FROM rust:latest as build
WORKDIR /usr/src

RUN rustup target add x86_64-unknown-linux-musl

ARG APP_NAME=rust_hello
RUN USER=root cargo new ${APP_NAME}

WORKDIR /usr/src/${APP_NAME}
COPY Cargo.toml Cargo.lock ./
RUN cargo update && \
    cargo build --release
# Not sure if having cargo build here does provide any cacing or not

COPY src ./src/

RUN cargo build --release

RUN cargo install --target x86_64-unknown-linux-musl --path .

FROM scratch
COPY --from=build /usr/local/cargo/bin/${APP_NAME} .
USER 1000

CMD ["/rust_hello", "start"]

