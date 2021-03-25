# build stage
FROM rust:latest as cargo-build

WORKDIR /usr/src/gico
COPY . .

RUN cargo install --path .
EXPOSE 9090

CMD ["gico"]
