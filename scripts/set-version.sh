#!/usr/bin/env bash

set -e

if [[ $# -ne 1 ]]; then
    echo "USAGE: $0 [version]"
    exit 1
fi

new_version=$1

function set_cargo_version {
    local file="$1"
    local version=`sed -En 's/version[[:space:]]*=[[:space:]]*"([[:digit:]]+\.[[:digit:]]+\.[[:digit:]]+)"/\1/p' < $file`
    local search='^(version[[:space:]]*=[[:space:]]*).+'
    local replace="\1\"${new_version}\""

    sed -i.tmp -E "s/${search}/${replace}/g" "$1"
    echo "$file set ($version -> $new_version)"
    rm "$1.tmp"
}

FILES=( "db/Cargo.toml" "api/Cargo.toml" )

for target in "${FILES[@]}"; do
    set_cargo_version "$target"
done


