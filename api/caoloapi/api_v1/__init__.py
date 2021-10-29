"""
request handlers
"""
from fastapi import APIRouter

from . import admin
from . import commands
from . import scripting
from . import users
from . import world

router = APIRouter()
router.include_router(world.router)
router.include_router(scripting.router)
router.include_router(admin.router)
router.include_router(commands.router)
router.include_router(users.router)
