import grpc
from .config import QUEEN_URL
from .util import aio_with_backoff

_QUEEN_CHANNEL_CACHE = {}


@aio_with_backoff(retries=None, max_sleep=10)
async def queen_channel(queen_url=QUEEN_URL) -> grpc.Channel:
    global _QUEEN_CHANNEL_CACHE
    if queen_url not in _QUEEN_CHANNEL_CACHE:
        # TODO: use secure channel pls
        _QUEEN_CHANNEL_CACHE[queen_url] = grpc.aio.insecure_channel(queen_url)
    return _QUEEN_CHANNEL_CACHE[queen_url]
