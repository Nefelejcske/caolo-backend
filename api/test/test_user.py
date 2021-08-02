from fastapi.testclient import TestClient

from caoloapi.app import app

client = TestClient(app)


def test_myself_without_token_returns_error():
    response = client.get("/myself", headers={})

    assert response.status_code == 401
