use std::fs;
use std::io::Write;
use std::path::PathBuf;

fn main() -> Result<(), std::io::Error> {
    let exitcode = my_main(std::env::args(), &mut std::io::stderr())?;
    std::process::exit(exitcode);
}

type R<A> = Result<A, Box<dyn std::error::Error>>;

fn my_main(
    args: impl Iterator<Item = String>,
    stderr: &mut dyn Write,
) -> Result<i32, std::io::Error> {
    match run(args, stderr) {
        Ok(()) => Ok(0),
        Err(error) => {
            stderr.write_all(format!("{}", error).as_bytes())?;
            Ok(1)
        }
    }
}

fn run(mut args: impl Iterator<Item = String>, stderr: &mut dyn Write) -> R<()> {
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
        let exitcode = my_main(
            vec![
                "si".to_string(),
                tempdir.path().join("foo").to_string_lossy().into_owned(),
            ]
            .into_iter(),
            &mut stderr,
        )?;
        assert_eq!(exitcode, 0);
        assert_eq!(String::from_utf8_lossy(stderr.get_ref()), "bar");
        Ok(())
    }

    #[test]
    fn path_not_found() -> R<()> {
        let mut stderr = Cursor::new(vec![]);
        let exitcode = my_main(
            vec!["si".to_string(), "does_not_exist.txt".to_string()].into_iter(),
            &mut stderr,
        )?;
        assert_eq!(exitcode, 1);
        assert_eq!(
            String::from_utf8_lossy(stderr.get_ref()),
            "path not found: does_not_exist.txt\n"
        );
        Ok(())
    }
}
