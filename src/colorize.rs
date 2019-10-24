pub struct Colorized<I> {
    inner: I,
    quoted: bool,
}

impl<I: Iterator<Item = u8>> Iterator for Colorized<I> {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|char| match char {
            b'\n' if self.quoted => {
                self.quoted = false;
                "\u{1b}[0m\n".bytes().collect()
            }
            b'\"' => {
                let result = if !self.quoted {
                    "\u{1b}[1;33m\""
                } else {
                    "\"\u{1b}[0m"
                }
                .bytes()
                .collect();
                self.quoted = !self.quoted;
                result
            }
            b'(' => "\u{1b}[1;36m(\u{1b}[0m".bytes().collect(),
            b')' => "\u{1b}[1;36m)\u{1b}[0m".bytes().collect(),
            char => vec![char],
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
                .into_iter()
                .map(|x| String::from_utf8_lossy(&x).into_owned())
                .collect::<Vec<_>>()
                .join(""),
            format!("f{}o", "\"o\"".yellow().bold())
        );
    }

    #[test]
    fn resets_at_newlines() {
        assert_eq!(
            colorize(b"foo\"bar\nf\"o\"o".to_vec().into_iter())
                .into_iter()
                .map(|x| String::from_utf8_lossy(&x).into_owned())
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
                .into_iter()
                .map(|x| String::from_utf8_lossy(&x).into_owned())
                .collect::<Vec<_>>()
                .join(""),
            format!("{}foo{}", "(".cyan().bold(), ")".cyan().bold())
        );
    }
}
