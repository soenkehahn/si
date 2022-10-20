use colored::*;
use source::Source;

pub fn colorize(contents: Source<char>) -> Source<String> {
    let mut parser = Parser { inner: contents };
    Source::new(move || parser.next_chunk())
}

pub struct Parser {
    inner: Source<char>,
}

type ParseResult<A> = Result<A, ()>;

impl Parser {
    fn next_chunk(&mut self) -> Option<String> {
        self.word()
            .or_else(|()| {
                self.char(|char| "(){}[]".chars().any(|c| c == char))
                    .map(|chunk| chunk.to_string().cyan().bold().to_string())
            })
            .or_else(|()| self.number_word())
            .or_else(|()| self.quoted('\"').map(|x| x.yellow().bold().to_string()))
            .or_else(|()| self.quoted('\'').map(|x| x.yellow().bold().to_string()))
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

    fn quoted(&mut self, quote_char: char) -> ParseResult<String> {
        self.char(|char| char == quote_char)?;
        let result = self.parse_zero_or_more(|this| this.quoted_char(quote_char));
        match self.char(|char| char == quote_char) {
            Ok(_) => Ok(format!(
                "{}{}{}",
                quote_char,
                result.collect::<String>(),
                quote_char
            )),
            Err(()) => Ok(format!("{}{}", quote_char, result.collect::<String>())),
        }
    }

    fn quoted_char(&mut self, quote_char: char) -> ParseResult<String> {
        self.escaped_char().or_else(|()| {
            self.char(|char| char != quote_char && char != '\n')
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
        self.char(|char| char.is_ascii_digit())
    }

    fn parse_zero_or_more<A, F: Fn(&mut Parser) -> ParseResult<A>>(
        &mut self,
        parse_element: F,
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
        colorize(Source::from(vec.into_iter())).join("")
    }

    macro_rules! replicate_tests {
        ($parameter: ident = { $snippet: ident : $argument: tt, $($rest: tt)* } $($tests: item)*) => {
            replicate_tests!($parameter = { $snippet : $argument } $($tests)*);
            replicate_tests!($parameter = { $($rest)* } $($tests)*);
        };
        ($parameter: ident = { $snippet: ident : $argument: expr } $($tests: item)*) => {
            mod $snippet {
                use super::*;

                const $parameter: char = $argument;

                $($tests)*
            }
        }
    }

    mod quotes {
        use super::*;

        replicate_tests! {
            QUOTE_CHAR = {
              double_quotes: '\"',
              single_quotes: '\''
            }

            fn convert_quotes(string: &str) -> String {
                string.replace("\"", &QUOTE_CHAR.to_string())
            }

            #[test]
            fn colorizes_quoted_strings() {
                assert_eq!(
                    test_colorize(&convert_quotes("f\"o\"o")),
                    format!("f{}o", convert_quotes("\"o\"").yellow().bold())
                );
            }

            #[test]
            fn allows_to_escape_double_quotes() {
                assert_eq!(
                    test_colorize(&convert_quotes(r#"a"b\"c\"d"e"#)),
                    convert_quotes(&format!("a{}e", r#""b\"c\"d""#.yellow().bold()))
                );
            }

            #[test]
            fn resets_at_newlines() {
                assert_eq!(
                    test_colorize(&convert_quotes("foo\"bar\nf\"o\"o")),
                    convert_quotes(
                        &format!(
                            "foo{}\nf{}o",
                            "\"bar".yellow().bold(),
                            "\"o\"".yellow().bold()
                        )
                    )
                );
            }
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
