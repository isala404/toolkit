FROM rust:latest AS build

WORKDIR /usr/src/app
COPY . .

# Install musl target (needed for alpine)
RUN apt-get update && apt-get install -y musl-tools pkg-config libssl-dev
RUN rustup target add x86_64-unknown-linux-musl

# Setup sqlx
ENV SQLX_OFFLINE=true

RUN cargo build --release --target x86_64-unknown-linux-musl

# Final image
FROM alpine:latest

RUN apk --no-cache add ca-certificates yt-dlp ffmpeg

WORKDIR /usr/src/app
COPY --from=build /usr/src/app/target/x86_64-unknown-linux-musl/release/toolkit .

CMD ["./toolkit"]
