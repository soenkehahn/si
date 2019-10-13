use colored::*;
use pager::Pager;
use std::fmt::Display;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

type R<A> = Result<A, Box<dyn std::error::Error>>;

fn wrap_main(action: fn(args: &mut dyn Iterator<Item = String>, stdout: &mut dyn Write) -> R<()>) {
    Pager::with_pager("less -RFX").setup();
    let mut stdout = std::io::stdout();
    let exitcode = match action(&mut std::env::args(), &mut stdout) {
        Ok(()) => 0,
        Err(error) => {
            stdout.write_all(format!("{}", error).as_bytes()).unwrap();
            1
        }
    };
    std::process::exit(exitcode);
}

fn main() {
    wrap_main(run);
}

fn run(args: &mut dyn Iterator<Item = String>, stdout: &mut dyn Write) -> R<()> {
    let entry = PathBuf::from(args.nth(1).unwrap_or_else(|| ".".to_string()));
    if !entry.exists() {
        return Err(format!("path not found: {}\n", entry.to_string_lossy()).into());
    }
    if entry.is_file() {
        let contents = fs::read(entry)?;
        stdout.write_all(&contents)?;
    } else if entry.is_dir() {
        let mut children = entry.read_dir()?.collect::<Result<Vec<_>, _>>()?;
        children.sort_unstable_by(|a, b| a.path().file_name().cmp(&b.path().file_name()));
        let stats = get_stats(&children)?;
        stdout.write_all(format!("{}\n", stats).as_bytes())?;
        stdout.write_all(format!("{}\n", "---".yellow().bold()).as_bytes())?;
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
    } else {
        return Err(format!("unknown filetype for: {}", entry.to_string_lossy()).into());
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
    use itertools::Itertools;
    use std::io::Cursor;
    use std::path::Path;
    use strip_ansi_escapes::strip;
    use tempdir::TempDir;

    struct Setup {
        stdout: Cursor<Vec<u8>>,
        tempdir: TempDir,
        outer_directory: PathBuf,
    }

    fn setup() -> R<Setup> {
        let outer_directory = std::env::current_dir()?;
        let tempdir = TempDir::new("si-test")?;
        std::env::set_current_dir(tempdir.path())?;
        Ok(Setup {
            stdout: Cursor::new(vec![]),
            tempdir,
            outer_directory,
        })
    }

    impl Setup {
        fn run(&mut self, args: Vec<String>) -> R<()> {
            let args = vec![vec!["si".to_string()], args].concat();
            run(&mut args.into_iter(), &mut self.stdout)
        }

        fn stdout(&self) -> String {
            String::from_utf8_lossy(self.stdout.get_ref()).into_owned()
        }

        fn tempdir(&self) -> &Path {
            self.tempdir.path()
        }
    }

    impl Drop for Setup {
        fn drop(&mut self) {
            std::env::set_current_dir(&self.outer_directory).unwrap();
        }
    }

    fn drop_stats(output: String) -> String {
        output.split("\n").skip(2).join("\n")
    }

    #[test]
    fn cats_files() -> R<()> {
        let mut setup = setup()?;
        fs::write(setup.tempdir().join("foo"), "bar")?;
        setup.run(vec![setup
            .tempdir()
            .join("foo")
            .to_string_lossy()
            .into_owned()])?;
        assert_eq!(setup.stdout(), "bar");
        Ok(())
    }

    #[test]
    fn path_not_found() -> R<()> {
        let mut setup = setup()?;
        let result = setup.run(vec!["does_not_exist.txt".to_string()]);
        assert_eq!(
            result.map_err(|x| x.to_string()),
            Err("path not found: does_not_exist.txt\n".to_string())
        );
        Ok(())
    }

    mod directories {
        use super::*;

        #[test]
        fn single_file() -> R<()> {
            let mut setup = setup()?;
            fs::write(setup.tempdir().join("foo"), "")?;
            setup.run(vec![setup.tempdir().to_string_lossy().into_owned()])?;
            assert_eq!(drop_stats(setup.stdout()), "foo\n");
            Ok(())
        }

        #[test]
        fn multiple_files_sorted() -> R<()> {
            let mut setup = setup()?;
            fs::write(setup.tempdir().join("foo"), "")?;
            fs::write(setup.tempdir().join("bar"), "")?;
            setup.run(vec![setup.tempdir().to_string_lossy().into_owned()])?;
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
            setup.run(vec![setup.tempdir().to_string_lossy().into_owned()])?;
            assert_eq!(strip(drop_stats(setup.stdout()))?, b"foo/\n");
            Ok(())
        }

        #[test]
        fn lists_directories_in_blue() -> R<()> {
            let mut setup = setup()?;
            fs::create_dir(setup.tempdir().join("foo"))?;
            setup.run(vec![setup.tempdir().to_string_lossy().into_owned()])?;
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
            setup.run(vec![setup.tempdir().to_string_lossy().into_owned()])?;
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
}
