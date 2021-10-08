FROM migrate/migrate:latest

RUN apk update
RUN apk add bash

WORKDIR /caolo

COPY ./db/migrations/ ./db/migrations/
COPY ./db/release.sh ./

ENTRYPOINT ["./release.sh"]
