# HOWTO write new rules

This document explains the overall process one ususually go through for adding
new rules.

## Create the nix code fixtures

In the `test_data` folder, create two new fixtures for the expected input and
outputs of the tool.

Eg: `test_data/fn_args_singleline.bad.nix` and `test_data/fn_args_singleline.good.nix`

Put the content in there and format the files by hand.

Running `cargo test` should fail if the fixtures aren't supported by the
current rewriting rules.

At this point, a PR can already be sent to demonstrate the failure.

## Create the new rule

Run the tool with `--parse` to learn the names of nodes were are interested in:

```
$ cargo run -- --parse test_data/fn_args_singleline.bad.nix

NODE_ROOT 0..25 {
  NODE_LAMBDA 0..24 {
    NODE_PATTERN 0..17 { <- we need this one
      TOKEN_CURLY_B_OPEN("{") 0..1
      NODE_PAT_ENTRY 1..7 {
        NODE_IDENT 1..7 {
          TOKEN_IDENT("stdenv") 1..7
        }
      }
      ....
```

Then add the spacing rules in `rules.rs`, together with the inline test:

```
// {arg}: 92 => { arg }: 92
.inside(NODE_PATTERN).after(T!['{']).single_space_or_newline()
.inside(NODE_PATTERN).before(T!['}']).single_space_or_newline()
```

At this point is makes sense to learn a bit more about rowan, the internals of
the project, and this should definitely be covered in another document. Until
then, please ping @matklad with any questions you might have.

Iterate until it works.

Push to the PR with the fixes.

## Success

Thanks for helping out!
