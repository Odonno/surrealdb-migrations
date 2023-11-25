#! /usr/bin/env bash

kill $(lsof -t -i:8000)
kill $(lsof -t -i:8001)

surreal start --strict --user root  --pass root  --bind 0.0.0.0:8000 memory --auth --allow-guests &
surreal start --strict --user admin --pass admin --bind 0.0.0.0:8001 memory --auth --allow-guests &
