use barat::{Barat, ParserOption};
use std::sync::Arc;

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
        "#
        .to_string();

    let conf = Arc::new(ParserOption::default());

    let mut text_parser = Barat::from_string(text, conf);

    let res = text_parser.parse(r#"{"hello": ["world", "second", "third"]}"#);
    if let Ok(ast) = res {
        ast.pretty_print();
    }
}
