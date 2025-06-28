# syntax=docker/dockerfile:1

# BUILD ########################################################################
FROM rust:1.88-alpine3.22 AS build

WORKDIR /usr/local/src

COPY Cargo.toml Cargo.lock ./
COPY src src/
COPY examples examples/
COPY build.rs build.rs

RUN apk add --no-cache musl-dev
RUN cargo fetch
RUN cargo build --locked --release --example repl

# RUNTIME #####################################################################
FROM alpine:3.22

WORKDIR /usr/local/bin

COPY --from=build /usr/local/src/target/release/examples/repl /usr/local/bin/reqlang-expr-repl

# Request files can be mounted here
WORKDIR /usr/local/src

ENTRYPOINT ["/usr/local/bin/reqlang-expr-repl"]