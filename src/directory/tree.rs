use crate::directory::{format_dir_entry, read_directory};
use crate::R;
use source::Source;
use std::fs;
use std::io::Write;

pub fn output(stdout: &mut dyn Write, children: Vec<fs::DirEntry>) -> R<()> {
    output_children(stdout, children, vec![])
}

fn output_children(
    stdout: &mut dyn Write,
    children: Vec<fs::DirEntry>,
    parent_prefix: Vec<bool>,
) -> R<()> {
    let mut source = Source::from(children.into_iter());
    while let Some(child) = source.next() {
        if !child.file_name().to_string_lossy().starts_with('.') {
            let child_prefix = {
                let mut clone = parent_prefix.clone();
                clone.push(source.has_next());
                clone
            };
            writeln!(
                stdout,
                "{}{}",
                render_prefix(child_prefix.clone()),
                format_dir_entry(&child)?
            )?;
            if child.path().is_dir() {
                let grand_children = read_directory(child.path())?;
                output_children(stdout, grand_children, child_prefix)?;
            }
        }
    }
    Ok(())
}

fn render_prefix(prefix: Vec<bool>) -> String {
    let mut result = "".to_string();
    let mut source = Source::from(prefix.into_iter().skip(1));
    while let Some(level_has_next) = source.next() {
        let is_leaf = !source.has_next();
        let snippet = match (is_leaf, level_has_next) {
            (true, true) => "├── ",
            (true, false) => "└── ",
            (false, true) => "│   ",
            (false, false) => "    ",
        };
        result.push_str(snippet);
    }
    result
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::*;

    fn dedent(string: &str) -> String {
        textwrap::dedent(string)
            .chars()
            .skip_while(|x| x == &'\n')
            .collect()
    }

    #[test]
    fn shows_a_directory_tree() -> R<()> {
        let mut setup = setup()?;
        fs::write(setup.tempdir().join("foo"), "")?;
        fs::create_dir("bar")?;
        fs::write(setup.tempdir().join("bar/baz"), "")?;
        setup.run(vec![])?;
        assert_eq!(
            setup.get_section(2),
            dedent(
                "
                    bar
                    └── baz
                    foo
                "
            )
        );
        Ok(())
    }

    #[test]
    fn works_for_deeper_trees() -> R<()> {
        let mut setup = setup()?;
        fs::create_dir_all("foo/bar")?;
        fs::write(setup.tempdir().join("foo/bar/baz"), "")?;
        setup.run(vec![])?;
        assert_eq!(
            setup.get_section(2),
            dedent(
                "
                    foo
                    └── bar
                        └── baz
                "
            )
        );
        Ok(())
    }

    #[test]
    fn sorts_children_alphabetically() -> R<()> {
        let mut setup = setup()?;
        fs::create_dir("dir")?;
        fs::write(setup.tempdir().join("dir/foo"), "")?;
        fs::write(setup.tempdir().join("dir/bar"), "")?;
        fs::write(setup.tempdir().join("dir/baz"), "")?;
        setup.run(vec![])?;
        assert_eq!(
            setup.get_section(2),
            dedent(
                "
                    dir
                    ├── bar
                    ├── baz
                    └── foo
                "
            )
        );
        Ok(())
    }

    #[test]
    fn renders_file_prefixes_differently_when_there_are_more_files() -> R<()> {
        let mut setup = setup()?;
        fs::create_dir("foo")?;
        fs::write(setup.tempdir().join("foo/bar"), "")?;
        fs::write(setup.tempdir().join("foo/baz"), "")?;
        setup.run(vec![])?;
        assert_eq!(
            setup.get_section(2),
            dedent(
                "
                    foo
                    ├── bar
                    └── baz
                "
            )
        );
        Ok(())
    }

    #[test]
    fn renders_prefix_lines_for_grandchildren_correctly() -> R<()> {
        let mut setup = setup()?;
        fs::create_dir_all("foo/bar")?;
        fs::create_dir_all("foo/baz")?;
        fs::write(setup.tempdir().join("foo/bar/file"), "")?;
        fs::write(setup.tempdir().join("foo/baz/file"), "")?;
        setup.run(vec![])?;
        assert_eq!(
            setup.get_section(2),
            dedent(
                "
                    foo
                    ├── bar
                    │   └── file
                    └── baz
                        └── file
                "
            )
        );
        Ok(())
    }

    #[test]
    fn renders_prefix_lines_for_grandchildren_correctly_for_deeper_trees() -> R<()> {
        let mut setup = setup()?;
        fs::create_dir_all("a/b/c")?;
        fs::write(setup.tempdir().join("a/b/c/d"), "")?;
        fs::write(setup.tempdir().join("a/e"), "")?;
        setup.run(vec![])?;
        assert_eq!(
            setup.get_section(2),
            dedent(
                "
                    a
                    ├── b
                    │   └── c
                    │       └── d
                    └── e
                "
            )
        );
        Ok(())
    }

    #[test]
    fn does_not_show_hidden_files() -> R<()> {
        let mut setup = setup()?;
        fs::write(setup.tempdir().join(".foo"), "")?;
        fs::create_dir_all("bar")?;
        fs::write(setup.tempdir().join("bar/baz"), "")?;
        fs::write(setup.tempdir().join("bar/.baz"), "")?;
        setup.run(vec![])?;
        assert_eq!(
            setup.get_section(2),
            dedent(
                "
                    bar
                    └── baz
                "
            )
        );
        Ok(())
    }
}
