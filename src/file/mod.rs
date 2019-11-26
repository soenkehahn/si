mod colorize;

use self::colorize::colorize;
use crate::stream::Stream;
use crate::{write_separator, R};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub fn output(stdout: &mut dyn Write, file: PathBuf) -> R<()> {
    let size = fs::metadata(&file)?.len();
    writeln!(stdout, "file: {}, {} bytes", file.to_string_lossy(), size)?;
    write_separator(stdout)?;
    for chunk in add_line_numbers(
        colorize(Stream::read_utf8_file(&file)?)
            .flat_map(|x| Stream::from(x.chars().collect::<Vec<_>>().into_iter())),
    ) {
        write!(stdout, "{}", chunk)?;
    }
    Ok(())
}

fn add_line_numbers(mut input: Stream<char>) -> Stream<String> {
    let mut line_start = true;
    let mut line_number = 0;
    Stream::new(move || match input.next() {
        Some(char) if line_start => {
            line_number += 1;
            Some(if char == '\n' {
                format!("{} |\n", line_number)
            } else {
                line_start = false;
                format!("{} | {}", line_number, char)
            })
        }
        Some('\n') => {
            line_start = true;
            Some("\n".to_string())
        }
        Some(char) => Some(char.to_string()),
        None => None,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::*;
    use colored::*;

    #[test]
    fn colorizes_file_contents() -> R<()> {
        let mut setup = setup()?;
        fs::write(setup.tempdir().join("foo"), "foo \"bar\"")?;
        setup.run(vec!["foo".to_string()])?;
        assert!(drop_stats(setup.stdout()).ends_with(&format!("foo {}", "\"bar\"".yellow().bold())));
        Ok(())
    }

    #[test]
    fn includes_a_stats_section_about_the_file() -> R<()> {
        let mut setup = setup()?;
        fs::write(setup.tempdir().join("foo"), "foo")?;
        setup.run(vec!["foo".to_string()])?;
        assert_eq!(get_line(setup.stdout(), 0), "file: foo, 3 bytes");
        assert_eq!(
            get_line(setup.stdout(), 1),
            "---".yellow().bold().to_string()
        );
        Ok(())
    }

    mod line_numbers {
        use super::*;

        #[test]
        fn shows_line_numbers() -> R<()> {
            let mut setup = setup()?;
            fs::write(setup.tempdir().join("foo"), "foo\nbar\nbaz\n")?;
            setup.run(vec!["foo".to_string()])?;
            assert_eq!(drop_stats(setup.stdout()), "1 | foo\n2 | bar\n3 | baz\n");
            Ok(())
        }

        #[test]
        fn works_for_consecutive_newlines() -> R<()> {
            let mut setup = setup()?;
            fs::write(setup.tempdir().join("foo"), "foo\n\nbar\n")?;
            setup.run(vec!["foo".to_string()])?;
            assert_eq!(drop_stats(setup.stdout()), "1 | foo\n2 |\n3 | bar\n");
            Ok(())
        }
    }
}
