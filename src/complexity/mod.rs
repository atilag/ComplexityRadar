use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::Command;
use std::{fs, vec};
use syn::{
    self, Block, Expr, ExprBlock, ExprClosure, ExprForLoop, ExprIf, ExprLet, ExprMatch,
    ExprMethodCall, ExprWhile, Item, ItemFn, Stmt,
};

#[derive(PartialEq, Eq, Debug)]
pub struct FunctionComplexity {
    pub function: String,
    pub cognitive_complexity_value: u16,
}

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

const NESTING_LEVEL_ZERO: u16 = 0;

trait LangEvaluator {
    fn eval(&self, file: PathBuf) -> Result<Vec<FunctionComplexity>>;
}
struct RustLangEvaluator;
impl LangEvaluator for RustLangEvaluator {
    fn eval(&self, file: PathBuf) -> Result<Vec<FunctionComplexity>> {
        if let Some(extension) = file.extension() {
            if extension != "rs" {
                return Err(anyhow!("Invalid source file"));
            }
        }

        let code = fs::read_to_string(&file)
            .map_err(|e| {
                format!(
                    "Cannot open code file: {}: Make sure you have cloned the repository locally. Error: {}",
                    file.to_string_lossy().to_string(),
                    e.to_string()
                )
            })
            .unwrap();
        let syntax_tree = syn::parse_file(&code)?;
        let functions_complexity = calc_complexities_by_function(syntax_tree);

        functions_complexity
    }
}

fn calc_complexities_by_function(syntax_tree: syn::File) -> Result<Vec<FunctionComplexity>> {
    Ok(syntax_tree
        .items
        .iter()
        .filter_map(|item| {
            if let Item::Fn(item_fn) = item {
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
    cognitive_complexity_block(&func.block, NESTING_LEVEL_ZERO)
}

fn cognitive_complexity_block(block: &Block, nesting_level: u16) -> u16 {
    let Block { stmts, .. } = &*block;
    stmts
        .iter()
        .map(|stmt| match stmt {
            Stmt::Expr(expr) | Stmt::Semi(expr, ..) => {
                cognitive_complexity_expr(expr, nesting_level)
            }
            Stmt::Local(local) => match &local.init {
                Some((_, expr)) => cognitive_complexity_expr(&expr, nesting_level),
                None => 0,
            },
            _ => 0,
        })
        .sum()
}

fn cognitive_complexity_expr(expr: &Expr, nesting_level: u16) -> u16 {
    let expr_cognitive_index = match expr {
        Expr::Match(ExprMatch { arms, .. }) => {
            let arm_complexity: u16 = arms
                .iter()
                .map(|arm| cognitive_complexity_expr(&arm.body, nesting_level + 1))
                .sum();
            1 + arm_complexity
        }
        Expr::If(ExprIf {
            cond,
            then_branch,
            else_branch,
            ..
        }) => {
            let conditional_expr_complexity = cognitive_complexity_expr(cond, nesting_level + 1);
            let then_block_complexity = cognitive_complexity_block(then_branch, nesting_level + 1);
            let else_block_complexity = else_branch.as_ref().map_or(0, |else_expr| {
                let box_expr = &else_expr.1;
                cognitive_complexity_expr(box_expr, nesting_level + 1)
            });
            1 + conditional_expr_complexity + then_block_complexity + else_block_complexity
        }
        Expr::ForLoop(ExprForLoop { body, .. }) | Expr::While(ExprWhile { body, .. }) => {
            1 + cognitive_complexity_block(body, nesting_level + 1)
        }
        Expr::MethodCall(ExprMethodCall { receiver, args, .. }) => {
            let complex_index_sum: u16 = args
                .iter()
                .map(|argument| cognitive_complexity_expr(argument, nesting_level))
                .sum();
            complex_index_sum + cognitive_complexity_expr(&receiver, nesting_level)
        }
        Expr::Closure(ExprClosure { body, .. }) => {
            // The closure (lambda) itself doesn't add to the index, but increments nesting level
            cognitive_complexity_expr(body, nesting_level + 1)
        }
        Expr::Block(ExprBlock { block, .. }) => cognitive_complexity_block(block, nesting_level),
        Expr::Let(ExprLet { expr, .. }) => cognitive_complexity_expr(expr, nesting_level),
        _ => 0,
    };

    // Return 0 in this case because if there's no cognitive index in the expression we don't have to add
    // any nesting level
    if expr_cognitive_index == 0 {
        return 0;
    }

    expr_cognitive_index + nesting_level
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
    async fn calculate_cognitive_complexity_of_two_rust_functions_from_a_file() {
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

    #[tokio::test]
    async fn calculate_cognitive_complexity_of_for_loop_and_if_nesting_level_1() {
        let simple_block_of_code = "
            fn function() {
                for i in 1..=10 { // 1 + 0 nesting
                    if i == 10 { // 1 + 1 nesting
                        println!(\"i = {i}\");
                    }
                }
            } // Total: 3
        ";

        let mut temp_rust_file = NamedTempFile::new().unwrap();
        temp_rust_file
            .write_all(simple_block_of_code.as_bytes())
            .unwrap();

        let expected = vec![FunctionComplexity {
            function: "function".to_string(),
            cognitive_complexity_value: 3,
        }];

        let cognitive_complex_index =
            compute_cognitive_index(ProgrammingLang::Rust, temp_rust_file.path().into()).unwrap();

        assert_eq!(expected, cognitive_complex_index);
    }

    #[tokio::test]
    async fn calculate_cognitive_complexity_of_two_ifs_nesting_level_2() {
        let simple_block_of_code = "
            fn function() {
                for i in 1..=10 { // 1 + 0 nesting
                    if i == 10 { // 1 + 1 nesting
                        if n == 10 { // 1 + 2 nesting
                            return 0;
                        }
                        if n == 10 { // 1 + 2 nesting
                            return 0;
                        }
                        println!(\"i = {i}\");
                    }
                }
            } // Total: 9
        ";

        let mut temp_rust_file = NamedTempFile::new().unwrap();
        temp_rust_file
            .write_all(simple_block_of_code.as_bytes())
            .unwrap();

        let expected = vec![FunctionComplexity {
            function: "function".to_string(),
            cognitive_complexity_value: 9,
        }];

        let cognitive_complex_index =
            compute_cognitive_index(ProgrammingLang::Rust, temp_rust_file.path().into()).unwrap();

        assert_eq!(expected, cognitive_complex_index);
    }

    #[tokio::test]
    async fn calculate_cognitive_complexity_of_if_with_3_binary_operators() {
        let simple_block_of_code = "
            fn function() {
                let a = true;
                let b = false;
                let c = true;
                // Binary operations are not supported yet, so they don't add up.
                if a || b && b || a && c || b { // 1 + 0 nesting 
                        println!(\"Hola!\");
                }
            } // Total: 1
        ";

        let mut temp_rust_file = NamedTempFile::new().unwrap();
        temp_rust_file
            .write_all(simple_block_of_code.as_bytes())
            .unwrap();

        let expected = vec![FunctionComplexity {
            function: "function".to_string(),
            cognitive_complexity_value: 1,
        }];

        let cognitive_complex_index =
            compute_cognitive_index(ProgrammingLang::Rust, temp_rust_file.path().into()).unwrap();

        assert_eq!(expected, cognitive_complex_index);
    }

    #[tokio::test]
    async fn calculate_cognitive_complexity_of_closure_with_an_if() {
        let simple_block_of_code = "
            fn function() {
                let v = vec![1,2,3,4];
                let sum : i32 = v.into_iter().map(|element|{
                    if element == 1 { // 1 + 2 nesting
                        return element * 2;
                    }
                    element + 1
                })
                .sum();
            } // Total: 3
        ";

        let mut temp_rust_file = NamedTempFile::new().unwrap();
        temp_rust_file
            .write_all(simple_block_of_code.as_bytes())
            .unwrap();

        let expected = vec![FunctionComplexity {
            function: "function".to_string(),
            cognitive_complexity_value: 3,
        }];

        let cognitive_complex_index =
            compute_cognitive_index(ProgrammingLang::Rust, temp_rust_file.path().into()).unwrap();

        assert_eq!(expected, cognitive_complex_index);
    }

    #[tokio::test]
    async fn calculate_cognitive_complexity_of_closures_with_ifs() {
        let simple_block_of_code = "
            fn function() {
                let v = vec![1,2,3,4];
                let sum : i32 = v
                    .into_iter()
                    .map(|element|{
                        if element == 1 { // 1 + 2 nesting
                            return element * 2;
                        }
                        element + 1
                    })
                    .map(|element|{
                        if element == 2 { // 1 + 2 nesting
                            return element * 2;
                        }
                        element + 1
                    })
                    .map(|element| element + 1)
                    .map(|element|{
                        if element > 3 { // 1 + 2 nesting
                            return element * 2
                        }
                        element + 1
                    })
                    .sum();
                println!(\"sum = {sum}\");
            } // Total: 9
        ";

        let mut temp_rust_file = NamedTempFile::new().unwrap();
        temp_rust_file
            .write_all(simple_block_of_code.as_bytes())
            .unwrap();

        let expected = vec![FunctionComplexity {
            function: "function".to_string(),
            cognitive_complexity_value: 9,
        }];

        let cognitive_complex_index =
            compute_cognitive_index(ProgrammingLang::Rust, temp_rust_file.path().into()).unwrap();

        assert_eq!(expected, cognitive_complex_index);
    }
}
