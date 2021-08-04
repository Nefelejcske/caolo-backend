FROM ubuntu:18.04

RUN curl -L https://packagecloud.io/golang-migrate/migrate/gpgkey | apt-key add -
RUN echo "deb https://packagecloud.io/golang-migrate/migrate/ubuntu/ $(lsb_release -sc) main" > /etc/apt/sources.list.d/migrate.list
RUN apt-get update
RUN apt-get install bash migrate -y

WORKDIR /caolo

COPY ./db/migrations/ ./db/migrations/
COPY ./release.sh ./

ENTRYPOINT ["./release.sh"]
