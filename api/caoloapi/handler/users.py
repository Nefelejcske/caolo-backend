from typing import Optional
import logging
import string
import random
from uuid import UUID

from fastapi import APIRouter, Request, Depends, HTTPException, Body, status, Query
from fastapi.security import OAuth2PasswordBearer, OAuth2PasswordRequestForm
from pydantic import BaseModel, Field, EmailStr
from jose import JWTError

from asyncpg.exceptions import UniqueViolationError

import cao_commands_pb2
import cao_commands_pb2_grpc
import cao_common_pb2
import cao_users_pb2_grpc
from google.protobuf.json_format import MessageToDict

from ..queen import queen_channel
from ..model.auth import (
    hashpw,
    verifypw,
    PEPPER_RANGE,
    create_access_token,
    decode_access_token,
)


router = APIRouter(tags=["users"])


class User(BaseModel):
    user_id: UUID
    username: str
    displayname: str
    email: Optional[str] = None


oauth2_scheme = OAuth2PasswordBearer(tokenUrl="token")


async def get_current_user_id(token: str = Depends(oauth2_scheme)):
    def credentials_exception():
        return HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Could not validate credentials",
            headers={"WWW-Authenticate": "Bearer"},
        )

    try:
        payload = decode_access_token(token)
    except (AssertionError, JWTError) as err:
        logging.info("Failed to validate JWT %s", err)
        raise credentials_exception()
    return payload.get("sub")


@router.get("/myself")
async def get_myself(req: Request, current_user=Depends(get_current_user_id)):
    current_user = await req.state.db.fetchrow(
        """
        SELECT id, username, email, display_name
        FROM user_account
        WHERE id=$1
        """,
        current_user,
    )
    if not current_user:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED, detail="Log in first"
        )
    return User(
        user_id=current_user["id"],
        username=current_user["username"],
        email=current_user["email"],
        displayname=current_user["display_name"],
    )


def __verify_pw(pw, salt, hashed):
    for pep in range(*PEPPER_RANGE):
        if verifypw(pw, salt, pep, hashed):
            return True
    return False


class RegisterForm(BaseModel):
    username: str = Field(..., min_length=3, max_length=125)
    email: EmailStr
    pw: str = Field(
        ...,
        min_length=8,
        max_length=125,
        description="Passwords must contain at least 8 characters",
    )


@router.get("/sim-users")
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


@router.get("/sim-user")
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


@router.post("/register")
async def register(req: Request, form_data: RegisterForm = Body(...)):
    db = req.state.db

    raw_pw = form_data.pw
    salt = "".join(random.choice(string.ascii_letters) for _ in range(10))
    pepper = random.choice(range(*PEPPER_RANGE))

    pw = hashpw(raw_pw, salt, pepper)

    async with db.transaction():
        pass

        try:
            res = await db.fetchrow(
                """
                INSERT INTO user_account (username, display_name, email, pw, salt)
                VALUES ($1, $1, $2, $3, $4)
                RETURNING id
                """,
                form_data.username,
                form_data.email,
                pw,
                salt,
            )

        except UniqueViolationError as err:
            status_code = status.HTTP_500_INTERNAL_SERVER_ERROR
            detail = ""
            if err.constraint_name == "username_is_unique":
                status_code = status.HTTP_400_BAD_REQUEST
                detail = "Username is already in use"
            elif err.constraint_name == "email_is_unique":
                status_code = status.HTTP_400_BAD_REQUEST
                detail = "Email is already in use"
            else:
                logging.exception("Failed to register new user, constraint not handled")

            raise HTTPException(status_code=status_code, detail=detail) from err

        # NOTE: these two futures could run concurrently, however `db` can not be passed to another coroutine...
        # we could use the connection pool directly instead and try with that...
        await _register_user_in_sim(res["id"])
        token = await _update_access_token(res["id"], db)
    return {"access_token": token, "token_type": "bearer"}


async def _register_user_in_sim(userid):
    queen = await queen_channel()

    stub = cao_commands_pb2_grpc.CommandStub(queen)
    cao_user_id = cao_common_pb2.Uuid()
    cao_user_id.data = userid.bytes
    msg = cao_commands_pb2.RegisterUserCommand(userId=cao_user_id, level=1)
    res = await stub.RegisterUser(msg)

    logging.info("Queen RegisterUser result: %s", res)


async def _update_access_token(userid, db):
    """
    generate a new access token for the given user and store it in the database
    """
    token = create_access_token({"sub": str(userid)})

    await db.execute(
        """
        UPDATE user_account
        SET token=$2
        WHERE id=$1
        """,
        userid,
        token,
    )

    return token


@router.post("/token")
async def login4token(req: Request, form_data: OAuth2PasswordRequestForm = Depends()):
    db = req.state.db

    user_in_db = await db.fetchrow(
        """
        SELECT id, pw, salt
        FROM user_account
        WHERE username=$1
        """,
        form_data.username,
    )

    if not user_in_db:
        raise HTTPException(status_code=400, detail="Incorrect username or password")

    if not __verify_pw(form_data.password, user_in_db["salt"], user_in_db["pw"]):
        raise HTTPException(status_code=400, detail="Incorrect username or password")

    token = await _update_access_token(user_in_db["id"], db)
    return {"access_token": token, "token_type": "bearer"}
