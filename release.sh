#!/bin/bash

echo Release command starting

backoff=1

COMMAND=migrate -database ${DATABASE_URL} -path db/migrations up

${COMMAND}

while [ $? -ne 0 ]; do
    if [ $backoff -gt 16 ]; then 
        echo Release command failed repeatedly, aborting
        exit 1
    fi;

    echo Release command failed, retrying in $backoff seconds
    sleep $backoff;
    backoff=$(($backoff * 2))

    ${COMMAND}
done;

echo Release command finished
