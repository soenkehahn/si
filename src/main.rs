use std::fs;
use std::io::Write;
use std::path::PathBuf;

type R<A> = Result<A, Box<dyn std::error::Error>>;

fn wrap_main(action: fn(args: &mut dyn Iterator<Item = String>, stderr: &mut dyn Write) -> R<()>) {
    let mut stderr = std::io::stderr();
    let exitcode = match action(&mut std::env::args(), &mut stderr) {
        Ok(()) => 0,
        Err(error) => {
            stderr.write_all(format!("{}", error).as_bytes()).unwrap();
            1
        }
    };
    std::process::exit(exitcode);
}

fn main() {
    wrap_main(run);
}

fn run(args: &mut dyn Iterator<Item = String>, stderr: &mut dyn Write) -> R<()> {
    let file = PathBuf::from(args.nth(1).unwrap());
    if !file.exists() {
        return Err(format!("path not found: {}\n", file.to_string_lossy()).into());
    }
    let contents = fs::read(file)?;
    stderr.write_all(&contents)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;
    use tempdir::TempDir;

    #[test]
    fn cats_files() -> R<()> {
        let tempdir = TempDir::new("si-test")?;
        fs::write(tempdir.path().join("foo"), "bar")?;
        let mut stderr = Cursor::new(vec![]);
        run(
            &mut vec![
                "si".to_string(),
                tempdir.path().join("foo").to_string_lossy().into_owned(),
            ]
            .into_iter(),
            &mut stderr,
        )?;
        assert_eq!(String::from_utf8_lossy(stderr.get_ref()), "bar");
        Ok(())
    }

    #[test]
    fn path_not_found() -> R<()> {
        let mut stderr = Cursor::new(vec![]);
        let result = run(
            &mut vec!["si".to_string(), "does_not_exist.txt".to_string()].into_iter(),
            &mut stderr,
        );
        assert_eq!(
            result.map_err(|x| x.to_string()),
            Err("path not found: does_not_exist.txt\n".to_string())
        );
        Ok(())
    }
}
