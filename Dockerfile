FROM rust:1.62.1 AS builder

WORKDIR /complier/simple-reverse-proxy

RUN apt update -y
RUN apt install musl-tools -y

# TARGET=x86_64-unknown-linux-musl
# TARGET=aarch64-unknown-linux-musl
ARG TARGET=x86_64-unknown-linux-musl

RUN rustup target add ${TARGET}

COPY . .

RUN cargo install --target ${TARGET} --path .

FROM scratch
COPY --from=builder /usr/local/cargo/bin/simple-reverse-proxy .
COPY ./config.properties .
CMD ["./simple-reverse-proxy"]