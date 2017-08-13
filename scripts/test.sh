#! /bin/bash

cd $(dirname $0)

# Transfer 4 from jaredloomis -> harrypotter
bash transaction.sh 1234 jaredloomis harrypotter 4

# Mine for harrypotter
bash mine.sh 1234 harrypotter

bash connect.sh 4567 1234

bash balance.sh 4567 jaredloomis
