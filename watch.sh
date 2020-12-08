#!/bin/sh

echo "STARTING SCRIPT"
touch .trigger
cargo watch -w ./src -w ./proto -x build -s 'touch .trigger' --postpone &
cargo watch --no-gitignore -w .trigger -x run 