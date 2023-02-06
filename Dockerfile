FROM debian:stable-slim
ARG APP=/usr/src/app

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY ./Cotevalentines2023 ${APP}

WORKDIR ${APP}

ENV DATABASE_URL=./db/sqlite.db
VOLUME [ "./db" ]
CMD [ "DATABASE_URL=${DATABASE_URL} ./cotevalentines" ]