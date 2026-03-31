use std::sync::Arc;
use swiftlet::preclude::*;

const GRAMMAR: &str = r#"
start: request

request: get_request
    | post_request

get_request: request_line_get headers NEWLINE
post_request: request_line_post headers NEWLINE body?

request_line_get: "GET" SP REQUEST_TARGET SP HTTP_VERSION NEWLINE
request_line_post: "POST" SP REQUEST_TARGET SP HTTP_VERSION NEWLINE

headers: HEADER_LINE*
body: JSON_BODY

REQUEST_TARGET: /\/[^\r\n ]*/
HTTP_VERSION: /HTTP\/(1\.0|1\.1|2\.0)/
SP: " "
HEADER_LINE: /[A-Za-z][A-Za-z0-9-]*: [^\r\n]*\r?\n/
JSON_BODY: /\{[\s\S]*\}/
NEWLINE: /\r?\n/
"#;

fn main() {
    let parser_opt = Arc::new(
        ParserOption {
            algorithm: Algorithm::Earley,
            debug: false,
            ..Default::default()
        }
    );
    match Swiftlet::from_string(GRAMMAR, parser_opt) {
        Ok(parser) => {
            let texts = [
                "GET /users?id=42&active=true HTTP/1.1\r\nHost: example.com\r\nAccept: application/json\r\n\r\n",
                "POST /api/messages HTTP/1.1\r\nHost: example.com\r\nContent-Type: application/json\r\nContent-Length: 17\r\n\r\n{\"message\":\"hi\"}",
            ];
            let prefix_text = "HTTP request: ";
            texts.into_iter().for_each(|text| {
                println!("{}", "-".repeat(text.len() + prefix_text.len()));
                println!("{}{}", prefix_text, text);
                println!("{}", "-".repeat(text.len() + prefix_text.len()));
                let parsed = parser.parse(text);
                println!("AST =>");
                match parsed {
                    Ok(ast) => {
                        ast.pretty_print();
                        println!();
                    }
                    Err(err) => {
                        eprintln!("{err}");
                    }
                }
            });
        },
        Err(err) => {
            eprintln!("{}", err);
        }
    }
}
