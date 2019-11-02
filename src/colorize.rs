pub struct Colorized<I> {
    inner: I,
    quoted: bool,
    escaped: bool,
}

impl<I: Iterator<Item = u8>> Iterator for Colorized<I> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|char| {
            let result = match char {
                b'\n' if self.quoted => {
                    self.quoted = false;
                    "\u{1b}[0m\n".to_string()
                }
                b'\"' if !self.escaped => {
                    let result = if !self.quoted {
                        "\u{1b}[1;33m\"".to_string()
                    } else {
                        "\"\u{1b}[0m".to_string()
                    };
                    self.quoted = !self.quoted;
                    result
                }
                b'(' | b')' | b'{' | b'}' => format!("\u{1b}[1;36m{}\u{1b}[0m", char::from(char)),
                char => char::from(char).to_string(),
            };
            match char {
                b'\\' => self.escaped = true,
                _ => self.escaped = false,
            }
            result
        })
    }
}

pub fn colorize<I: Iterator<Item = u8>>(contents: I) -> Colorized<I> {
    Colorized {
        inner: contents,
        quoted: false,
        escaped: false,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use colored::*;

    fn test_colorize(input: &str) -> String {
        colorize(input.bytes()).collect::<Vec<_>>().join("")
    }

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
}
