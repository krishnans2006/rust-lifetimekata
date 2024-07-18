use require_lifetimes::require_lifetimes;

#[derive(Debug, PartialEq, Eq)]
enum MatcherToken<'a> {
    /// This is just text without anything special.
    RawText(&'a str),
    /// This is when text could be any one of multiple
    /// strings. It looks like `(one|two|three)`, where
    /// `one`, `two` or `three` are the allowed strings.
    OneOfText(Vec<&'a str>),
    /// This is when you're happy to accept any single character.
    /// It looks like `.`
    WildCard,
}

#[derive(Debug, PartialEq, Eq)]
struct Matcher<'a> {
    /// This is the actual text of the matcher
    text: &'a str,
    /// This is a vector of the tokens inside the expression.
    tokens: Vec<MatcherToken<'a>>,
    /// This keeps track of the most tokens that this matcher has matched.
    most_tokens_matched: usize,
}

impl<'a, 'b> Matcher<'a> {
    /// This should take a string reference, and return
    /// an `Matcher` which has parsed that reference.
    #[require_lifetimes]
    fn new(text: &'a str) -> Option<Matcher<'a>> {
        let mut tokens: Vec<MatcherToken> = Vec::new();

        let mut in_text_block = false;
        let mut text_start: usize = 0;

        let mut in_or_block = false;
        let mut or_options: Vec<&str> = Vec::new();

        for (i, c) in text.chars().enumerate() {
            if c == '.' {
                if in_text_block {
                    tokens.push(MatcherToken::RawText(&text[text_start..i])); // Not including i (".")
                    in_text_block = false;
                }
                tokens.push(MatcherToken::WildCard);
            } else if c == '(' {
                if in_or_block {
                    return None;
                }
                if in_text_block {
                    tokens.push(MatcherToken::RawText(&text[text_start..i]));  // Not including i ("(")
                    in_text_block = false;
                }
                in_or_block = true;
            } else if c == ')' {
                if !in_or_block {
                    return None;
                }
                if in_text_block {
                    or_options.push(&text[text_start..i]);  // Not including i (")")
                }
                in_or_block = false;
                in_text_block = false;
                tokens.push(MatcherToken::OneOfText(or_options));
                or_options = Vec::new();
            } else if c == '|' {
                if !in_or_block {
                    return None;
                }
                if !in_text_block {
                    return None;
                }
                or_options.push(&text[text_start..i]);  // Not including i ("|")
                in_text_block = false;
            } else {
                if !in_text_block {
                    in_text_block = true;
                    text_start = i;
                }
            }
        }

        if in_or_block {
            return None;
        }

        return Some(Matcher {
            text,
            tokens,
            most_tokens_matched: 0
        });
    }

    /// This should take a string, and return a vector of tokens, and the corresponding part
    /// of the given string. For examples, see the test cases below.
    #[require_lifetimes]
    fn match_string(&'_ mut self, string: &'b str) -> Vec<(&'b MatcherToken, &'b str)> {
        self.most_tokens_matched = 0;

        let mut string_pointer = 0;
        let mut matches: Vec<(&'b MatcherToken, &'b str)> = Vec::new();

        for token in &self.tokens {
            match token {
                MatcherToken::RawText(text) => {
                    if !(&string[string_pointer..].starts_with(text)) {
                        return matches;
                    }
                    let new_string_pointer = string_pointer + text.len();
                    matches.push((&token, &string[string_pointer..new_string_pointer]));
                    self.most_tokens_matched += 1;
                    string_pointer = new_string_pointer;
                },
                MatcherToken::OneOfText(options) => {
                    for text in options {
                        if (&string[string_pointer..]).starts_with(text) {
                            let new_string_pointer = string_pointer + text.len();
                            matches.push((&token, &string[string_pointer..new_string_pointer]));
                            self.most_tokens_matched += 1;
                            string_pointer = new_string_pointer;
                            break;
                        }
                        // At this point, none of the options matched...
                        return matches;
                    }
                },
                MatcherToken::WildCard => {
                    matches.push((&token, &string[string_pointer..string_pointer+1]));
                    self.most_tokens_matched += 1;
                    string_pointer += 1;
                }
            }
            if string_pointer > string.len() {
                return matches;
            }
        }

        return matches;
    }
}

fn main() {
    unimplemented!()
}

#[cfg(test)]
mod test {
    use super::{Matcher, MatcherToken};
    #[test]
    fn simple_test() {
        let match_string = "abc(d|e|f).".to_string();
        let mut matcher = Matcher::new(&match_string).unwrap();

        println!("{matcher:?}");

        assert_eq!(matcher.most_tokens_matched, 0);

        {
            let candidate1 = "abcge".to_string();
            let result = matcher.match_string(&candidate1);
            assert_eq!(result, vec![(&MatcherToken::RawText("abc"), "abc"),]);
            assert_eq!(matcher.most_tokens_matched, 1);
        }

        {
            // Change 'e' to '💪' if you want to test unicode.
            let candidate1 = "abcde".to_string();
            let result = matcher.match_string(&candidate1);
            assert_eq!(
                result,
                vec![
                    (&MatcherToken::RawText("abc"), "abc"),
                    (&MatcherToken::OneOfText(vec!["d", "e", "f"]), "d"),
                    (&MatcherToken::WildCard, "e") // or '💪'
                ]
            );
            assert_eq!(matcher.most_tokens_matched, 3);
        }
    }

    #[test]
    fn broken_matcher() {
        let match_string = "abc(d|e|f.".to_string();
        let matcher = Matcher::new(&match_string);
        assert_eq!(matcher, None);
    }
}
