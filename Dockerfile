# build stage
FROM rust:latest as cargo-build

RUN apt-get update && apt-get install musl-tools libssl-dev build-essential -y
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/app
COPY . .

RUN RUSTFLAGS=-Clinker=musl-gcc cargo build --release --target=x86_64-unknown-linux-musl

FROM alpine:latest

RUN addgroup -g 1000 app
RUN adduser -D -s /bin/sh -u 1000 -G app app

WORKDIR /home/app/bin/
COPY --from=cargo-build /usr/src/app/target/x86_64-unknown-linux-musl/release/gico .

RUN chown app:app gico
USER app

EXPOSE 9090

CMD ["./gico"]
