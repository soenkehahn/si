#![cfg(test)]

use crate::R;
use indoc::indoc;
use std::fs;

#[test]
fn generate_readme() -> R<()> {
    let readme = indoc!(
        "
            `si` is a command line tool that shows information about a given path. It's
            meant to replace the use of `ls`, `cat` and `tree` when manually exploring
            files.
        "
    );
    let old_readme = String::from_utf8(fs::read("README.md")?)?;
    if readme != old_readme {
        fs::write("README.md", readme)?;
    }
    Ok(())
}
