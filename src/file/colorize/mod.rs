mod peekable;

use self::peekable::Peekable;
use crate::stream::Stream;
use colored::*;

pub fn colorize(contents: Stream<char>) -> Stream<String> {
    Stream::from(Parser {
        inner: Peekable::new(contents),
    })
}

pub struct Parser {
    inner: Peekable<char>,
}

type ParseResult<A> = Result<A, ()>;

impl Iterator for Parser {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        self.next_chunk()
    }
}

impl Parser {
    fn next_chunk(&mut self) -> Option<String> {
        self.word()
            .or_else(|()| {
                self.char(|char| "(){}[]".chars().any(|c| c == char))
                    .map(|chunk| chunk.to_string().cyan().bold().to_string())
            })
            .or_else(|()| self.number_word())
            .or_else(|()| self.quoted().map(|x| x.yellow().bold().to_string()))
            .map(Some)
            .unwrap_or_else(|()| self.any_char())
    }

    fn any_char(&mut self) -> Option<String> {
        self.inner.next().map(|char| char.to_string())
    }

    fn word(&mut self) -> ParseResult<String> {
        let mut result = self.char(|c| c.is_alphabetic())?.to_string();
        for c in self.parse_zero_or_more(|this| this.char(|c| c.is_alphanumeric())) {
            result.push(c)
        }
        Ok(result)
    }

    fn quoted(&mut self) -> ParseResult<String> {
        self.char(|char| char == '\"')?;
        let result = self.parse_zero_or_more(|this| this.quoted_char());
        match self.char(|char| char == '\"') {
            Ok(_) => Ok(format!("\"{}\"", result.collect::<String>())),
            Err(()) => Ok(format!("\"{}", result.collect::<String>())),
        }
    }

    fn quoted_char(&mut self) -> ParseResult<String> {
        self.escaped_char().or_else(|()| {
            self.char(|char| char != '\"' && char != '\n')
                .map(|char| char.to_string())
        })
    }

    fn escaped_char(&mut self) -> ParseResult<String> {
        let a = self.char(|c| c == '\\')?;
        let b = self.char(|_| true)?;
        Ok(format!("{}{}", a, b))
    }

    fn char<F: Fn(char) -> bool>(&mut self, predicate: F) -> ParseResult<char> {
        match self.inner.peek() {
            Some(char) => {
                if predicate(char) {
                    self.inner.next();
                    Ok(char)
                } else {
                    Err(())
                }
            }
            None => Err(()),
        }
    }

    fn number_word(&mut self) -> ParseResult<String> {
        self.parse_one_or_more(|this| {
            this.parse_one_or_more(|this| this.digit())
                .map(|vec| vec.collect())
                .map(|x: String| x.red().bold().to_string())
                .or_else(|()| {
                    this.parse_one_or_more(|this| this.char(|c| c.is_alphabetic()))
                        .map(|vec| vec.collect())
                })
        })
        .map(|vec| vec.collect())
    }

    fn digit(&mut self) -> ParseResult<char> {
        self.char(|char| char.is_digit(10))
    }

    fn parse_zero_or_more<A>(
        &mut self,
        parse_element: fn(&mut Parser) -> ParseResult<A>,
    ) -> impl Iterator<Item = A> {
        let mut result = vec![];
        while let Ok(element) = parse_element(self) {
            result.push(element);
        }
        result.into_iter()
    }
    fn parse_one_or_more<A>(
        &mut self,
        parse_element: fn(&mut Parser) -> ParseResult<A>,
    ) -> ParseResult<impl Iterator<Item = A>> {
        let first = parse_element(self)?;
        let mut result = vec![first];
        while let Ok(element) = parse_element(self) {
            result.push(element);
        }
        Ok(result.into_iter())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_colorize(input: &str) -> String {
        let vec: Vec<char> = input.to_string().to_owned().chars().collect();
        colorize(Stream::from(vec.into_iter())).to_vec().join("")
    }

    mod quotes {
        use super::*;

        #[test]
        fn colorizes_double_quoted_strings() {
            assert_eq!(
                test_colorize("f\"o\"o"),
                format!("f{}o", "\"o\"".yellow().bold())
            );
        }

        #[test]
        fn allows_to_escape_double_quotes() {
            assert_eq!(
                test_colorize(r#"a"b\"c\"d"e"#),
                format!("a{}e", r#""b\"c\"d""#.yellow().bold())
            );
        }

        #[test]
        fn resets_at_newlines() {
            assert_eq!(
                test_colorize("foo\"bar\nf\"o\"o"),
                format!(
                    "foo{}\nf{}o",
                    "\"bar".yellow().bold(),
                    "\"o\"".yellow().bold()
                )
            );
        }
    }

    #[test]
    fn colorizes_round_brackets() {
        assert_eq!(
            test_colorize("(foo)"),
            format!("{}foo{}", "(".cyan().bold(), ")".cyan().bold())
        );
    }

    #[test]
    fn colorizes_curly_brackets() {
        assert_eq!(
            test_colorize("{foo}"),
            format!("{}foo{}", "{".cyan().bold(), "}".cyan().bold())
        );
    }

    #[test]
    fn colorizes_square_brackets() {
        assert_eq!(
            test_colorize("[foo]"),
            format!("{}foo{}", "[".cyan().bold(), "]".cyan().bold())
        );
    }

    mod numbers {
        use super::*;

        #[test]
        fn colorizes_numbers() {
            assert_eq!(
                test_colorize("foo 42 bar"),
                format!("foo {} bar", "42".red().bold().to_string())
            );
        }

        #[test]
        fn works_for_numbers_at_the_end_of_lines() {
            assert_eq!(
                test_colorize("23\n42"),
                format!(
                    "{}\n{}",
                    "23".red().bold().to_string(),
                    "42".red().bold().to_string()
                )
            );
        }

        #[test]
        fn does_not_colorize_numbers_within_identifiers() {
            assert_eq!(test_colorize("foo42bar"), format!("foo42bar"));
        }

        #[test]
        fn does_colorize_numbers_within_identifiers_when_starting_with_a_digit() {
            assert_eq!(
                test_colorize("42foo23"),
                format!(
                    "{}foo{}",
                    "42".red().bold().to_string(),
                    "23".red().bold().to_string()
                )
            );
        }
    }
}
