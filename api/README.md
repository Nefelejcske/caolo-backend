## Prerequisites

### Native builds:

-   [Poetry](https://python-poetry.org/docs/)
-   [Python](https://python.org/)
-   [Protoc](https://grpc.io/docs/protoc-installation/)
-   [PostgeSQL](https://www.postgresql.org/)
-   [migrate](https://github.com/golang-migrate/migrate/blob/master/cmd/migrate/README.md)

### Docker builds:

-   [Docker](https://www.docker.com/)
-   [Make](https://www.gnu.org/software/make/) (Optional)

## Setting up

```
migrate -database ${DATABASE_URL} -path ../db/migrations up
poetry install
```

## Running

-   Running the web service

    ```
    uvicorn caoloapi.app:app --reload
    ```

## OpenAPI

Visit `http[s]://<url>/docs`

## E2E testing

- Start `sim`
- Start a database

```
make test

# or

pytest test/
```
