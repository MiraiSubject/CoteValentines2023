FROM rust:1.67 as builder
WORKDIR /usr/src/cotevalentines
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates tzdata && rm -rf /var/lib/apt/lists/*
ARG APP=/app
RUN mkdir -p ${APP}
WORKDIR ${APP}
COPY --from=builder /usr/local/cargo/bin/cotevalentines /app/cotevday

ENV TZ=Etc/UTC

ENV DATABASE_URL=/app/db/sqlite.db
VOLUME [ "/app/db" ]
CMD [ "/app/cotevday" ]