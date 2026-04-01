pub(crate) mod image;
pub(crate) mod registry;
pub(crate) mod string;
pub(crate) mod tag;

pub(crate) fn is_glob_pattern(s: &str) -> bool {
    s.contains('*') || s.contains('?')
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case("a*b", true)]
    #[case("?abc", true)]
    #[case("abc", false)]
    #[case("", false)]
    fn test_is_glob_pattern(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(is_glob_pattern(input), expected);
    }
}
