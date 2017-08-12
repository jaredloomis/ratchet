#! /bin/bash

PORT=$1
ACCOUNT=$2

echo $(curl "http://localhost:$PORT/balance?account=$ACCOUNT")
