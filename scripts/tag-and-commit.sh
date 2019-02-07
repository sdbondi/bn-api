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

declare -r SSH_FILE="$(mktemp -u $HOME/.ssh/githubXXX)"

echo -n $GITHUB_SSH_KEY > $SSH_FILE

# Enable SSH authentication

chmod 600 "$SSH_FILE" && \
    printf "%s\n" \
      "Host github.com" \
      "  IdentityFile $SSH_FILE" \
      "  LogLevel ERROR" >> ~/.ssh/config



version=$APP_VERSION

git add db/Cargo.toml api/Cargo.toml
git commit -m  "Version set to ${version} [skip ci]"
git tag ${new_version}
git push sshremote master
git push sshremote ${new_version}
