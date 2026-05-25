#!/bin/bash

cd /app && { ./auroraops-server || ./hotgo; } &
echo "auroraops-server start all server.."
tail -f /dev/null
