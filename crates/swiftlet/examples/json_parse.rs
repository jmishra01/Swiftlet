use std::sync::Arc;
use swiftlet::{ParserOption, Swiftlet};

fn main() {
    let text = r#"
        ?start: value
        ?value: object | array
        ?atom: value | string | INT
        object: "{" members? "}"
        ?members: members "," pair | pair
        pair: string ":" atom
        array: "[" _elements? "]"
        _elements: _elements "," atom | atom
        ?string: STRING
        %import (WS, INT, STRING)
        %ignore WS
        "#;

    let conf = Arc::new(ParserOption::default());

    let text_parser = Swiftlet::from_string(text, conf);

    let res = text_parser.parse(r#"{"hello": ["world", "second", "third"]}"#);
    if let Ok(ast) = res {
        ast.pretty_print();
    }
}
