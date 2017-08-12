#! /bin/bash

PORT=$1
ADDRESS=$2

echo $(curl "http://localhost:$PORT/balance?address=$ADDRESS")
