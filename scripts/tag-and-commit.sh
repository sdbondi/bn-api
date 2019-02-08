#!/usr/bin/env bash

set -e

if [[ -z "$CI" ]]; then
    echo "Script should be run in CI only"
    exit 1
fi

if [[ -z "$APP_VERSION" ]]; then
    echo "APP_VERSION env var required"
    exit 1
fi

mkdir -p $HOME/.ssh/
declare -r SSH_FILE="$(mktemp -u $HOME/.ssh/githubXXXXXX)"

ssh-keyscan github.com > ~/.ssh/known_hosts 2> /dev/null
eval $(ssh-agent -s)
ssh-add <(echo "$GITHUB_SSH_KEY")

git config --global user.email "$GH_USER_EMAIL"
git config --global user.name "$GH_USER_NAME"

version=$APP_VERSION

git checkout master

git remote add sshremote git@github.com:$DRONE_REPO.git

git add db/Cargo.toml api/Cargo.toml
git commit -m  "Version set to ${version} [skip ci]"
git tag ${version}
git push sshremote master
git push sshremote ${version}
