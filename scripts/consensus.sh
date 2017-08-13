#! /bin/bash

PORT=$1

echo $(curl "http://localhost:$PORT/consensus")
