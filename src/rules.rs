//! This module contains specific `super::dsl` rules for formatting nix language.
use crate::dsl::{self, inside};
use rnix::parser::nodes::*;

#[rustfmt::skip]
pub(crate) fn spacing() -> Vec<dsl::SpacingRule> {
    let mut rules = Vec::new();
    rules.push(
        inside(NODE_SET_ENTRY)
            .around(T![=])  // { a=92; }
            .single_space() // -----------
            .into(),        // { a = 92; }
    );
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
        }
    }

    #[derive(Debug)]
    struct TestCase {
        before: String,
        after: String,
    }

    impl TestCase {
        fn try_from(group: &[&str]) -> Option<TestCase> {
            let divisor = group.iter().position(|line| line.contains("----"))?;
            let before = group[..divisor].join("\n");
            let after = group[divisor + 1..].join("\n");
            Some(TestCase { before, after })
        }

        fn collect() -> Vec<TestCase> {
            let this_file = include_str!("./rules.rs");
            let lines = this_file
                .lines()
                .map(|line| line.find("// ").map(|idx| &line[idx + 3..]));

            let mut res = vec![];
            let mut group = vec![];
            for line in lines {
                match line {
                    Some(it) => group.push(it),
                    None => {
                        res.extend(TestCase::try_from(&group));
                        group.clear()
                    }
                }
            }
            res.extend(TestCase::try_from(&group));
            assert!(res.len() > 0);
            res
        }
    }
}
