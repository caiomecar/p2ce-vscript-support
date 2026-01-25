#[cfg(test)]
mod tests {
    use sq_3_parser::*;

    #[test]
    fn test_parser() {
        //  1  * 12321
        const SOURCE: &'static str = "local function abc[123](abc = 2){}";
        let parse = Parse::new(SOURCE);
        eprintln!("{:?}", parse.errors());
        eprintln!("{:#?}", parse.into_syntax());
    }

    #[test]
    fn test_empty() {
        const SOURCE: &'static str = "";
        let parse = Parse::new(SOURCE);
        eprintln!("{:?}", parse.errors());
        eprintln!("{:#?}", parse.into_syntax());
    }
}
