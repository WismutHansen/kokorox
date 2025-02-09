# syntax=docker/dockerfile:1

FROM python:3.12 AS builderpy

COPY /scripts/fetch_voices.py .

ADD https://huggingface.co/hexgrad/kLegacy/resolve/main/v0.19/kokoro-v0_19.onnx .

RUN pip install requests torch numpy && python fetch_voices.py


FROM rust:1.84.0-slim-bookworm AS builderrs

RUN apt-get update -qq && apt-get install -qq -y pkg-config libssl-dev clang git cmake && rustup component add rustfmt

WORKDIR /app

COPY . .
COPY Cargo.toml .
COPY Cargo.lock .

RUN cargo build --release


FROM debian:bookworm-slim AS runner

WORKDIR /app

COPY --from=builderrs /app/target/release/build ./target/release/build
COPY --from=builderrs /app/target/release/koko ./target/release/koko
COPY --from=builderpy /data ./data
COPY --from=builderpy /kokoro-v0_19.onnx ./checkpoints/kokoro-v0_19.onnx

RUN chmod +x ./target/release/koko && apt-get update -qq && apt-get install -qq -y pkg-config libssl-dev 

EXPOSE 3000

ENTRYPOINT [ "./target/release/koko" ] 