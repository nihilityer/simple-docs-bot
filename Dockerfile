FROM rust:alpine3.21 AS builder

RUN apk add --update --no-cache gcc make openssl perl clang-dev musl-dev yaml curl libmicrohttpd libuuid mariadb-connector-c-dev mariadb-dev
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /opt/app
COPY . .
RUN cargo build --release --target=x86_64-unknown-linux-musl
CMD [ "sh" ]

FROM alpine:3.21
RUN apk add --update --no-cache openssl tzdata git mariadb-connector-c-dev mariadb-dev && rm -rf /var/cache/apk/*
RUN ln -sf /usr/share/zoneinfo/Asia/Shanghai /etc/localtime
RUN echo "Asia/Shanghai" > /etc/timezone
WORKDIR /docs
VOLUME /docs
COPY --from=builder /opt/app/target/x86_64-unknown-linux-musl/release/simple-docs-bot /opt/app/simple-docs-bot
CMD ["/opt/app/simple-docs-bot"]