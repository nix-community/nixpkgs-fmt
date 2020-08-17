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
