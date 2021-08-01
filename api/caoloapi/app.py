import asyncio
import logging

from fastapi import FastAPI, Response
from fastapi.middleware.cors import CORSMiddleware
from fastapi.middleware.gzip import GZipMiddleware
import asyncpg
from .config import DB_URL

from . import handler
from .queen import queen_channel

import cao_common_pb2
import cao_common_pb2_grpc


logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(pathname)s:%(lineno)d: %(message)s",
)


tags_metadata = [
    {"name": "world", "description": "game world related stuff"},
    {"name": "scripting", "description": "Cao-Lang related stuff"},
    {"name": "commands", "description": "Simulation interaction"},
    {"name": "users", "description": "User management"},
]

app = FastAPI(title="Cao-Lo API", version="0.1.0", openapi_tags=tags_metadata)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)
app.add_middleware(GZipMiddleware, minimum_size=1000)

_DB_POOL = None


async def db_pool():
    global _DB_POOL
    if _DB_POOL is None:
        _DB_POOL = await asyncpg.create_pool(DB_URL)
    return _DB_POOL


@app.middleware("http")
async def db_session(req, call_next):
    resp = Response(status_code=500)
    pool = await db_pool()
    assert pool is not None
    async with pool.acquire() as con:
        req.state.db = con
        resp = await call_next(req)
    return resp


@app.middleware("http")
async def rate_limit(req, call_next):
    # TODO
    return await call_next(req)


@app.get("/health")
async def health():
    async def _ping_queen():
        channel = await queen_channel()
        stub = cao_common_pb2_grpc.HealthStub(channel)
        msg = cao_common_pb2.Empty()
        resp = await stub.Ping(msg)
        return resp

    await asyncio.gather(_ping_queen(), db_pool())  # test dependencies

    return Response(status_code=204)


app.include_router(handler.world.router)
app.include_router(handler.scripting.router)
app.include_router(handler.admin.router)
app.include_router(handler.commands.router)
app.include_router(handler.users.router)


@app.on_event("startup")
async def on_start():
    # force connections on startup instead of at the first request
    await db_pool()
