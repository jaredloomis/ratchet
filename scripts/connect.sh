#! /bin/bash

PORT_1=$1
PORT_2=$2

echo $(curl "http://localhost:$PORT_1/connect?peer=localhost:$PORT_2")
