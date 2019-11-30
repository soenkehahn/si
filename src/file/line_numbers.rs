use crate::R;
use source::Source;
use std::path::Path;

pub fn add(file: &Path, mut input: Source<char>) -> R<Source<String>> {
    let max_number_length = max_number_length(file)?;
    let mut line_start = true;
    let mut line_number = 0;
    Ok(Source::new(move || match input.next() {
        Some(char) if line_start => {
            line_number += 1;
            Some(if char == '\n' {
                format!("{} |\n", pad(max_number_length, line_number))
            } else {
                line_start = false;
                format!("{} | {}", pad(max_number_length, line_number), char)
            })
        }
        Some('\n') => {
            line_start = true;
            Some("\n".to_string())
        }
        Some(char) => Some(char.to_string()),
        None => None,
    }))
}

fn max_number_length(file: &Path) -> R<usize> {
    let count = Source::read_utf8_file(file)?.count(|char| char == '\n');
    let max_digits = count.to_string().len();
    Ok(max_digits)
}

fn pad(max_number_length: usize, n: i32) -> String {
    let number_string = n.to_string();
    let padding = if number_string.len() < max_number_length {
        max_number_length - number_string.len()
    } else {
        0
    };
    let padding_string: String = Source::replicate(padding as u32, " ").join("");
    format!("{}{}", padding_string, n)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::*;
    use source::source;
    use std::fs;

    #[test]
    fn shows_line_numbers() -> R<()> {
        let mut setup = setup()?;
        fs::write(setup.tempdir().join("foo"), "foo\nbar\nbaz\n")?;
        setup.run(vec!["foo".to_string()])?;
        assert_eq!(setup.get_section(1), "1 | foo\n2 | bar\n3 | baz\n");
        Ok(())
    }

    #[test]
    fn works_for_consecutive_newlines() -> R<()> {
        let mut setup = setup()?;
        fs::write(setup.tempdir().join("foo"), "foo\n\nbar\n")?;
        setup.run(vec!["foo".to_string()])?;
        assert_eq!(setup.get_section(1), "1 | foo\n2 |\n3 | bar\n");
        Ok(())
    }

    fn run_on_lines(source: Source<&str>) -> R<Vec<String>> {
        let mut setup = setup()?;
        fs::write(setup.tempdir().join("foo"), source.join("\n"))?;
        setup.run(vec!["foo".to_string()])?;
        Ok(setup
            .get_section(1)
            .lines()
            .map(|x| x.to_string())
            .collect::<Vec<String>>())
    }

    mod padding {
        use super::*;

        #[test]
        fn numbers_always_take_up_the_same_number_of_characters() -> R<()> {
            let lines = run_on_lines(source!["foo"; 12].append(""))?;
            assert_eq!(lines[0], " 1 | foo");
            assert_eq!(lines[11], "12 | foo");
            Ok(())
        }

        #[test]
        fn line_number_padding_works_for_empty_lines() -> R<()> {
            let lines = run_on_lines(source!["foo", ""].concat(source!["foo"; 12]))?;
            assert_eq!(lines[1], " 2 |");
            Ok(())
        }

        #[test]
        fn corner_case_1() -> R<()> {
            let lines = run_on_lines(source!["foo"; 9].append(""))?;
            assert_eq!(lines.last(), Some(&"9 | foo".to_string()));
            Ok(())
        }

        #[test]
        fn corner_case_2() -> R<()> {
            let lines = run_on_lines(source!["foo"; 10].append(""))?;
            assert_eq!(lines[8], " 9 | foo");
            assert_eq!(lines[9], "10 | foo");
            Ok(())
        }

        #[test]
        fn corner_case_3() -> R<()> {
            let lines = run_on_lines(source!["foo"; 99].append(""))?;
            assert_eq!(lines.last(), Some(&"99 | foo".to_string()));
            Ok(())
        }

        #[test]
        fn corner_case_4() -> R<()> {
            let lines = run_on_lines(source!["foo"; 100].append(""))?;
            assert_eq!(lines[8], "  9 | foo");
            assert_eq!(lines[9], " 10 | foo");
            assert_eq!(lines[98], " 99 | foo");
            assert_eq!(lines[99], "100 | foo");
            Ok(())
        }
    }
}
