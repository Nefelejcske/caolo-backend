# ----------- Build image -----------

FROM python:3.9-slim AS build

RUN apt-get update
RUN apt-get install curl git build-essential -y
# install rust in case the wheel for cao-lang can't be downloaded...
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN pip install -U pip virtualenv

WORKDIR /caolo/api
RUN python -m venv .env
RUN .env/bin/pip install --upgrade pip
RUN .env/bin/pip install gunicorn poetry wheel

COPY ./api/pyproject.toml ./pyproject.toml
COPY ./api/poetry.lock ./poetry.lock

# Install deps
RUN .env/bin/poetry export -f requirements.txt -o requirements.txt
RUN .env/bin/pip install -r requirements.txt

# Build caoloapi
WORKDIR /caolo
COPY ./protos/ ./protos/
COPY ./api/ ./api/
WORKDIR /caolo/api
# build protos
RUN .env/bin/python setup.py protos
RUN .env/bin/poetry build

# ----------- Prod image -----------

FROM python:3.9-slim

WORKDIR /caolo/api

RUN apt-get update

COPY --from=build /caolo/api/start.sh ./
COPY --from=build /caolo/api/.env ./.env
COPY --from=build /caolo/api/dist ./dist

ENV PATH="/caolo/api/.env/bin:$PATH"

RUN pip install ./dist/caoloapi-0.1.0-py3-none-any.whl

RUN chmod +x start.sh

ENTRYPOINT [ "sh", "./start.sh"]
