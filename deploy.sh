#!/usr/bin/env bash
#
# Make sure to run wasm/build.sh before running this script
set -euo pipefail

## Functions ##

run() {
  echo
  echo "$ $*" >&2
  "$@"
}

## Main ##

origin_url=git@github.com:nix-community/nixpkgs-fmt.git
workdir=$(mktemp -d)
cleanup() {
  rm -rf "$workdir"
}
trap cleanup EXIT

if [[ -n "${DEPLOY_SSH_KEY:-}" ]]; then
  ssh_key_path=$workdir/ssh_key
  echo "$DEPLOY_SSH_KEY" | base64 -d > "$ssh_key_path"
  run chmod 0600 "$ssh_key_path"
  export GIT_SSH_COMMAND="ssh -i '$ssh_key_path'"
fi

run git clone --depth=1 --branch=gh-pages "$origin_url" "$workdir/repo"

run rsync -rl --exclude .git --delete wasm/ "$workdir/repo"

run rm "$workdir/repo/pkg/.gitignore"

run cd "$workdir/repo"

run git add -A .

if ! run git commit --author "CI <ci@ci.com>" --message "." ; then
  exit
fi

run git show --stat-count=10 HEAD
run git push -f origin gh-pages
