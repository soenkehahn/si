use crate::{write_separator, R};
use colored::*;
use std::fmt::Display;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub fn output(stdout: &mut dyn Write, directory: PathBuf) -> R<()> {
    let mut children = directory.read_dir()?.collect::<Result<Vec<_>, _>>()?;
    children.sort_unstable_by(|a, b| a.path().file_name().cmp(&b.path().file_name()));
    let stats = get_stats(&children)?;
    stdout.write_all(format!("{}\n", stats).as_bytes())?;
    write_separator(stdout)?;
    for child in children {
        let path = child
            .path()
            .file_name()
            .unwrap_or_else(|| {
                panic!(format!(
                    "directory entry has no last component: {:?}",
                    child
                ))
            })
            .to_string_lossy()
            .into_owned();
        let list_entry = if child.path().is_dir() {
            format!("{}/", path.blue().bold())
        } else {
            path
        };
        stdout.write_all(format!("{}\n", list_entry).as_bytes())?;
    }
    Ok(())
}

struct Stats {
    entries: usize,
    directories: usize,
    files: usize,
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::*;
    use itertools::Itertools;
    use strip_ansi_escapes::strip;

    #[test]
    fn single_file() -> R<()> {
        let mut setup = setup()?;
        fs::write(setup.tempdir().join("foo"), "")?;
        setup.run(vec![".".to_string()])?;
        assert_eq!(drop_stats(setup.stdout()), "foo\n");
        Ok(())
    }

    #[test]
    fn multiple_files_sorted() -> R<()> {
        let mut setup = setup()?;
        fs::write(setup.tempdir().join("foo"), "")?;
        fs::write(setup.tempdir().join("bar"), "")?;
        setup.run(vec![".".to_string()])?;
        assert_eq!(drop_stats(setup.stdout()), "bar\nfoo\n");
        Ok(())
    }

    #[test]
    fn lists_working_directory_when_no_argument_given() -> R<()> {
        let mut setup = setup()?;
        fs::write(setup.tempdir().join("foo"), "")?;
        fs::write(setup.tempdir().join("bar"), "")?;
        setup.run(vec![])?;
        assert_eq!(drop_stats(setup.stdout()), "bar\nfoo\n");
        Ok(())
    }

    #[test]
    fn lists_directories_with_a_trailing_slash() -> R<()> {
        let mut setup = setup()?;
        fs::create_dir(setup.tempdir().join("foo"))?;
        setup.run(vec![".".to_string()])?;
        assert_eq!(strip(drop_stats(setup.stdout()))?, b"foo/\n");
        Ok(())
    }

    #[test]
    fn lists_directories_in_blue() -> R<()> {
        let mut setup = setup()?;
        fs::create_dir(setup.tempdir().join("foo"))?;
        setup.run(vec![".".to_string()])?;
        assert_eq!(
            drop_stats(setup.stdout()),
            format!("{}/\n", "foo".blue().bold())
        );
        Ok(())
    }

    #[test]
    fn shows_the_number_of_files_and_directories() -> R<()> {
        let mut setup = setup()?;
        fs::create_dir(setup.tempdir().join("foo"))?;
        fs::write(setup.tempdir().join("bar"), "")?;
        setup.run(vec![".".to_string()])?;
        assert_eq!(
            setup.stdout().split("\n").take(2).join("\n"),
            format!("2 entries, 1 directory, 1 file\n{}", "---".yellow().bold())
        );
        Ok(())
    }

    mod stats_pluralization {
        use super::*;
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
}
