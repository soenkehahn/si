use pager::Pager;
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
        let mut children: Vec<String> = entry
            .read_dir()?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|x| {
                x.path()
                    .file_name()
                    .unwrap_or_else(|| {
                        panic!(format!("directory entry has no last component: {:?}", x))
                    })
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        children.sort();
        for child in children {
            stdout.write_all(child.as_bytes())?;
            stdout.write_all(b"\n")?;
        }
    } else {
        return Err(format!("unknown filetype for: {}", entry.to_string_lossy()).into());
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;
    use std::path::Path;
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
            assert_eq!(setup.stdout(), "foo\n");
            Ok(())
        }

        #[test]
        fn multiple_files_sorted() -> R<()> {
            let mut setup = setup()?;
            fs::write(setup.tempdir().join("foo"), "")?;
            fs::write(setup.tempdir().join("bar"), "")?;
            setup.run(vec![setup.tempdir().to_string_lossy().into_owned()])?;
            assert_eq!(setup.stdout(), "bar\nfoo\n");
            Ok(())
        }

        #[test]
        fn lists_directory_when_no_argument_given() -> R<()> {
            let mut setup = setup()?;
            fs::write(setup.tempdir().join("foo"), "")?;
            fs::write(setup.tempdir().join("bar"), "")?;
            setup.run(vec![])?;
            assert_eq!(setup.stdout(), "bar\nfoo\n");
            Ok(())
        }
    }
}
