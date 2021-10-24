FROM golang:alpine as protos

RUN apk add python3 protoc

RUN export GO111MODULE=on  # Enable module mode
RUN go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
RUN go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest

WORKDIR /caolo
COPY ./protos ./protos/
COPY ./rt/protos.py ./rt/

WORKDIR /caolo/rt

ENV CAO_PROTOS_PATH=/caolo/protos
RUN python3 protos.py

FROM golang:alpine as build

WORKDIR /caolo/rt
COPY ./rt ./
COPY --from=protos /caolo/rt/ ./

RUN go build

FROM alpine

WORKDIR /caolo
COPY --from=build /caolo/rt/cao-rt ./

ENTRYPOINT ./cao-rt -addr :${PORT:-8080} -simAddr ${CAO_QUEEN_URL}
