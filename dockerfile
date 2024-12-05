FROM rust:1.83-bullseye as builder
WORKDIR /usr/src/activity-tracker-bot
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
COPY --from=builder /usr/local/cargo/bin/activity-tracker-bot /usr/local/bin/activity-tracker-bot
CMD ["activity-tracker-bot"]

