use regex::Regex;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Result;

pub fn compute_cognitive_index(
    prog_lang: ProgrammingLang,
    file: PathBuf,
) -> Result<Vec<FunctionComplexity>> {
    let lang_evaluator = create_lang_evaluator(prog_lang);
    lang_evaluator.eval(file)
}

pub enum ProgrammingLang {
    Rust,
    Python,
    Go,
}

#[derive(PartialEq, Eq, Debug)]
pub struct FunctionComplexity {
    function: String,
    cognitive_complexity_idx: u16,
}

trait LangEvaluator {
    fn eval(&self, file: PathBuf) -> Result<Vec<FunctionComplexity>>;
}
struct RustLangEvaluator;
impl LangEvaluator for RustLangEvaluator {
    fn eval(&self, file: PathBuf) -> Result<Vec<FunctionComplexity>> {
        let file_path = file.into_os_string().into_string().unwrap();
        let output = Command::new("cargo")
            .arg("clippy")
            .arg("--")
            .arg("-A")
            .arg("clippy::all")
            .arg("-D")
            .arg("clippy::cognitive_complexity")
            .output()
            .map_err(|error| {
                println!("Error running cargo clippy: {error}");
            });

        let stdout = match output {
            Ok(output) => String::from_utf8(output.stderr)
                .expect("Unintiligible output from clippy command")
                .to_owned(),
            Err(_) => "".to_string(),
        };

        get_function_complexities_from_clippy(stdout)
    }
}

fn get_function_complexities_from_clippy(text: String) -> Result<Vec<FunctionComplexity>> {
    let regex_pattern = r#"the function has a cognitive complexity of \((\d+)/\d+\)\n\s+-->\s+(\S+):(\d+):\d+\n\s+\|\n(\d+.+)\n"#;
    let regex = Regex::new(regex_pattern).unwrap();

    // Extract the matched strings
    regex.captures(&text).unwrap();

    if let Some(captures) = regex.captures(&text) {
        let complexity = captures.get(1).unwrap().as_str();
        let file_path = captures.get(2).unwrap().as_str();
        let line_number = captures.get(3).unwrap().as_str();
        let function_name = captures.get(4).unwrap().as_str();
        println!("Complexity: {}", complexity);
        println!("File path: {}", file_path);
        println!("Line number: {}", line_number);
        println!("function name: {}", function_name);
    }

    Ok(vec![])
}

struct PythonLangEvaluator;
impl LangEvaluator for PythonLangEvaluator {
    fn eval(&self, file: PathBuf) -> Result<Vec<FunctionComplexity>> {
        let file_path = file.into_os_string().into_string().unwrap();
        let output = Command::new("flake8")
            .arg("--select CCR001")
            .arg("--max-cognitive-complexity=1")
            .arg(format!("{file_path}"))
            .output()
            .map_err(|error| {
                println!("Error: {error}");
            });

        let stdout = match output {
            Ok(output) => String::from_utf8(output.stderr)
                .expect("Unintiligible output from flake8 command")
                .to_owned(),
            Err(_) => "".to_string(),
        };

        get_function_complexities_from_flake8(stdout)
    }
}

fn get_function_complexities_from_flake8(text: String) -> Result<Vec<FunctionComplexity>> {
    Ok(vec![])
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
            fn ugly_function(){
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
            }
        ";

        let mut temp_rust_file = NamedTempFile::new().unwrap();
        temp_rust_file
            .write_all(complex_block_of_code.as_bytes())
            .unwrap();

        let expected = vec![FunctionComplexity {
            function: "ugly_function".to_string(),
            cognitive_complexity_idx: 9,
        }];

        let cognitive_complex_index =
            compute_cognitive_index(ProgrammingLang::Rust, temp_rust_file.path().into()).unwrap();

        assert_eq!(expected, cognitive_complex_index);
    }
}
