#! /bin/bash

PORT=$1
FROM=$2
TO=$3
AMOUNT=$4

DATA="{\"from\": \"$FROM\", \"to\": \"$TO\", \"amount\": $AMOUNT}"

echo $(curl "http://localhost:$PORT/transaction" --data "$DATA")
