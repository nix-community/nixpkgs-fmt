
1.3.0 / 2022-06-15
==================

### Formatting Changes

  * Remove indentation for lambda function in top level
  * fix multiline comment (#245)
  * Improve formatting with tab characters (#275)
  * change NODE_LET_IN behavior to fix idempotent issue.
  * Update rnix to 0.10.2 (#297, #278)
  * add single space after variable declaration

### Other

  * improve error CLI ergonomics (#269)
  * Properly handle SIGPIPE (#256)

  * Add links to VSCode extensions to README (#259)
  * Bump crossbeam-channel from 0.3.9 to 0.4.4 (#293)
  * Bump regex from 1.5.4 to 1.5.6 (#294)
  * README: fix installation (#246)
  * build: replace flake-compat with flake.lock.nix
  * cargo update
  * cargo update (#271)
  * cargo: fix the rowan dependency
  * default.nix: composition > inheritance
  * default.nix: keep back-compat
  * devShell: add stdenv.cc to the environment
  * docs: clarify changelog generation (#277)
  * fix CLI option output-format (#242)
  * fix ordering error in CI
  * flake update
  * flake.lock.nix: work in pure mode
  * nix: make the shell buildable
  * nix: remove naersk (#272)
  * nix: replace nixpkgs-mozilla with fenix
  * refactor: avoid non_fmt_panics warning (#279)

1.2.0 / 2021-03-29
==================

### Formatting Changes

nixpkgs-fmt is now fully idempotent over nixpkgs!

  * add format rule for NODE_OR_DEFAULT, adding nixpkgs repo test, and remove walk_non_whitespace function (#235)
  * add and fix new test_date to match the new block comment formatting (#233)

### Other

  * flake: use `lib` instead of `stdenv.lib` (PR #234)
  * refactor block comment formatting
  * update vscode's setting.json

1.1.0 / 2021-02-21
==================

### Formatting Changes

  * relax spacing rule for Newline type, simplify NODE_LET_IN spacing rule, clean up warnings (#220)
  * fix #205 - Add space between inherit (#219)

### Other

  * Merge pull request #230 from jD91mZM2/bump-rnix
  * Bump rust version in nix
  * Update rnix + rowan
  * deploy.sh: build wasm before deploying
  * flake: make defaultPackage an alias of nixpkgs-fmt
  * flake update (#227)
  * add dependabot for updating github actions (#226)
  * ci: update GH actions (#225)


1.0.0 / 2020-08-17
==================

### Formatting Changes
  * Add NODE_APPLY rule, remove top_level predicate from spacing and modify test data (#212)
  * Refactor node paren predicate to match node binop and if_else
  * Simplify parentheses rules and fix some test_data to match new rule (#212)
  * Simplify node if_else rules(#212)
  * Unified rules for node apply and remove node apply rule under node key value (#212)
  * Alternative interpolation indentation strategy (#214)
  * Remove unnecessary predicates for NODE_PAREN rules (#212)
  * Commit whitespace changes before computing indentation (#209)

### Other
  
  * Update flake to the new format
  * Use T! macro for symbolic tokens(#211)
  * Remove some commented code (#217)
  * Fix typo (#216)
  * Remove some dead code (#208)
  * Check idempotence before expected (#207)
  
0.9.0 / 2020-05-07
==================

### Formatting changes

  * Change lambda inside node pattern indentation rules (#204)
  * Change key value spacing rues (#204)
  * Change `${ .. }` formatting rules (#204)
  * Change `( .. )` spacing rules (#204)
  * Update test_data (#202, #204)
  * Change `assert` indentation rules (#202)
  * Change `inherit` spacing rules (#202)
  * Change function apply formatting rules (#202, #204)
  * Change `if .. then .. else` spacing rules (#202)

### Other

  * Remove debug print when running nixpkgs-fmt

0.8.0 / 2020-04-22
==================

### Formatting changes

  * Change multiline string formatting rules (#193) 
  * Change `${ .. }` formatting rules (#187)
  * Change function apply function rules (#174)
  * Change `let .. in ..` formatting rules (#180)
  * Change binops formatting rules (#177)
  * Update test_data (#173, #174, #176, 177, #180, #182, #183, #187, #188, #193)
  * Change brackets' formatting rules (#188)
  * Change `( .. )` formatting rules (#177, #180, #182, #183)
  * Change `if .. then .. else` formatting rules (#176)
  * Change comment rules (#180, #193)
  * Change semicolon formatting rules (#172)
  * Change lambda function formatting rules (#173)
  * Change `{ .. }` formatting rules (#177)

### Other

  * Update README (#192)
  * Update naersk
  * Update flake.nix (#173, #188, 193)

0.7.0 / 2020-02-09
==================

### Formatting changes

  * Change the `let ... in ...` formatting rules (#169, #168, #167, #125)

### Other

  * Add flake support
  * Update naersk
  * CI: switch to GitHub actions

0.6.1 / 2019-11-05
==================

### Formatting changes

  * Support float scientific notation (#150)

### Other

  * Fix clippy lint warnings/errors (#149)

0.6.0 / 2019-09-16
==================

### Formatting changes

NONE

### Features

  * print touched files to stdout (#148)
  * implement `nixpkgs-fmt --check` for CI (#148)

### Other

  * shell.nix: pin rust version and use extensions from the distribution (#148)
  * fix typo in README (#146)

0.5.0 / 2019-09-07
==================

### Formatting changes

  * convert tabs to spaces (#143)

### Features

  * add --explain mode to expose the engine rewrite decisions (#142)

### Other

  * replace #[macro_use] extern crate with modern syntax (#141)
  * incorporate recent rnix renamings (#144)
  * nix: use naersk so hashes are always up to date (#145)

0.4.0 / 2019-08-31
==================

### Formatting changes

* Don't force newline before ++ anymore (#139)
* Always indent concatenated lists
* Add line break after comment in list

### Features

* Add ability to print syntax tree in JSON format
* Format directories out of the box. Eg: `nixpkgs-fmt .`
* Refactor input handling, makes formatting 4x faster

### Changes

* Add test to make sure the output is idempotent

### Other

* BREAKING: Remove the --in-place flag

0.3.1 / 2019-08-23
==================

  * fix the release process

0.3.0 / 2019-08-23
==================

  * First lambda arg is on the line with brace

0.2.0 / 2019-08-23
==================

First release!
