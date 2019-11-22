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
    for chunk in colorize(Stream::read_utf8_file(&file)?)
        .flat_map(|x| Stream::from_iterator(x.chars().collect::<Vec<_>>().into_iter()))
    {
        write!(stdout, "{}", chunk)?;
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
}
