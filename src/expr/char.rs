use super::core::{CoreExpr, ExprExtension, TerminalMatcher};

#[derive(Debug)]
pub enum CharRule {
    Char(char),
    Alpha,
    Num,
    Whitespace,
}

#[derive(Debug)]
struct CharMatcher {
    rule: CharRule,
}

impl TerminalMatcher for CharMatcher {
    type Terminal = char;

    fn matches(&self, terminal: &Self::Terminal) -> bool {
        match &self.rule {
            CharRule::Char(c) => terminal == c,
            CharRule::Alpha => terminal.is_alphabetic(),
            CharRule::Num => terminal.is_numeric(),
            CharRule::Whitespace => terminal.is_whitespace(),
        }
    }
}

#[derive(Debug)]
pub enum CharExpr {
    Char(char),
    Alpha,
    Num,
    Whitespace,
    Sequence(Vec<CharExpr>),
    Choice(Vec<CharExpr>),
    Repeat(Box<CharExpr>),
    Null,
}

impl ExprExtension<'_, CharMatcher> for CharExpr {
    fn into_core_expr(&self) -> CoreExpr<CharMatcher> {
        match self {
            CharExpr::Char(c) => CoreExpr::Terminal(CharMatcher { rule: CharRule::Char(*c) }),
            CharExpr::Alpha => CoreExpr::Terminal(CharMatcher { rule: CharRule::Alpha }),
            CharExpr::Num => CoreExpr::Terminal(CharMatcher { rule: CharRule::Num }),
            CharExpr::Whitespace => CoreExpr::Terminal(CharMatcher {
                rule: CharRule::Whitespace,
            }),
            CharExpr::Sequence(exprs) => CoreExpr::Sequence(exprs.iter().map(|expr| expr.into_core_expr()).collect()),
            CharExpr::Choice(exprs) => CoreExpr::Choice(exprs.iter().map(|expr| expr.into_core_expr()).collect()),
            CharExpr::Repeat(expr) => CoreExpr::Repeat(Box::new(expr.into_core_expr())),
            CharExpr::Null => CoreExpr::Null,
        }
    }
}

impl std::ops::Add for CharExpr {
    type Output = CharExpr;

    fn add(self, other: CharExpr) -> CharExpr {
        CharExpr::Sequence(vec![self, other])
    }
}

impl std::ops::BitOr for CharExpr {
    type Output = CharExpr;

    fn bitor(self, other: CharExpr) -> CharExpr {
        CharExpr::Choice(vec![self, other])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn as_slice(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    #[test]
    fn test_single_terminal() {
        let expr = CharExpr::Char('a').into_core_expr();
        let matcher = expr.compile();
        assert!(matcher.match_sequence(&as_slice("a")));
        assert!(!matcher.match_sequence(&as_slice("b")));
    }

    #[test]
    fn test_sequence_of_terminal() {
        let expr = (CharExpr::Char('a') + CharExpr::Char('b')).into_core_expr();
        let matcher = expr.compile();
        assert!(matcher.match_sequence(&as_slice("ab")));
        assert!(!matcher.match_sequence(&as_slice("a")));
        assert!(!matcher.match_sequence(&as_slice("aa")));
        assert!(!matcher.match_sequence(&as_slice("abc")));
        assert!(!matcher.match_sequence(&as_slice("ba")));
    }

    #[test]
    fn test_choice_of_terminal() {
        let expr = (CharExpr::Char('a') | CharExpr::Char('b')).into_core_expr();
        let matcher = expr.compile();
        assert!(matcher.match_sequence(&as_slice("a")));
        assert!(matcher.match_sequence(&as_slice("b")));
        assert!(!matcher.match_sequence(&as_slice("c")));
    }

    #[test]
    fn test_choice_of_sequence() {
        let expr = (CharExpr::Char('a') + CharExpr::Char('b') | CharExpr::Char('c')).into_core_expr();
        let matcher = expr.compile();
        assert!(matcher.match_sequence(&as_slice("ab")));
        assert!(matcher.match_sequence(&as_slice("c")));
        assert!(!matcher.match_sequence(&as_slice("a")));
        assert!(!matcher.match_sequence(&as_slice("b")));
        assert!(!matcher.match_sequence(&as_slice("ac")));
        assert!(!matcher.match_sequence(&as_slice("bc")));
    }

    #[test]
    fn test_repeat() {
        let expr = CharExpr::Repeat(Box::new(CharExpr::Char('a'))).into_core_expr();
        let matcher = expr.compile();
        println!("{:?}", matcher);
        assert!(matcher.match_sequence(&as_slice("")));
        assert!(matcher.match_sequence(&as_slice("a")));
        assert!(matcher.match_sequence(&as_slice("aa")));
        assert!(matcher.match_sequence(&as_slice("aaa")));
        assert!(!matcher.match_sequence(&as_slice("b")));
        assert!(!matcher.match_sequence(&as_slice("ab")));
    }

    #[test]
    fn test_null() {
        let expr = CharExpr::Null.into_core_expr();
        let matcher = expr.compile();
        assert!(matcher.match_sequence(&as_slice("")));
        assert!(!matcher.match_sequence(&as_slice("a")));
    }

    #[test]
    fn test_repeat_of_sequence() {
        let expr = CharExpr::Repeat(Box::new(CharExpr::Char('a') + CharExpr::Char('b'))).into_core_expr();
        let matcher = expr.compile();
        assert!(matcher.match_sequence(&as_slice("")));
        assert!(matcher.match_sequence(&as_slice("ab")));
        assert!(matcher.match_sequence(&as_slice("abab")));
        assert!(matcher.match_sequence(&as_slice("ababab")));
        assert!(!matcher.match_sequence(&as_slice("a")));
        assert!(!matcher.match_sequence(&as_slice("b")));
        assert!(!matcher.match_sequence(&as_slice("aba")));
        assert!(!matcher.match_sequence(&as_slice("abb")));
    }

    #[test]
    fn test_repeat_of_choice() {
        let expr = CharExpr::Repeat(Box::new(CharExpr::Char('a') | CharExpr::Char('b'))).into_core_expr();
        let matcher = expr.compile();
        assert!(matcher.match_sequence(&as_slice("")));
        assert!(matcher.match_sequence(&as_slice("a")));
        assert!(matcher.match_sequence(&as_slice("b")));
        assert!(matcher.match_sequence(&as_slice("aa")));
        assert!(matcher.match_sequence(&as_slice("ab")));
        assert!(matcher.match_sequence(&as_slice("ba")));
        assert!(matcher.match_sequence(&as_slice("bb")));
        assert!(!matcher.match_sequence(&as_slice("c")));
        assert!(!matcher.match_sequence(&as_slice("ac")));
        assert!(!matcher.match_sequence(&as_slice("abc")));
    }

    // #[test]
    // fn test_choice_of_repeat() {
    //     let expr = (CharExpr::Repeat(Box::new(CharExpr::Char('a'))) | CharExpr::Repeat(Box::new(CharExpr::Char('b')))).into_core_expr();
    //     let matcher = expr.compile();
    //     assert!(matcher.match_sequence(&as_slice("")));
    //     assert!(matcher.match_sequence(&as_slice("a")));
    //     assert!(matcher.match_sequence(&as_slice("aa")));
    //     assert!(matcher.match_sequence(&as_slice("aaa")));
    //     assert!(matcher.match_sequence(&as_slice("b")));
    //     assert!(!matcher.match_sequence(&as_slice("ab")));
    //     assert!(!matcher.match_sequence(&as_slice("bbbba")));
    // }
}
