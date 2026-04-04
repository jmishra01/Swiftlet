use std::collections::HashMap;
use swiftlet::preclude::*;
//
// const GRAMMAR: &str = r#"
// start: assignment+
// assignment: KEY "=" value
// ?value: quoted | bare
// quoted: STRING
// bare: BARE
//
// KEY: /[A-Z_][A-Z0-9_]*/
// BARE: /[^\s#\n]+/
//
// %import (WS, STRING, SH_COMMENT)
// %ignore WS
// %ignore SH_COMMENT
// "#;

const GRAMMAR: &str = r#"
start: line*
?line: assignment _NL
    | COMMENT _NL
    | _NL
assignment: KEY "=" value
?value: quoted | bare
quoted: STRING
bare: BARE

KEY: /[A-Z_][A-Z0-9_]*/
BARE: /[^\s#\n]+/
COMMENT: SH_COMMENT
_NL: NEWLINE

%import (WS_INLINE, STRING, SH_COMMENT, NEWLINE)
%ignore WS_INLINE
"#;

#[derive(Debug)]
struct AppConfig {
    host: String,
    port: u16,
    debug: bool,
    workers: usize,
    database_url: String,
    log_level: String,
}

fn token_word(ast: &Ast) -> Option<&str> {
    match ast {
        Ast::Token(token) => Some(token.word()),
        Ast::Tree(_, children) => children.iter().find_map(token_word),
    }
}

fn unquote(value: &str) -> String {
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}

fn parse_assignment(ast: &Ast) -> Option<(String, String)> {
    let Ast::Tree(name, children) = ast else {
        return None;
    };

    if name != "assignment" || children.len() < 3 {
        return None;
    }

    let key = token_word(&children[0])?.to_string();
    let value = token_word(children.last()?)?;
    Some((key, unquote(value)))
}

fn ast_to_env(ast: &Ast) -> HashMap<String, String> {
    ast.iter_trees("assignment")
        .filter_map(parse_assignment)
        .collect()
}

fn require<'a>(env: &'a HashMap<String, String>, key: &str) -> Result<&'a str, String> {
    env.get(key)
        .map(String::as_str)
        .ok_or_else(|| format!("missing required key: {key}"))
}

fn parse_bool(value: &str) -> Result<bool, String> {
    match value {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => Err(format!("invalid boolean value: {value}")),
    }
}

fn build_config(env: &HashMap<String, String>) -> Result<AppConfig, String> {
    Ok(AppConfig {
        host: require(env, "HOST")?.to_string(),
        port: require(env, "PORT")?
            .parse()
            .map_err(|_| "PORT must be a valid u16".to_string())?,
        debug: parse_bool(require(env, "DEBUG")?)?,
        workers: require(env, "WORKERS")?
            .parse()
            .map_err(|_| "WORKERS must be a valid usize".to_string())?,
        database_url: require(env, "DATABASE_URL")?.to_string(),
        log_level: env
            .get("LOG_LEVEL")
            .cloned()
            .unwrap_or_else(|| "info".to_string()),
    })
}

fn main() {
    let input = r#"
# service settings
HOST=127.0.0.1
PORT=8080
DEBUG=true
WORKERS=4
DATABASE_URL="postgres://swiftlet:secret@localhost:5432/app"
LOG_LEVEL=debug
"#;

    let parser_opt = Arc::new(ParserConfig {
        algorithm: Algorithm::CLR,
        start: "start".to_string(),
        ..Default::default()
    });

    let parser = Swiftlet::from_str(GRAMMAR)
        .map(|grammar| grammar.parser(parser_opt))
        .expect("failed to build parser");

    match parser.parse(input) {
        Ok(ast) => {
            println!("AST");
            ast.pretty_print();

            let env = ast_to_env(&ast);
            match build_config(&env) {
                Ok(config) => {
                    println!("\nTyped config");
                    println!("{config:#?}");
                    println!(
                        "\nService will start on {}:{} with {} workers, debug={}, log_level={}",
                        config.host, config.port, config.workers, config.debug, config.log_level
                    );
                    println!("Database: {}", config.database_url);
                }
                Err(err) => eprintln!("invalid config: {err}"),
            }
        }
        Err(err) => eprintln!("parse error: {err}"),
    }
}
