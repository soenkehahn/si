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
                let mut result: Vec<u8> = vec![];
                if !self.quoted {
                    for c in "\u{1b}[1;33m".bytes() {
                        result.push(c)
                    }
                }
                result.push(b'\"');
                if self.quoted {
                    for c in "\u{1b}[0m".bytes() {
                        result.push(c)
                    }
                }
                self.quoted = !self.quoted;
                result
            }
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
}
