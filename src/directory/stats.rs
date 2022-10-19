use crate::{Context, R};
use std::fmt::Display;
use std::fs;

pub fn output(context: &mut Context, children: &[fs::DirEntry]) -> R<()> {
    let stats = get_stats(children)?;
    context
        .stdout
        .write_all(format!("{}\n", stats).as_bytes())?;
    Ok(())
}

pub struct Stats {
    entries: usize,
    directories: usize,
    files: usize,
}

fn get_stats(entries: &[fs::DirEntry]) -> R<Stats> {
    let mut stats = Stats {
        entries: 0,
        directories: 0,
        files: 0,
    };
    for entry in entries {
        stats.entries += 1;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            stats.directories += 1;
        } else if file_type.is_file() {
            stats.files += 1;
        }
    }
    Ok(stats)
}

impl Display for Stats {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let entries = match self.entries {
            1 => "entry",
            _ => "entries",
        };
        let directories = match self.directories {
            1 => "directory",
            _ => "directories",
        };
        let files = match self.files {
            1 => "file",
            _ => "files",
        };
        write!(
            formatter,
            "{} {}, {} {}, {} {}",
            self.entries, entries, self.directories, directories, self.files, files
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod stats_pluralization {
    use super::*;
    use crate::test::*;

    #[test]
    fn shows_the_number_of_files_and_directories() -> R<()> {
        let mut setup = setup()?;
        fs::create_dir(setup.tempdir().join("foo"))?;
        fs::write(setup.tempdir().join("bar"), "")?;
        setup.run(vec!["."])?;
        assert_eq!(
            setup.get_section(0),
            format!("2 entries, 1 directory, 1 file\n")
        );
        Ok(())
    }

    #[test]
    fn zeros() {
        assert_eq!(
            format!(
                "{}",
                Stats {
                    entries: 0,
                    directories: 0,
                    files: 0
                }
            ),
            "0 entries, 0 directories, 0 files"
        );
    }

    #[test]
    fn ones() {
        assert_eq!(
            format!(
                "{}",
                Stats {
                    entries: 1,
                    directories: 1,
                    files: 1
                }
            ),
            "1 entry, 1 directory, 1 file"
        );
    }

    #[test]
    fn twos() {
        assert_eq!(
            format!(
                "{}",
                Stats {
                    entries: 2,
                    directories: 2,
                    files: 2
                }
            ),
            "2 entries, 2 directories, 2 files"
        );
    }
}
