import logging
from typing import Optional

from fastapi import FastAPI, Response, status
from fastapi.middleware.cors import CORSMiddleware
from fastapi.middleware.gzip import GZipMiddleware
import asyncpg
from .config import DB_URL

from . import api_v1

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

_DB_POOL: Optional[asyncpg.Pool] = None


async def db_pool() -> asyncpg.Pool:
    global _DB_POOL
    if _DB_POOL is None:
        _DB_POOL = await asyncpg.create_pool(DB_URL)
    assert _DB_POOL is not None  # Silence the warning
    return _DB_POOL


@app.middleware("http")
async def db_session(req, call_next):
    resp = Response(status_code=status.HTTP_503_SERVICE_UNAVAILABLE)
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


@app.get("/v1/health", response_model=HealthStatus)
async def health():
    return Response(status_code=status.HTTP_204_NO_CONTENT)


app.include_router(api_v1.router, prefix="/v1")


@app.on_event("startup")
async def on_start():
    # force connections on startup instead of at the first request
    await db_pool()
