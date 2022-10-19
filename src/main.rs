mod directory;
mod file;
mod utils;

use colored::*;
use lexiclean::Lexiclean;
use pager::Pager;
use source::Source;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use utils::render_path;

type R<A> = Result<A, Box<dyn std::error::Error>>;

pub struct Context<'a> {
    args: Vec<String>,
    stdout: &'a mut dyn Write,
    terminal_width: Option<usize>,
}

fn wrap_main(action: fn(context: &mut Context) -> R<()>) {
    let terminal_width = term_size::dimensions_stdout().map(|(width, _)| width);
    colored::control::set_override(true);
    Pager::with_pager("less -RFX").setup();
    let mut stdout = std::io::stdout();
    let exitcode = match action(&mut Context {
        args: std::env::args().collect(),
        stdout: &mut stdout,
        terminal_width,
    }) {
        Ok(()) => 0,
        Err(error) => {
            std::io::stderr()
                .write_all(format!("{}", error).as_bytes())
                .unwrap();
            1
        }
    };
    std::process::exit(exitcode);
}

fn main() {
    wrap_main(run);
}

fn run(context: &mut Context) -> R<()> {
    let entry = PathBuf::from(
        context
            .args
            .get(1)
            .cloned()
            .unwrap_or_else(|| ".".to_string()),
    );
    show_information(context, entry)?;
    Ok(())
}

fn show_information(context: &mut Context, entry: PathBuf) -> R<()> {
    if !entry.exists() {
        return Err(format!("path not found: {}\n", render_path(entry)).into());
    }
    if entry.is_symlink() {
        let link = fs::read_link(&entry)?;
        let destination = match entry.parent() {
            Some(parent) => parent.join(&link).lexiclean(),
            None => return Err(format!("symlink {} has no parent", render_path(entry)).into()),
        };
        writeln!(
            context.stdout,
            "{} is a symbolic link pointing to {}\nresolving to:",
            render_path(&entry),
            link.to_string_lossy(),
        )?;
        write_separator(context)?;
        show_information(context, destination)?;
    } else if entry.is_file() {
        file::output(context, entry)?;
    } else if entry.is_dir() {
        directory::output(context, entry)?;
    } else {
        return Err(format!("unknown filetype for: {}", render_path(entry)).into());
    }
    Ok(())
}

fn separator(terminal_width: Option<usize>) -> String {
    format!(
        "{}\n",
        Source::replicate(terminal_width.unwrap_or(20) as u32, "─")
            .join("")
            .yellow()
            .bold()
    )
}

fn write_separator(context: &mut Context) -> R<()> {
    context
        .stdout
        .write_all(separator(context.terminal_width).as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
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

    pub const TEST_TERMINAL_WIDTH: Option<usize> = Some(50);

    impl Setup {
        pub fn run<S: Into<String>>(&mut self, args: Vec<S>) -> R<()> {
            let context = &mut Context {
                args: vec![
                    vec!["si".to_string()],
                    args.into_iter().map(|x| x.into()).collect(),
                ]
                .concat(),
                stdout: &mut self.stdout,
                terminal_width: TEST_TERMINAL_WIDTH,
            };
            run(context)?;
            eprintln!("stdout:\n{}", self.stdout());
            Ok(())
        }

        pub fn tempdir(&self) -> &Path {
            self.tempdir.path()
        }

        pub fn stdout(&self) -> String {
            String::from_utf8_lossy(self.stdout.get_ref()).into_owned()
        }

        pub fn get_section(&self, n: usize) -> String {
            self.stdout()
                .split(&separator(TEST_TERMINAL_WIDTH))
                .nth(n)
                .expect("not enough sections")
                .to_string()
        }
    }

    impl Drop for Setup {
        fn drop(&mut self) {
            std::env::set_current_dir(&self.outer_directory).unwrap();
        }
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
        setup.run(vec!["foo"])?;
        assert!(setup.get_section(1).ends_with("bar"));
        Ok(())
    }

    #[test]
    fn path_not_found() -> R<()> {
        let mut setup = setup()?;
        let result = setup.run(vec!["does_not_exist.txt"]);
        assert_eq!(
            result.map_err(|x| x.to_string()),
            Err("path not found: ./does_not_exist.txt\n".to_string())
        );
        Ok(())
    }

    #[test]
    fn separators_span_the_terminal_width() -> R<()> {
        let mut setup = setup()?;
        setup.run(vec!["."])?;
        let expected = Source::replicate(TEST_TERMINAL_WIDTH.unwrap() as u32, "─")
            .join("")
            .yellow()
            .bold()
            .to_string();
        assert_eq!(setup.stdout().lines().collect::<Vec<&str>>()[1], expected);
        Ok(())
    }

    #[test]
    fn resolves_symlinks() -> R<()> {
        let mut setup = setup()?;
        fs::write(setup.tempdir().join("foo"), "foo")?;
        std::os::unix::fs::symlink("foo", "bar")?;
        setup.run(vec!["bar"])?;
        assert_eq!(
            setup.get_section(0),
            "./bar is a symbolic link pointing to foo\nresolving to:\n"
        );
        assert_eq!(setup.get_section(1), "file: ./foo, 3 bytes\n");
        Ok(())
    }

    #[test]
    fn resolves_symlinks_in_other_directories() -> R<()> {
        let mut setup = setup()?;
        fs::create_dir(setup.tempdir().join("dir"))?;
        fs::write(setup.tempdir().join("dir/foo"), "foo")?;
        std::os::unix::fs::symlink("foo", "dir/bar")?;
        setup.run(vec!["dir/bar"])?;
        assert_eq!(
            setup.get_section(0),
            "./dir/bar is a symbolic link pointing to foo\nresolving to:\n"
        );
        assert_eq!(setup.get_section(1), "file: ./dir/foo, 3 bytes\n");
        Ok(())
    }

    #[test]
    fn resolved_symlinks_are_printed_normalized() -> R<()> {
        let mut setup = setup()?;
        fs::create_dir(setup.tempdir().join("dir"))?;
        fs::write(setup.tempdir().join("foo"), "foo")?;
        std::os::unix::fs::symlink("../foo", "dir/bar")?;
        setup.run(vec!["dir/bar"])?;
        assert_eq!(
            setup.get_section(0),
            "./dir/bar is a symbolic link pointing to ../foo\nresolving to:\n"
        );
        assert_eq!(setup.get_section(1), "file: ./foo, 3 bytes\n");
        Ok(())
    }

    #[test]
    fn render_path_works() -> R<()> {
        assert_eq!(render_path(PathBuf::from("foo")), "./foo");
        assert_eq!(render_path(PathBuf::from("/foo")), "/foo");
        assert_eq!(render_path(PathBuf::from("./foo")), "./foo");
        Ok(())
    }
}
