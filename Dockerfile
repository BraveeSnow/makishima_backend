FROM rust:1.77.1-alpine3.19 as builder

WORKDIR /usr/src/makishima
COPY ./Cargo.* ./
COPY ./src/ ./src

RUN apk add --no-cache musl-dev pkgconf libressl-dev
RUN cargo install --path .


FROM alpine:3.19

RUN apk add libssl3
COPY --from=builder /usr/local/cargo/bin/makishima_backend /usr/local/bin/makishima_backend

EXPOSE 3000
CMD [ "makishima_backend" ]
