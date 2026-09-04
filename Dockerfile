FROM rust:1.98-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release
CMD ["./target/release/playcua"]
