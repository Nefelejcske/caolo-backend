# ----------- Build image -----------

FROM python:3.9-slim AS build

RUN apt-get update
RUN apt-get install curl git build-essential -y
RUN pip install -U pip

WORKDIR /caolo/api
RUN pip install --upgrade pip
RUN pip install poetry wheel

# Install deps
COPY ./api/pyproject.toml ./pyproject.toml
COPY ./api/poetry.lock ./poetry.lock
RUN poetry export -f requirements.txt -o requirements.txt
RUN pip install -r requirements.txt

# Build caoloapi
WORKDIR /caolo
COPY ./protos/ ./protos/
COPY ./api/ ./api/
WORKDIR /caolo/api
# build protos
RUN python setup.py protos
# build the wheel
RUN poetry build

# ----------- Prod image -----------

FROM python:3.9-slim

WORKDIR /caolo/api

RUN apt-get update

ENV PATH="/caolo/api/.env/bin:$PATH"
RUN pip install gunicorn

# cache dependencies
COPY --from=build /caolo/api/requirements.txt ./
RUN pip install  -r requirements.txt 

COPY --from=build /caolo/api/start.sh ./
COPY --from=build /caolo/api/dist ./dist

RUN pip install ./dist/caoloapi-0.1.0-py3-none-any.whl

RUN chmod +x start.sh

ENTRYPOINT [ "sh", "./start.sh"]
