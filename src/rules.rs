//! This module contains specific `super::dsl` rules for formatting nix language.
use rnix::{
    parser::nodes::*,
    types::{Apply, Lambda, TypedNode, With},
    SyntaxElement, SyntaxKind, T,
};

use crate::{
    dsl::{IndentDsl, SpacingDsl},
    tree_utils::prev_sibling,
};

#[rustfmt::skip]
pub(crate) fn spacing() -> SpacingDsl {
    // Note: comments with fat arrow are tests!
    let mut dsl = SpacingDsl::default();

    dsl
        // { a=92; } => { a = 92; }
        .inside(NODE_SET_ENTRY).around(T![=]).single_space()

        // { a = 92 ; } => { a = 92; }
        .inside(NODE_SET_ENTRY).before(T![;]).no_space()
        .inside(NODE_SET_ENTRY).before(T![;]).when(after_bin_op).no_space_or_newline()

        // a==  b => a == b
        .inside(NODE_OPERATION).around(T![==]).single_space()

        // a++  b => a ++ b
        .inside(NODE_OPERATION).after(T![++]).single_space()
        .inside(NODE_OPERATION).before(T![++]).single_space_or_newline()

        // a+  b => a + b
        // a  -   b => a - b
        // a*  b => a * b
        // a/  b => a / b
        .inside(NODE_OPERATION).after([T![++], T![+], T![-], T![*], T![/]]).single_space()
        .inside(NODE_OPERATION).before([T![++], T![+], T![-], T![*], T![/]]).single_space_or_newline()

        // foo . bar . baz => foo.bar.baz
        .inside(NODE_INDEX_SET).around(T![.]).no_space()

        // {} : 92 => {}: 92
        .inside(NODE_LAMBDA).before(T![:]).no_space()

        // [1 2 3] => [ 1 2 3 ]
        .inside(NODE_LIST).after(T!["["]).single_space_or_newline()
        .inside(NODE_LIST).before(T!["]"]).single_space_or_newline()
        // [ ] => []
        .inside(NODE_LIST).between(T!["["], T!["]"]).no_space()
        .inside(NODE_LIST).after(VALUES).single_space_or_newline()
        .inside(NODE_LIST).after(TOKEN_COMMENT).single_space_or_newline()

        // {foo = 92;} => { foo = 92; }
        .inside(NODE_SET).after(T!["{"]).single_space_or_newline()
        .inside(NODE_SET).before(T!["}"]).single_space_or_newline()
        // { } => {}
        .inside(NODE_SET).between(T!["{"], T!["}"]).no_space()
        .inside(NODE_SET).after(NODE_SET_ENTRY).single_space_or_newline()

        // {arg}: 92 => { arg }: 92
        .inside(NODE_PATTERN).after(T!["{"]).single_space()
        .inside(NODE_PATTERN).before(T!["}"]).single_space_or_newline()
        // { }: 92 => {}: 92
        .inside(NODE_PATTERN).between(T!["{"], T!["}"]).no_space()

        // { foo,bar }: 92 => { foo, bar }: 92
        .inside(NODE_PATTERN).after(T![,]).single_space()
        .inside(NODE_PATTERN).before(T![,]).no_space_or_newline()

        // { inherit( x )  y  z  ; } => { inherit (x) y z; }
        .inside(NODE_INHERIT).around(NODE_INHERIT_FROM).single_space()
        .inside(NODE_INHERIT).before(T![;]).no_space()
        .inside(NODE_INHERIT).before(NODE_IDENT).single_space()
        .inside(NODE_INHERIT_FROM).after(T!["("]).no_space()
        .inside(NODE_INHERIT_FROM).before(T![")"]).no_space()

        // let   foo = bar;in  92 => let foo = bar; in 92
        .inside(NODE_LET_IN).after(T![let]).single_space_or_newline()
        .inside(NODE_LET_IN).around(T![in]).single_space_or_newline()
        ;

    dsl
}

fn after_bin_op(node: SyntaxElement<'_>) -> bool {
    match prev_sibling(node).map(|it| it.kind()) {
        Some(NODE_OPERATION) => true,
        _ => false,
    }
}

#[rustfmt::skip]
pub(crate) fn indentation() -> IndentDsl {
    let mut dsl = IndentDsl::default();
    dsl
        .inside(NODE_LIST).indent(VALUES)
        .inside(ENTRY_OWNERS).indent([NODE_SET_ENTRY, NODE_INHERIT])

        .inside(NODE_LAMBDA).when(lambda_body_not_on_top_level).indent(VALUES)
        .inside(NODE_WITH).when(with_body).indent(VALUES)
        .inside(NODE_APPLY).when(apply_arg).indent(VALUES)

        // FIXME: don't force indent if comment is on the first line
        .inside(NODE_LIST).indent(TOKEN_COMMENT)
        .inside(ENTRY_OWNERS).indent(TOKEN_COMMENT)
        ;
    dsl
}

fn lambda_body_not_on_top_level(body: SyntaxElement<'_>) -> bool {
    let body = match body {
        SyntaxElement::Node(it) => it,
        SyntaxElement::Token(_) => return false,
    };
    let lambda = match body.parent().and_then(Lambda::cast) {
        None => return false,
        Some(it) => it,
    };
    lambda.body() == body && lambda.node().parent().map(|it| it.kind()) != Some(NODE_ROOT)
}

fn with_body(body: SyntaxElement<'_>) -> bool {
    let body = match body {
        SyntaxElement::Node(it) => it,
        SyntaxElement::Token(_) => return false,
    };
    let with = match body.parent().and_then(With::cast) {
        None => return false,
        Some(it) => it,
    };
    with.body() == body
}

fn apply_arg(arg: SyntaxElement<'_>) -> bool {
    let arg = match arg {
        SyntaxElement::Node(it) => it,
        SyntaxElement::Token(_) => return false,
    };
    let apply = match arg.parent().and_then(Apply::cast) {
        None => return false,
        Some(it) => it,
    };
    apply.value() == arg
}

static ENTRY_OWNERS: &'static [SyntaxKind] = &[NODE_SET, NODE_LET_IN];

static VALUES: &'static [SyntaxKind] = &[
    NODE_IDENT,
    NODE_INDEX_SET,
    NODE_LAMBDA,
    NODE_LET_IN,
    NODE_LIST,
    NODE_OPERATION,
    NODE_PAREN,
    NODE_SET,
    NODE_STRING,
    NODE_VALUE,
];

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use crate::reformat_string;

    #[test]
    fn smoke() {
        TestCase {
            name: None,
            before: "{
foo = x:
92;
}"
            .into(),
            after: "{
  foo = x:
    92;
}"
            .into(),
        }
        .run()
        .map_err(|e| panic!(e))
        .unwrap();
    }

    /// For convenience, tests in this module are specified inline in comments,
    /// right next to the corresponding rule definition. This test looks at the
    /// text of this file, extracts test cases from comments and checks them.
    #[test]
    fn test_inline_tests() {
        let this_file = include_str!("./rules.rs");
        let tests = TestCase::collect_from_comments(this_file);
        run(&tests);
    }

    #[test]
    fn test_bad_good_tests() {
        let test_data = {
            let dir = env!("CARGO_MANIFEST_DIR");
            PathBuf::from(dir).join("test_data")
        };
        let tests = TestCase::collect_from_dir(&test_data);
        run(&tests);
    }

    #[derive(Debug)]
    struct TestCase {
        name: Option<String>,
        before: String,
        after: String,
    }

    impl TestCase {
        fn try_from(line: &str) -> Option<TestCase> {
            let divisor = line.find("=>")?;
            let before = line[..divisor].trim().to_string();
            let after = line[divisor + 3..].trim().to_string();
            Some(TestCase {
                name: None,
                before,
                after,
            })
        }

        fn collect_from_comments(text: &str) -> Vec<TestCase> {
            let res = text
                .lines()
                .filter_map(|line| line.find("// ").map(|idx| &line[idx + 3..]))
                .filter_map(TestCase::try_from)
                .collect::<Vec<_>>();

            assert!(res.len() > 0);
            res
        }

        fn collect_from_dir(dir: &Path) -> Vec<TestCase> {
            let mut res = vec![];
            for entry in fs::read_dir(dir).unwrap() {
                let entry = entry.unwrap();
                let file_name = entry.file_name();
                let before_name = file_name.to_str().unwrap();
                if before_name.ends_with(".bad.nix") {
                    let after_name = before_name.replace(".bad.", ".good.");
                    let test_case = TestCase {
                        name: Some(after_name.to_string()),
                        before: fs::read_to_string(dir.join(before_name)).unwrap(),
                        after: fs::read_to_string(dir.join(&after_name)).unwrap_or_else(|_err| {
                            panic!("{} not found", after_name);
                        }),
                    };
                    res.push(test_case);
                }
            }
            assert!(res.len() > 0);
            res
        }

        fn run(&self) -> Result<(), String> {
            let name = self.name.as_ref().map(|it| it.as_str()).unwrap_or("");
            let expected = &self.after;
            let actual = &reformat_string(&self.before);
            if expected != actual {
                return Err(format!(
                    "\n\nAssertion failed: wrong formatting\
                     \nTest: {}\n\
                     \nBefore:\n{}\n\
                     \nAfter:\n{}\n\
                     \nExpected:\n{}\n",
                    name, self.before, actual, self.after,
                ));
            }
            let second_round = &reformat_string(actual);
            if actual != second_round {
                return Err(format!(
                    "\n\nAssertion failed: formatting is not idempotent\
                     \nTest: {}\n\
                     \nBefore:\n{}\n\
                     \nAfter:\n{}\n",
                    name, actual, second_round,
                ));
            }
            Ok(())
        }
    }

    fn run(tests: &[TestCase]) {
        let mut n_failed = 0;
        for test in tests {
            if let Err(msg) = test.run() {
                n_failed += 1;
                eprintln!("{}", msg)
            }
        }
        if n_failed > 0 {
            panic!(
                "{} failed test cases out of {} total",
                n_failed,
                tests.len()
            )
        }
    }
}
