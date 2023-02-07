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

COPY ./cotevalentines ${APP}

WORKDIR ${APP}
RUN chmod +x ./cotevalentines
ENV DATABASE_URL=./db/sqlite.db
VOLUME [ "/usr/src/app/db" ]
CMD [ "/usr/src/app/cotevalentines" ]