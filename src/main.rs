mod directory;
mod file;

use colored::*;
use pager::Pager;
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
        file::output(stdout, entry)?;
    } else if entry.is_dir() {
        directory::output(stdout, entry)?;
    } else {
        return Err(format!("unknown filetype for: {}", entry.to_string_lossy()).into());
    }
    Ok(())
}

fn write_separator(stdout: &mut dyn Write) -> R<()> {
    stdout.write_all(format!("{}\n", "---".yellow().bold()).as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use itertools::Itertools;
    use std::fs;
    use std::io::Cursor;
    use std::path::Path;
    use tempdir::TempDir;

    pub struct Setup {
        stdout: Cursor<Vec<u8>>,
        tempdir: TempDir,
        outer_directory: PathBuf,
    }

    pub fn setup() -> R<Setup> {
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
        pub fn run(&mut self, args: Vec<String>) -> R<()> {
            let args = vec![vec!["si".to_string()], args].concat();
            run(&mut args.into_iter(), &mut self.stdout)
        }

        pub fn stdout(&self) -> String {
            String::from_utf8_lossy(self.stdout.get_ref()).into_owned()
        }

        pub fn tempdir(&self) -> &Path {
            self.tempdir.path()
        }
    }

    impl Drop for Setup {
        fn drop(&mut self) {
            std::env::set_current_dir(&self.outer_directory).unwrap();
        }
    }

    pub fn drop_stats(output: String) -> String {
        output.split("\n").skip(2).join("\n")
    }

    pub fn get_line(output: String, line: usize) -> String {
        output
            .split("\n")
            .nth(line)
            .expect(&format!("get_line: no {}th line in:\n{}", line, output))
            .to_string()
    }

    #[test]
    fn cats_files() -> R<()> {
        let mut setup = setup()?;
        fs::write(setup.tempdir().join("foo"), "bar")?;
        setup.run(vec!["foo".to_string()])?;
        assert_eq!(drop_stats(setup.stdout()), "bar");
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
}
