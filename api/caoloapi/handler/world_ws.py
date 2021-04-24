import json
import logging
import asyncio
from dataclasses import dataclass

from fastapi import (
    status,
    APIRouter,
    WebSocket,
    Depends,
    WebSocketDisconnect,
    HTTPException,
)


from websockets.exceptions import ConnectionClosedError, ConnectionClosedOK


from ..model.game_state import (
    manager as game_state_manager,
    get_room_objects,
    get_terrain,
)
from ..api_schema import parse_room_id

router = APIRouter(tags=["world"])


@dataclass
class WorldClient:
    ws: WebSocket
    room_id: str = None
    last_seen: int = -1


class WorldMessenger:
    def __init__(self):
        self.connections = []
        self.game_state = None
        self.running = False

    async def connect(self, ws: WorldClient):
        self.connections.append(ws)

    async def disconnect(self, ws: WorldClient):
        try:
            self.connections.remove(ws)
        except ValueError:
            pass

    async def send_terrain(self, client):
        pl = await self._terrain_payload(client)
        pl = json.dumps(pl, default=lambda o: o.__dict__)
        await client.ws.send_text(pl)

    async def send_entities(self, client):
        state = self.game_state or game_state_manager.game_state
        if not state:
            logging.error("No GameState is available")
            return
        client.last_seen = state.created
        entities = get_room_objects(state, client.room_id)
        pl = {"entities": entities, "ty": "entities"}
        pl = json.dumps(pl, default=lambda o: o.__dict__)
        await client.ws.send_text(pl)

    async def _terrain_payload(self, client):
        try:
            room_id = parse_room_id(client.room_id)
        except ValueError as err:
            logging.debug("Failed to parse roomId %s", err)
            return {"error": "Failed to parse roomId", "ty": "error"}

        try:
            terrain = await get_terrain(room_id.q, room_id.r)
            return {"terrain": terrain, "ty": "terrain"}
        except HTTPException as err:
            if err.status_code == status.HTTP_404_NOT_FOUND:
                return {"error": "Room terrain was not found", "ty": "error"}
            else:
                logging.exception("Failed to get terrain")
                return {"error": "Internal server error", "ty": "error"}

    def on_new_state(self, state):
        self.game_state = state
        asyncio.create_task(self.broadcast())

    async def broadcast(self):
        dc = []
        for client in self.connections:
            try:
                await self.send_entities(client)
            except (WebSocketDisconnect, ConnectionClosedError, ConnectionClosedOK):
                dc.append(client)
            except:
                logging.exception("Sending game state failed")
                dc.append(client)
        # disconnected clients
        for c in dc:
            await self.disconnect(c)


world_messanger = WorldMessenger()


game_state_manager.on_new_state(world_messanger.on_new_state)


# NOTE:
# the router.websocket ignores the router's path prefix
@router.websocket("/world/object-stream")
async def object_stream(
    ws: WebSocket, manager: WorldMessenger = Depends(lambda: world_messanger)
):
    """
    Streams game objects of a room.

    Incoming messages should be json, having a `ty` field describing the message type.

    Valid message types:
    - 'room_id': payload = { "room_id": "q;r" } subscribe to a new room.
      e.g.
      ```json
      {
        "ty": "room_id",
        "room_id": "15;12"
      }
      ```
    """
    logging.debug("Client is attempting to connect to object stream")
    await ws.accept()
    client = WorldClient(ws=ws, room_id=None)
    logging.debug("Client connected to object stream")
    try:
        while 1:
            msg = await ws.receive_json()
            if msg.get("ty", None) == "room_id":
                room_id = msg["room_id"]
                # on new room_id disconnect first, as the the terrain sending may take some time to complete
                # and clients may receive entities for the incorrect room
                try:
                    await manager.disconnect(client)
                except:
                    logging.debug("Failed to disconnect client")
                client.room_id = room_id
                # on new room_id send a state immediately
                await manager.send_terrain(client)
                await manager.send_entities(client)
                # subscribe to updates
                await manager.connect(client)
            else:
                raise ValueError("Unprocessable message", msg)
    except (WebSocketDisconnect, ConnectionClosedError, ConnectionClosedOK):
        logging.debug("Client disconnected")
    except:
        logging.exception("Error in object streaming to client")
    finally:
        await manager.disconnect(client)
