#!/usr/bin/env bash
# Ensure we are in the root of the git repo
# cd $(git rev-parse --show-toplevel)

pwd

ls -lrta

cd db
cargo run --release create -c $DATABASE_URL -f -e superuser@test.com -p password -m 8883
cd ..
cargo test --release --exclude bigneon_api --all
