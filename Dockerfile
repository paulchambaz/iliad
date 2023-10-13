FROM rust:1.73-bullseye AS build

RUN rustup default nightly

WORKDIR /app
COPY . .

RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update --no-install-recommends && \
    apt-get install -y ffmpeg libsqlite3-0

WORKDIR /app

COPY --from=build /app/target/release/iliad /app
COPY --from=build /app/.env /app

CMD [ "./iliad" ] 
