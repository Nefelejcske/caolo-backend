"""
Requires the `api` service
"""

from fastapi.testclient import TestClient

from caoloapi.app import app

client = TestClient(app)


def test_rooms():
    response = client.get("/v1/world/rooms")
    assert response.status_code == 200

    rooms = response.json()
    assert rooms

    assert "roomId" in rooms[0]
    assert "radius" in rooms[0]


def test_room_terrain_layout():
    response = client.get(url="/v1/world/room-terrain-layout", params={"radius": 4})
    assert response.status_code == 200

    layout = response.json()
    assert layout
