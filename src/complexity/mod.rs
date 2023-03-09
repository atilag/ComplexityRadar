use crate::report::print_report;
use anyhow::Result;
use futures::executor::block_on;
use regex::Regex;
use std::f32::consts::E;
use std::path::PathBuf;
use std::process::Command;
use std::{fs, vec};
use syn::{self, Block, Expr, ExprForLoop, ExprIf, ExprMatch, ExprWhile, Item, ItemFn, Stmt};

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
    cognitive_complexity_value: u16,
}

trait LangEvaluator {
    fn eval(&self, file: PathBuf) -> Result<Vec<FunctionComplexity>>;
}
struct RustLangEvaluator;
impl LangEvaluator for RustLangEvaluator {
    fn eval(&self, file: PathBuf) -> Result<Vec<FunctionComplexity>> {
        let code = fs::read_to_string(&file)
            .map_err(|e| e.to_string())
            .unwrap();
        let syntax_tree = syn::parse_file(&code).map_err(|e| e.to_string()).unwrap();
        let functions_complexity = calc_complexities_by_function(syntax_tree).unwrap();

        Ok(functions_complexity)
    }
}

fn calc_complexities_by_function(syntax_tree: syn::File) -> Result<Vec<FunctionComplexity>> {
    Ok(syntax_tree
        .items
        .iter()
        .filter_map(|item| {
            if let syn::Item::Fn(item_fn) = item {
                Some(item_fn)
            } else {
                None
            }
        })
        .map(|func| {
            let cognitive_complexity_value = cognitive_complexity_func(&func);
            FunctionComplexity {
                function: get_function_name(&func),
                cognitive_complexity_value: cognitive_complexity_value,
            }
        })
        .collect::<Vec<FunctionComplexity>>())
}

fn get_function_name(item_fn: &ItemFn) -> String {
    item_fn.sig.ident.to_string()
}

fn cognitive_complexity_func(func: &ItemFn) -> u16 {
    1 + cognitive_complexity_block(&func.block)
}

fn cognitive_complexity_block(block: &Block) -> u16 {
    let Block { stmts, .. } = &*block;
    stmts
        .iter()
        .map(|stmt| match stmt {
            Stmt::Expr(expr) => cognitive_complexity_expr(expr),
            Stmt::Item(Item::Fn(fn_item)) => cognitive_complexity_func(fn_item),
            _ => 0,
        })
        .sum()
}

fn cognitive_complexity_expr(expr: &Expr) -> u16 {
    match expr {
        Expr::Match(ExprMatch { arms, .. }) => {
            let arm_complexity: u16 = arms
                .iter()
                .map(|arm| cognitive_complexity_expr(&arm.body))
                .sum();
            1 + arm_complexity
        }
        Expr::If(ExprIf {
            cond,
            then_branch,
            else_branch,
            ..
        }) => {
            let cond_expr_complexity = cognitive_complexity_expr(cond);
            let then_block_complexity = cognitive_complexity_block(then_branch);
            let else_block_complexity = else_branch.as_ref().map_or(0, |else_expr| {
                let box_expr = &else_expr.1;
                cognitive_complexity_expr(box_expr)
            });
            1 + cond_expr_complexity + then_block_complexity + else_block_complexity
        }
        Expr::ForLoop(ExprForLoop { expr, body, .. }) => {
            let expr_complexity = cognitive_complexity_expr(expr);
            let block_complexity = cognitive_complexity_block(body);
            1 + expr_complexity + block_complexity
        }
        Expr::While(ExprWhile { cond, body, .. }) => {
            let cond_expr_complexity = cognitive_complexity_expr(cond);
            let body_complexity = cognitive_complexity_block(body);
            1 + cond_expr_complexity + body_complexity
        }
        _ => 0,
    }
}

fn get_function_complexities_from_clippy(text: String) -> Result<Vec<FunctionComplexity>> {
    // This is the typical output of the clippy command:
    //
    //warning: the function has a cognitive complexity of (10/8)
    //--> src/main.rs:28:4
    //   |
    //28 | fn function() {

    let regex_pattern = r#"the function has a cognitive complexity of \((\d+)/\d+\)\n\s+-->\s+(\S+):(\d+):\d+\n\s+\|\n.+\|\s+([^\n{]+)"#;
    let regex = Regex::new(regex_pattern).unwrap();

    let func_cc_idxes: Vec<FunctionComplexity> = regex
        .captures_iter(&text)
        .map(|captures| {
            println!("Len: {}", captures.len());

            let complexity = captures.get(1).unwrap().as_str();
            let file_path = captures.get(2).unwrap().as_str();
            let line_number = captures.get(3).unwrap().as_str();
            let function_name = captures.get(4).unwrap().as_str();

            println!("complexity {complexity}");
            println!("file_path {file_path}");
            println!("line_number {line_number}");
            println!("function_name {function_name}");

            FunctionComplexity {
                function: function_name.to_string(),
                cognitive_complexity_value: complexity.parse::<u16>().unwrap_or(0),
            }
        })
        .collect();

    Ok(func_cc_idxes)
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
            fn function() {
                let mut b = 5;
                for i in 1..=10 {
                    if i == 10 {
                        if b == 5 {
                            for a in 1..=3 {
                                println!(
                                    \"a = {a}
                                
                                \"
                                );
                            }
                        }
                    } else if i == 3 {
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

            fn function2() {
                let mut b = 5;
                for i in 1..=10 {
                    if i == 10 {
                        if b == 5 {
                            for a in 1..=3 {
                                println!(\"a = {a}\");
                            }
                        }
                    } else if i == 3 {
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
            function: "fn function()".to_string(),
            cognitive_complexity_value: 9,
        }];

        let cognitive_complex_index =
            compute_cognitive_index(ProgrammingLang::Rust, temp_rust_file.path().into()).unwrap();

        assert_eq!(expected, cognitive_complex_index);
    }
}
