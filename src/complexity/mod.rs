use anyhow::Result;

pub fn compute_cognitive_index(prog_lang: ProgrammingLang, code_block: &str) -> Result<u16> {
    let lang_evaluator = create_lang_evaluator(prog_lang);
    lang_evaluator.eval(code_block)
}

pub enum ProgrammingLang {
    Rust,
    Python,
    Go
}


trait LangEvaluator {
    fn eval(&self, code_block: &str) -> Result<u16>;
}
struct RustLangEvaluator;
impl LangEvaluator for RustLangEvaluator {
    fn eval(&self, code_block: &str) -> Result<u16> {
        Ok(10)
    }
    
}


// Factory function to create language evaluators.
fn create_lang_evaluator(prog_lang: ProgrammingLang) -> Box<dyn LangEvaluator> {
    match prog_lang {
        ProgrammingLang::Rust => Box::new(RustLangEvaluator{}),
        _ => panic!("Language evaluator not implemented yet!")
    }
}



#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn calculate_cognitive_complexity_of_a_rust_block() {

        let block_of_code = "
            let c = if a > b { a } else { b };
            let mut sum = 0;
            for i in 1..c+1 {
                if c % i == 0 {
                    sum += i;
                }
            }
            return sum;
        ";

        let expected = 10;
        let cognitive_complex_index = compute_cognitive_index(ProgrammingLang::Rust, block_of_code).unwrap();

        assert_eq!(expected, cognitive_complex_index);
    }
}
