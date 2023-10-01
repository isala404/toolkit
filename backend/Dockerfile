FROM rust:latest AS build

WORKDIR /usr/src/app
COPY . .

# Setup sqlx
RUN cargo install sqlx-cli
RUN sqlx db setup --database-url=sqlite:toolkit.db
ENV DATABASE_URL=sqlite:toolkit.db

RUN cargo build --release

# Final image
FROM alpine:latest

RUN apk --no-cache add ca-certificates

WORKDIR /usr/src/app
COPY --from=build /usr/src/app/target/release/toolkit .

CMD ["./toolkit"]
