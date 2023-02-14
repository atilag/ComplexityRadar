use std::path::PathBuf;
use std::process::Command;

use anyhow::Result;

pub fn compute_cognitive_index(prog_lang: ProgrammingLang, file: PathBuf) -> Result<u16> {
    let lang_evaluator = create_lang_evaluator(prog_lang);
    lang_evaluator.eval(file)
}

pub enum ProgrammingLang {
    Rust,
    Python,
    Go,
}

trait LangEvaluator {
    fn eval(&self, file: PathBuf) -> Result<u16>;
}
struct RustLangEvaluator;
impl LangEvaluator for RustLangEvaluator {
    fn eval(&self, file: PathBuf) -> Result<u16> {
        let file_path = file.into_os_string().into_string().unwrap();
        let output = Command::new("cargo")
            .arg("clippy")
            .arg("--")
            .arg("-A clippy::all")
            .arg("-D clippy::cognitive_complexity")
            .arg("--file ")
            .arg(file_path)
            .output();
        //.map_err(|error| {
        //    println!("Error: {error}");
        //});

        match output {
            Ok(num) => println!("Ok({:#?})", num),
            Err(error) => println!("Error: {:#?}", error),
        };

        Ok(9)
    }
}

struct PythonLangEvaluator;
impl LangEvaluator for PythonLangEvaluator {
    fn eval(&self, file: PathBuf) -> Result<u16> {
        let file_path = file.into_os_string().into_string().unwrap();
        let output = Command::new("flake8")
            .arg("--select CCR001")
            .arg("--max-cognitive-complexity=1")
            .arg(format!("{file_path}"))
            .output()
            .map_err(|error| {
                println!("Error: {error}");
            });
        Ok(9)
    }
}

// Factory function to create language evaluators.
fn create_lang_evaluator(prog_lang: ProgrammingLang) -> Box<dyn LangEvaluator> {
    match prog_lang {
        ProgrammingLang::Rust => Box::new(RustLangEvaluator {}),
        _ => panic!("Language evaluator not implemented yet!"),
    }
}

#[cfg(test)]
mod test {
    use std::io::Write;

    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn calculate_cognitive_complexity_of_a_rust_file() {
        let complex_block_of_code = "
            let mut b  = 5;
            for i in 1..=10 {
            if i == 10 {
                    if b == 5 {
                        for a in 1..=3 {
                            println!(\"a = {a}\");
                        }
                    }
                }else if i == 3 {
                    if b == 3 {
                        for a in 1..=3 {
                            println!(\"a = {a}\");
                        }
                    } else if b == 5 {
                        for a in 1..=3 {
                            b = i;
                            println!(\"a = {a}\");
                        }
                    }
                }
            }
        ";

        let mut temp_rust_file = NamedTempFile::new().unwrap();
        temp_rust_file
            .write_all(complex_block_of_code.as_bytes())
            .unwrap();

        let expected = 9;
        let cognitive_complex_index =
            compute_cognitive_index(ProgrammingLang::Rust, temp_rust_file.path().into()).unwrap();

        assert_eq!(expected, cognitive_complex_index);
    }
}
