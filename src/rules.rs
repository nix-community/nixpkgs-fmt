//! This module contains specific `super::dsl` rules for formatting nix language.
use rnix::parser::nodes::*;

use crate::dsl::{self, inside};

#[rustfmt::skip]
pub(crate) fn spacing() -> Vec<dsl::SpacingRule> {
    let mut rules = Vec::new();
    let mut r = |b: dsl::SpacingRuleBuilder| rules.push(b.into());

    // { a=92; } => { a = 92; }
    r(inside(NODE_SET_ENTRY).around(T![=]).single_space());

    // {a = 92; } => { a = 92; }
    r(inside(NODE_SET).after(T!['{']).single_space());

    // { a = 92;} => { a = 92; }
    r(inside(NODE_SET).before(T!['}']).single_space());
    rules
}

#[cfg(test)]
mod tests {
    use crate::reformat_string;

    /// For convenience, tests in this module are specified inline in comments,
    /// right next to the corresponding rule definition. This test looks at the
    /// text of this file, extracts test cases from comments and checks them.
    #[test]
    fn test_fmt_spacing() {
        let tests = TestCase::collect();
        for test_case in tests {
            let expected = test_case.after;
            let actual = reformat_string(&test_case.before);
            assert_eq!(expected, actual);
            assert_eq!(
                actual,
                reformat_string(&actual),
                "formatting is not idempotent",
            );
        }
    }

    #[derive(Debug)]
    struct TestCase {
        before: String,
        after: String,
    }

    impl TestCase {
        fn try_from(line: &str) -> Option<TestCase> {
            let divisor = line.find("=>")?;
            let before = line[..divisor].trim().to_string();
            let after = line[divisor + 3..].trim().to_string();
            Some(TestCase { before, after })
        }

        fn collect() -> Vec<TestCase> {
            let this_file = include_str!("./rules.rs");
            let res = this_file
                .lines()
                .filter_map(|line| line.find("// ").map(|idx| &line[idx + 3..]))
                .filter_map(TestCase::try_from)
                .collect::<Vec<_>>();

            assert!(res.len() > 0);
            res
        }
    }
}
