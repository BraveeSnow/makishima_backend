FROM rust:1.77.1-bookworm as builder

WORKDIR /usr/src/makishima
COPY . .

RUN cargo install --path .


FROM debian:bookworm-slim

RUN apt update && apt install libssl3
COPY --from=builder /usr/local/cargo/bin/makishima_backend /usr/local/bin/makishima_backend
CMD [ "makishima_backend" ]
