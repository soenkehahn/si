#![cfg(test)]

use crate::test::setup;
use crate::R;
use std::fs;

#[test]
fn generate_readme() -> R<()> {
    let stdout = {
        let mut s = setup()?;
        s.run(vec![".".to_string()])?;
        s.stdout()
    };

    let readme = format!(
        "
`si` is a command line tool that shows information about a given path. It's
meant to replace the use of `ls`, `cat` and `tree` when manually exploring
files.

Example:
``` bash
{}
```
",
        stdout
    );
    let readme = format!("{}\n", readme.trim());
    let old_readme = String::from_utf8(fs::read("README.md")?)?;
    if readme != old_readme {
        fs::write("README.md", readme)?;
    }
    Ok(())
}
