mod colorize;
mod line_numbers;

use self::colorize::colorize;
use crate::{write_separator, Context, R};
use source::Source;
use std::fs;
use std::path::PathBuf;

pub fn output(context: &mut Context, file: PathBuf) -> R<()> {
    let size = fs::metadata(&file)?.len();
    writeln!(
        context.stdout,
        "file: {}, {} bytes",
        file.to_string_lossy(),
        size
    )?;
    write_separator(context)?;
    for chunk in line_numbers::add(
        &file,
        colorize(Source::read_utf8_file(&file)?)
            .flat_map(|x| Source::from(x.chars().collect::<Vec<_>>().into_iter())),
    )? {
        write!(context.stdout, "{}", chunk)?;
    }
    Ok(())
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
        setup.run(vec!["foo"])?;
        assert!(setup
            .get_section(1)
            .ends_with(&format!("foo {}", "\"bar\"".yellow().bold())));
        Ok(())
    }

    #[test]
    fn includes_a_stats_section_about_the_file() -> R<()> {
        let mut setup = setup()?;
        fs::write(setup.tempdir().join("foo"), "foo")?;
        setup.run(vec!["foo"])?;
        assert_eq!(get_line(setup.stdout(), 0), "file: foo, 3 bytes");
        assert_eq!(
            get_line(setup.stdout(), 1),
            Source::replicate(TEST_TERMINAL_WIDTH.unwrap() as u32, "─")
                .join("")
                .yellow()
                .bold()
                .to_string()
        );
        Ok(())
    }

    #[test]
    fn does_not_crash_for_invalid_utf_8() -> R<()> {
        let mut setup = setup()?;
        fs::write(setup.tempdir().join("foo"), [0xc3, 0x28])?;
        setup.run(vec!["foo"])?;
        Ok(())
    }
}
