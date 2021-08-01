import pytest
from fastapi.testclient import TestClient
from caoloapi.app import app

client = TestClient(app)


@pytest.mark.dependency()
def test_health():
    response = client.get("/health")
    assert response.status_code == 204


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
