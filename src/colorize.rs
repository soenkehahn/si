pub struct Colorized<I> {
    inner: I,
    quoted: bool,
}

impl<I: Iterator<Item = u8>> Iterator for Colorized<I> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|char| match char {
            b'\n' if self.quoted => {
                self.quoted = false;
                "\u{1b}[0m\n".to_string()
            }
            b'\"' => {
                let result = if !self.quoted {
                    "\u{1b}[1;33m\"".to_string()
                } else {
                    "\"\u{1b}[0m".to_string()
                };
                self.quoted = !self.quoted;
                result
            }
            b'(' => "\u{1b}[1;36m(\u{1b}[0m".to_string(),
            b')' => "\u{1b}[1;36m)\u{1b}[0m".to_string(),
            char => char::from(char).to_string(),
        })
    }
}

pub fn colorize<I>(contents: I) -> Colorized<I> {
    Colorized {
        inner: contents,
        quoted: false,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use colored::*;

    #[test]
    fn colorizes_double_quoted_strings() {
        assert_eq!(
            colorize(b"f\"o\"o".to_vec().into_iter())
                .collect::<Vec<_>>()
                .join(""),
            format!("f{}o", "\"o\"".yellow().bold())
        );
    }

    #[test]
    fn resets_at_newlines() {
        assert_eq!(
            colorize(b"foo\"bar\nf\"o\"o".to_vec().into_iter())
                .collect::<Vec<_>>()
                .join(""),
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
            colorize(b"(foo)".to_vec().into_iter())
                .collect::<Vec<_>>()
                .join(""),
            format!("{}foo{}", "(".cyan().bold(), ")".cyan().bold())
        );
    }
}
