use std::sync::Arc;
use swiftlet::preclude::*;

const GRAMMAR: &str = r#"
start: message

message: request
    | response

request: request_line header_section CRLF message_body?
response: status_line header_section CRLF message_body?

request_line: method SP request_target SP http_version CRLF
status_line: http_version SP status_code SP reason_phrase CRLF

header_section: (header_line CRLF)*
header_line: field_name ":" field_value?

method: METHOD
request_target: REQUEST_TARGET
http_version: HTTP_VERSION
status_code: STATUS_CODE
reason_phrase: REASON_PHRASE
field_name: TOKEN
field_value: FIELD_VALUE
message_body: BODY

CRLF: /\r\n/
SP: " "
METHOD: /(GET|POST|PUT|PATCH|DELETE|HEAD|OPTIONS|TRACE|CONNECT)/
HTTP_VERSION: /HTTP\/\d\.\d/
STATUS_CODE: /\d{3}/
REQUEST_TARGET: /[^ \t\r\n]+/
REASON_PHRASE: /[^\r\n]+/
TOKEN: /[!#$%&'*+.^_`|~0-9A-Za-z-]+/
FIELD_VALUE: /[ \t]*[^\r\n]+/
BODY: /.+/s
"#;

fn main() {
    let parser_opt = Arc::new(ParserOption {
        algorithm: Algorithm::Earley,
        debug: false,
        ..Default::default()
    });
    match Swiftlet::from_string(GRAMMAR, parser_opt) {
        Ok(parser) => {
            let texts = [
                "GET /users?id=42&active=true HTTP/1.1\r\nHost: example.com\r\nAccept: application/json\r\n\r\n",
                "POST /api/messages HTTP/1.1\r\nHost: example.com\r\nContent-Type: application/json\r\nContent-Length: 17\r\n\r\n{\"message\":\"hi\"}",
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 5\r\n\r\nhello",
            ];
            let prefix_text = "HTTP message: ";
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
        }
        Err(err) => {
            eprintln!("{}", err);
        }
    }
}
