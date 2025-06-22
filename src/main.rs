pub mod frontend;
pub mod errors;
pub mod sema;

use crate::frontend::{
    lexer::Lexer,
    parser::Parser,
    utils::visitor::Visitor,
};

use crate::sema::{
    utils::MultiStageSymbolTable,
    passes::{
        populate_table::FullSymbolTablePass,
        type_checker::TypeChecker
    }
};

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    let path = Path::new(file_path);

    if !path.exists() {
        eprintln!("File not found: {}", file_path);
        std::process::exit(1);
    }

    let mut file = File::open(path).expect("Unable to open file");
    let mut source_code = Default::default();
    file.read_to_string(&mut source_code).expect("Unable to read file");

    let mut lexer = Lexer::new(&source_code, path.to_str().unwrap().to_string());
    let start = Instant::now();
    lexer.scan_tokens();
    let duration = start.elapsed();
    println!("Lexing took: {:?}", duration);

    let tokens = lexer.tokens.clone();

    // let interpolated_strings = si::extract_interpolated_strings(&lexer.tokens);
    // let mut tokenised: Vec<Vec<Token>> = vec![];
    // for (_, interp) in &interpolated_strings {
    //     tokenised = interp.tokenize_interpolations(|expr, offset| {
    //         let mut sublexer = Lexer::new(expr, path.to_str().unwrap().to_string());

    //         sublexer.set_offset(offset, interp.interpolations[0].line);
    //         sublexer.scan_tokens();

    //         if let Some(Token { kind: TokenKind::Eof, .. }) = sublexer.tokens.last() {
    //             sublexer.tokens.pop();
    //         }
    //         sublexer.tokens
    //     });
    // }

    // for tokens in tokenised {
    //     for token in tokens {
    //         println!("{:?}", token);
    //     }
    // }

    let mut parser = Parser::new(&tokens, &source_code, path.to_str().unwrap().to_string());

    let start = Instant::now();
    let module = parser.parse();
    let duration = start.elapsed();
    println!("Parsing took: {:?}", duration);

    for statement in &module.stmts {
        println!("{:#?}", statement);
    }

    // First populate the symbol table
    let mut table = MultiStageSymbolTable::new();
    let mut pass = FullSymbolTablePass { table };

    let start = Instant::now();
    FullSymbolTablePass::visit_module(&mut pass, &module).expect("Failed to populate symbol table");
    let duration = start.elapsed();
    println!("Symbol table population took: {:?}", duration);
    println!("Symbol table: {:#?}", pass.table);

    // Then run the type checker
    let mut type_checker = TypeChecker::new();
    type_checker.table = pass.table; // Transfer the populated symbol table

    let start = Instant::now();
    TypeChecker::visit_module(&mut type_checker, &module).expect("Failed to type check");
    let duration = start.elapsed();
    println!("Type checking took: {:?}", duration);

    // Report any type errors
    if !type_checker.errors.is_empty() {
        println!("\nType errors found:");
        for error in type_checker.errors {
            println!("{}", error);
        }
        std::process::exit(1);
    } else {
        println!("\nType checking passed successfully!");
    }
}
