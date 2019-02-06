#!/usr/bin/env bash
# Ensure we are in the root of the git repo
#cd $(git rev-parse --show-toplevel)

./target/release/bndb_cli create -c $DATABASE_URL -f -e superuser@test.com -p password -m 8883
cd api && cargo test --release
