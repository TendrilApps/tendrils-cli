//! Most tests are covered by the `resolve_path_variables_tests`, the tests
//! here are to cover other edge cases

use crate::tendril::parse_env_variables;

#[test]
fn empty_brackets_returns_empty_brackets() {
    let actual = parse_env_variables("empty<>var");

    assert_eq!(actual, ["<>"])
}
