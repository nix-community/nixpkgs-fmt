//! This module contains specific `super::dsl` rules for formatting nix language.
use rnix::parser::nodes::*;

use crate::dsl::{self, inside};

#[rustfmt::skip]
pub(crate) fn spacing() -> Vec<dsl::SpacingRule> {
    let mut rules = Vec::new();
    let mut r = |b: dsl::SpacingRuleBuilder| rules.push(b.into());

    // Note: comments with fat arrow are tests!

    // { a=92; } => { a = 92; }
    r(inside(NODE_SET_ENTRY).around(T![=]).single_space());

    // { a = 92 ; } => { a = 92; }
    r(inside(NODE_SET_ENTRY).before(T![;]).no_space());

    // {a = 92; } => { a = 92; }
    r(inside(NODE_SET).after(T!['{']).single_space());

    // { a = 92;} => { a = 92; }
    r(inside(NODE_SET).before(T!['}']).single_space());

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

    rules
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use crate::reformat_string;

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
                        after: fs::read_to_string(dir.join(after_name)).unwrap(),
                    };
                    res.push(test_case);
                }
            }
            assert!(res.len() > 0);
            res
        }
    }

    fn run(tests: &[TestCase]) {
        for test_case in tests {
            let name = test_case.name.as_ref().map(|it| it.as_str()).unwrap_or("");
            let expected = &test_case.after;
            let actual = &reformat_string(&test_case.before);
            if expected != actual {
                panic!(
                    "assertion failed: wrong formatting.\
                     \nBefore:\n{}\n\
                     \nAfter:\n{}\n\
                     \nExpected:\n{}\n",
                    test_case.before, actual, test_case.after,
                )
            }
            assert_eq!(expected, actual, "\nwrong formatting\n{}\n", name);
            assert_eq!(
                actual,
                &reformat_string(actual),
                "\nformatting is not idempotent\n{}\n",
                name,
            );
        }
    }
}
