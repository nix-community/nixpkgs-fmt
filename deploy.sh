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

origin_url=$(git remote get-url origin)
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

run cd "$workdir/repo"

if ! git diff-files --quiet ; then
  echo "No changes found, aborting"
  exit
fi

run git add -A .
run git commit --author "CI <ci@ci.com>" --message "."
run git show --stat-count=10 HEAD
run git push -f origin gh-pages
