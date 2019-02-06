#!/usr/bin/env bash

set -e

new_version=$APP_VERSION

function bump_patch {
    local file="$1"
    local version=`sed -En 's/version[[:space:]]*=[[:space:]]*"([[:digit:]]+\.[[:digit:]]+\.[[:digit:]]+)"/\1/p' < $file`
    local search='^(version[[:space:]]*=[[:space:]]*).+'
    local replace="\1\"${new_version}\""

    sed -i.tmp -E "s/${search}/${replace}/g" "$1"
    echo "$file set to $new_version from $version"
    rm "$1.tmp"
}

FILES=( "db/Cargo.toml" "api/Cargo.toml" )

for target in "${FILES[@]}"; do
    bump_patch "$target"
done


