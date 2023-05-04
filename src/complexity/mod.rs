use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;
use std::{fs, vec};
use syn::{
    self, BinOp, Block, Expr, ExprForLoop, ExprIf, ExprMatch, ExprWhile, Item, ItemFn, Stmt,
};

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
    cognitive_complexity_block(&func.block)
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
    let mut binary_count = 0;
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
            let conditional_expr_complexity = cognitive_complexity_expr(cond) + 1;
            let then_block_complexity = cognitive_complexity_block(then_branch);
            let else_block_complexity = else_branch.as_ref().map_or(0, |else_expr| {
                let box_expr = &else_expr.1;
                cognitive_complexity_expr(box_expr) + 1
            });
            conditional_expr_complexity + then_block_complexity + else_block_complexity
        }
        Expr::ForLoop(ExprForLoop { expr, body, .. }) => cognitive_complexity_block(body) + 1,
        Expr::While(ExprWhile { cond, body, .. }) => cognitive_complexity_block(body) + 1,
        Expr::Binary(expr_binary) => {
            if let BinOp::And(_) | BinOp::Or(_) | BinOp::BitXor(_) = expr_binary.op {
                binary_count += 1;
            }
            binary_count += cognitive_complexity_expr(&expr_binary.left);
            binary_count += cognitive_complexity_expr(&expr_binary.right);
            binary_count
        }
        _ => 0,
    }
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
    // Yep, the initial idea is to use flake's cognitive complexity linter flag
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
            fn function() { // 38
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

        let expected = vec![
            FunctionComplexity {
                function: "function".to_string(),
                cognitive_complexity_value: 11,
            },
            FunctionComplexity {
                function: "function2".to_string(),
                cognitive_complexity_value: 11,
            },
        ];

        let cognitive_complex_index =
            compute_cognitive_index(ProgrammingLang::Rust, temp_rust_file.path().into()).unwrap();

        assert_eq!(expected, cognitive_complex_index);
    }
}
