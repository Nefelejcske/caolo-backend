from typing import Dict, List, Tuple
from uuid import UUID

from fastapi import APIRouter, Query

import cao_common_pb2
import cao_world_pb2
import cao_world_pb2_grpc
import cao_users_pb2_grpc
from google.protobuf.json_format import MessageToDict

from ..queen import queen_channel


router = APIRouter(prefix="/world", tags=["world"])

TERRAIN_LAYOUT_CACHE = {}

@router.get("/users")
async def list_users():
    # TODO:
    # admins only
    queen = await queen_channel()
    stub = cao_users_pb2_grpc.UsersStub(queen)
    msg = cao_common_pb2.Empty()

    payload = []
    async for userid in stub.ListUsers(msg):
        userid = MessageToDict(
            userid,
            including_default_value_fields=True,
            preserving_proto_field_name=False,
        )
        payload.append(userid)

    return payload


@router.get("/user")
async def get_user_from_sim(user_id: UUID = Query(...)):
    queen = await queen_channel()
    stub = cao_users_pb2_grpc.UsersStub(queen)
    msg = cao_common_pb2.Uuid()
    msg.data = user_id.bytes

    result = await stub.GetUserInfo(msg)

    return MessageToDict(
        result,
        including_default_value_fields=True,
        preserving_proto_field_name=False,
    )

@router.get("/room-terrain-layout", response_model=List[Tuple[int, int]])
async def room_terrain_layout(radius: int = Query(...)):
    """
    return the coordinates of the room grid points in a list.

    If you query the terrain the i-th terrain enum value
    will correspond to the i-th coordinates returned by this endpoint
    """
    return await __get_room_terrain_layout(radius)


async def __get_room_terrain_layout(radius):
    if radius in TERRAIN_LAYOUT_CACHE:
        return TERRAIN_LAYOUT_CACHE[radius]

    channel = await queen_channel()
    stub = cao_world_pb2_grpc.WorldStub(channel)

    msg = cao_world_pb2.GetRoomLayoutMsg(radius=radius)
    room_layout = await stub.GetRoomLayout(msg)
    TERRAIN_LAYOUT_CACHE[radius] = [(p.q, p.r) for p in room_layout.positions]

    return TERRAIN_LAYOUT_CACHE[radius]


@router.get("/tile-enum")
async def tile_enum_values():
    """
    The dictionary returned by this endpoint can be used to
    map Terrain enum values to string values if necessary.
    """
    return {x.index: str(x.name) for x in cao_world_pb2._TERRAIN.values}


@router.get("/rooms", response_model=List[Dict])
async def rooms():
    channel = await queen_channel()
    stub = cao_world_pb2_grpc.WorldStub(channel)

    res = await stub.GetRoomList(cao_common_pb2.Empty())

    return MessageToDict(
        res,
        including_default_value_fields=True,
        preserving_proto_field_name=False,
    ).get("rooms", [])
