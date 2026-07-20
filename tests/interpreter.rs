use lox::interpreter::Interpreter;
use mofu::walk_dir::walk_dir;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
struct TestCase {
    input: PathBuf,
    output: Option<PathBuf>,
}

#[test]
fn test_interpreter() -> io::Result<()> {
    let files: HashSet<PathBuf> = walk_dir("tests/fixtures", 0)?
        .filter_map(|entry| {
            if entry.metadata().is_file() {
                let path = PathBuf::from(entry);
                Some(path)
            } else {
                None
            }
        })
        .collect();

    let cases = files.iter().filter_map(|file| {
        if !file.extension().is_some_and(|extension| extension == "lox") {
            return None;
        }
        let input = file.clone();
        let output = file.with_extension("out");
        let output = if files.contains(&output) {
            Some(output)
        } else {
            None
        };
        let case = TestCase { input, output };
        Some(case)
    });

    for case in cases {
        let input = fs::read_to_string(&case.input)?;
        let mut output = Vec::new();
        let mut interpreter = Interpreter::new(&mut output);
        interpreter.interpret(&input).unwrap();
        let output = str::from_utf8(&output).unwrap();
        let expected = case
            .output
            .as_ref()
            .map(fs::read_to_string)
            .transpose()?
            .unwrap_or_else(String::new);
        if output != expected {
            let file = case.input.display();
            panic!(
                "\
==================== Test Case ====================
file: {file}

-------------------- Input ------------------------
{input}

-------------------- Output -----------------------
{output}

-------------------- Expected ---------------------
{expected}

====================================================
"
            );
        }
    }

    Ok(())
}
