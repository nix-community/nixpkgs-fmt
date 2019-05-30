//! This module contains specific `super::dsl` rules for formatting nix language.
use rnix::{parser::nodes::*, SyntaxKind};

use crate::dsl::{self, indent, inside};

#[rustfmt::skip]
pub(crate) fn spacing() -> Vec<dsl::SpacingRule> {
    let mut rules = Vec::new();
    let mut r = |b: dsl::SpacingRuleBuilder| rules.push(b.into());

    // Note: comments with fat arrow are tests!

    // { a=92; } => { a = 92; }
    r(inside(NODE_SET_ENTRY).around(T![=]).single_space());

    // { a = 92 ; } => { a = 92; }
    r(inside(NODE_SET_ENTRY).before(T![;]).no_space());

    // a++  b => a ++ b
    r(inside(NODE_OPERATION).around(T![++]).single_space());

    // a==  b => a == b
    r(inside(NODE_OPERATION).around(T![==]).single_space());

    // foo . bar . baz => foo.bar.baz
    r(inside(NODE_INDEX_SET).around(T![.]).no_space());

    // {} : 92 => {}: 92
    r(inside(NODE_LAMBDA).before(T![:]).no_space());

    // [1 2 3] => [ 1 2 3 ]
    r(inside(NODE_LIST).after(T!['[']).single_space_or_newline());
    r(inside(NODE_LIST).before(T![']']).single_space_or_newline());

    // {foo = 92;} => { foo = 92; }
    r(inside(NODE_SET).after(T!['{']).single_space_or_newline());
    r(inside(NODE_SET).before(T!['}']).single_space_or_newline());

    rules
}

#[rustfmt::skip]
pub(crate) fn indentation() -> Vec<dsl::IndentRule> {
    let mut rules = Vec::new();
    let mut r = |i: dsl::IndentRule| rules.push(i);
    r(indent(NODE_LIST, LIST_ELEMENTS));
    r(indent(ENTRY_OWNERS, NODE_SET_ENTRY));

    // FIXME: don't force indent if comment is on the first line
    r(indent(NODE_LIST, TOKEN_COMMENT));
    r(indent(ENTRY_OWNERS, TOKEN_COMMENT));
    rules
}

static ENTRY_OWNERS: &'static [SyntaxKind] = &[NODE_SET, NODE_LET_IN];

static LIST_ELEMENTS: &'static [SyntaxKind] = &[
    NODE_VALUE,
    NODE_LIST,
    NODE_SET,
    NODE_INDEX_SET,
    NODE_LAMBDA,
    NODE_STRING,
    NODE_PAREN,
    NODE_IDENT,
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
            before: "{ foo = 1;\nbar = 2; }".into(),
            after: "{\n  foo = 1;\n  bar = 2;\n}".into(),
        }
        .run();
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

        fn run(&self) {
            let name = self.name.as_ref().map(|it| it.as_str()).unwrap_or("");
            let expected = &self.after;
            let actual = &reformat_string(&self.before);
            if expected != actual {
                panic!(
                    "assertion failed({}): wrong formatting\
                     \nBefore:\n{}\n\
                     \nAfter:\n{}\n\
                     \nExpected:\n{}\n",
                    name, self.before, actual, self.after,
                )
            }
            let second_round = &reformat_string(actual);
            if actual != second_round {
                panic!(
                    "assertion failed({}): formatting is not idempotent\
                     \nBefore:\n{}\n\
                     \nAfter:\n{}\n",
                    name, actual, second_round,
                )
            }
        }
    }

    fn run(tests: &[TestCase]) {
        tests.iter().for_each(|it| it.run())
    }
}
