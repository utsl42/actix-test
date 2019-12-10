FROM rust:1.39.0 AS build-rs
WORKDIR /usr/src

RUN cargo install cargo-build-deps
RUN USER=root cargo new actix-test
WORKDIR /usr/src/actix-test
RUN rm src/main.rs
COPY Cargo.toml Cargo.lock build.rs Makefile ./
COPY src/ src/
RUN cargo build-deps --release
RUN ls -R .
RUN cargo install --path .

FROM node:13-buster AS build-frontend
WORKDIR /usr/src
COPY Makefile ./
RUN mkdir frontend
COPY frontend/*.json frontend/webpack.config.js ./frontend/
RUN make prep
COPY frontend/src/ ./frontend/src/
RUN make build

FROM debian:buster-slim
WORKDIR /
COPY --from=build-rs /usr/local/cargo/bin/server .
COPY --from=build-frontend /usr/src/frontend/dist /frontend/dist
COPY countries.json .
USER 1000
CMD ["./server"]
EXPOSE 63333
