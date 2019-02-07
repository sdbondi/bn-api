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

ssh-keyscan github.com > ~/.ssh/known_hosts

eval $(ssh-agent -s)

#echo -n $GITHUB_SSH_KEY > $SSH_FILE
ssh-add <(echo "$GITHUB_SSH_KEY")
# Enable SSH authentication

#chmod 600 "$SSH_FILE"
#printf "%s\n" \
#  "Host github.com" \
#  "  IdentityFile $SSH_FILE" \
#  "  LogLevel ERROR" >> ~/.ssh/config

ssh -T git@github.com

git config --global user.email "$GH_USER_EMAIL"
git config --global user.name "$GH_USER_NAME"

version=$APP_VERSION

git checkout master

git remote add sshremote git@github.com:sdbondi/bn-api.git

git add db/Cargo.toml api/Cargo.toml
git commit -m  "Version set to ${version} [skip ci]"
git tag ${new_version}
git push sshremote master
git push sshremote ${new_version}
