mod stats;
mod tree;

use crate::{write_separator, Context, R};
use colored::*;
use std::fs;
use std::path::PathBuf;

pub fn output(context: &mut Context, directory: PathBuf) -> R<()> {
    let children = read_directory(directory)?;
    stats::output(context, &children)?;
    write_separator(context)?;
    output_file_listing(context, &children)?;
    write_separator(context)?;
    tree::output(context, children)?;
    Ok(())
}

fn output_file_listing(context: &mut Context, children: &[fs::DirEntry]) -> R<()> {
    for child in children {
        let path = format_dir_entry(child)?;
        let list_entry = if child.path().is_dir() {
            format!("{}/", path.blue().bold())
        } else {
            path
        };
        context
            .stdout
            .write_all(format!("{}\n", list_entry).as_bytes())?;
    }
    Ok(())
}

fn format_dir_entry(dir_entry: &fs::DirEntry) -> R<String> {
    Ok(dir_entry
        .path()
        .file_name()
        .ok_or_else(|| format!("directory entry has no last component: {:?}", dir_entry))?
        .to_string_lossy()
        .into_owned())
}

fn read_directory(directory: PathBuf) -> R<Vec<fs::DirEntry>> {
    let mut children = directory.read_dir()?.collect::<Result<Vec<_>, _>>()?;
    children.sort_unstable_by(|a, b| a.path().file_name().cmp(&b.path().file_name()));
    Ok(children)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::*;
    use std::fs;
    use strip_ansi_escapes::strip;

    #[test]
    fn single_file() -> R<()> {
        let mut setup = setup()?;
        fs::write(setup.tempdir().join("foo"), "")?;
        setup.run(vec!["."])?;
        assert_eq!(setup.get_section(1), "foo\n");
        Ok(())
    }

    #[test]
    fn multiple_files_sorted() -> R<()> {
        let mut setup = setup()?;
        fs::write(setup.tempdir().join("foo"), "")?;
        fs::write(setup.tempdir().join("bar"), "")?;
        setup.run(vec!["."])?;
        assert_eq!(setup.get_section(1), "bar\nfoo\n");
        Ok(())
    }

    #[test]
    fn lists_working_directory_when_no_argument_given() -> R<()> {
        let mut setup = setup()?;
        fs::write(setup.tempdir().join("foo"), "")?;
        fs::write(setup.tempdir().join("bar"), "")?;
        setup.run::<String>(vec![])?;
        assert_eq!(setup.get_section(1), "bar\nfoo\n");
        Ok(())
    }

    #[test]
    fn lists_directories_with_a_trailing_slash() -> R<()> {
        let mut setup = setup()?;
        fs::create_dir(setup.tempdir().join("foo"))?;
        setup.run(vec!["."])?;
        assert_eq!(strip(setup.get_section(1))?, b"foo/\n");
        Ok(())
    }

    #[test]
    fn lists_directories_in_blue() -> R<()> {
        let mut setup = setup()?;
        fs::create_dir(setup.tempdir().join("foo"))?;
        setup.run(vec!["."])?;
        assert_eq!(setup.get_section(1), format!("{}/\n", "foo".blue().bold()));
        Ok(())
    }
}
