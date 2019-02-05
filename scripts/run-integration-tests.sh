#!/usr/bin/env bash

# Ensure we are in the root of the git repo
#cd $(git rev-parse --show-toplevel)
cd db

if [[ -f ./target/release/bndb_cli ]]; then
    echo "----- Reusing built bndb_cli -----"
    ./target/release/bndb_cli create -c $DATABASE_URL -f -e superuser@test.com -p password -m 8883 || {
        echo "Migrations failed"
        exit 1
    }
else
    cargo run --release create -c $DATABASE_URL -f -e superuser@test.com -p password -m 8883 || {
        echo "Migrations failed"
        exit 1
    }
fi

if [[ -f ./target/release/server ]]; then
    echo "----- Reusing built server -----"
    ./target/release/server -t false &
    export SERVER_PID=$!$1
else
    cd ../api
    cargo build --release
    cargo run --release -- -t false &
    export SERVER_PID=$!$1
fi

# Run newman tests
newman run --timeout-request 60000 ../integration-tests/bigneon-tests.postman_collection.json -e ../integration-tests/travis.postman_environment.json
NEWMAN_EXIT_CODE=$?
kill -s SIGTERM $SERVER_PID
if [[ $NEWMAN_EXIT_CODE -ne 0 ]]
then
    exit $NEWMAN_EXIT_CODE
fi
cargo run --release -- -b true

