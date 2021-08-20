import pytest
from fastapi.testclient import TestClient
from caoloapi.app import app
from pprint import pprint

client = TestClient(app)


@pytest.mark.dependency()
def test_health():
    response = client.get("/health")
    assert response.status_code == 200


@pytest.mark.dependency(depends=["test_health"])
def test_schema():
    response = client.get("/scripting/schema")

    assert response.status_code == 200

    body = response.json()

    assert body

    expected_keys = sorted(
        [
            "name",
            "description",
            "inputs",
            "ty",
            "outputs",
            "properties",
        ]
    )
    assert sorted(list(body[0].keys())) == expected_keys


def test_compile_hello_world():
    response = client.post(
        "/scripting/compile",
        json={
            "lanes": [
                {
                    "name": "hello",
                    "cards": [
                        {"ty": "StringLiteral", "val": "World"},
                        {"ty": "CallNative", "val": "console_log"},
                    ],
                }
            ]
        },
    )

    pprint(response.json())
    assert response.status_code == 200


def test_compile_bad_ty():
    response = client.post(
        "/scripting/compile",
        json={
            "lanes": [
                {
                    "name": "hello",
                    "cards": [
                        {"ty": "poggers-moggers", "val": "console_log"},
                    ],
                }
            ]
        },
    )

    pprint(response.json())
    assert response.status_code == 400


def test_compile_missing_val():
    response = client.post(
        "/scripting/compile",
        json={
            "lanes": [
                {
                    "name": "hello",
                    "cards": [
                        {"ty": "CallNative"},
                    ],
                }
            ]
        },
    )

    pprint(response.json())
    assert response.status_code == 400
