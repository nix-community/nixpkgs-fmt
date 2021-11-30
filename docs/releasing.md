# Releasing

This document describes the steps taken to release new versions of this
project.

1. Fill the CHANGELOG.md file. 
    - Use `git-changelog` to automatically generate the changelog from commit messages.
    You can install this from the [`git-extras`](https://github.com/tj/git-extras) package.
2. Bump the version in Cargo.toml
3. Run `./update-cargo-nix.sh` to update the Cargo.lock and associated nix
   files
4. Create a release commit: `git commit -a -m "Release v<VERSION>"`
5. Tag the release: `git tag v<VERSION>`
6. Push all of this: `git push --follow-tags`
7. Run `cargo publish`
