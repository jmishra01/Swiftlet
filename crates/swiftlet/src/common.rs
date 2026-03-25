use crate::lexer::{RegexFlag, TerminalDef};
use crate::terminal_def;
use std::collections::HashMap;
use std::sync::Arc;

/// Returns built-in terminal definitions available through `%import`.
pub(crate) fn get_common_terminals() -> HashMap<String, Arc<TerminalDef>> {
    let digit = r"\d";
    let hex_digit = r"[a-fA-F0-9]+";
    let integer = r"\d+";
    let signed_integer = r"(-|\+)\d+";
    let negative = r"-\s*\d+";
    let decimal = r"\d+\.\d+";
    let lower_case_letter = "[a-z]";
    let upper_case_letter = "[A-Z]";
    let letter = "[a-zA-Z]";
    let word = format!("{letter}+");
    let cname = "[_a-zA-Z][_a-zA-Z0-9]+";
    let ws_inline = r"(\s|\t)+";
    let ws = r"[ \t\f\r\n]+";
    let cr = r"\r";
    let lf = r"\n";
    let newline = r"(\r?\n)+";
    let sh_comment = r"#[^\n]*";
    // let string_pattern = r#""([^"\\]*(\\[^"]*)*)""#;
    let string_pattern = r#"".*?""#;
    let quote_pattern = "'.*?'";

    HashMap::from([
        (
            "WS".to_string(),
            terminal_def!("WS", ws, RegexFlag::default(), 0),
        ),
        (
            "DIGIT".to_string(),
            terminal_def!("DIGIT", digit, RegexFlag::default(), 0),
        ),
        (
            "HEXDIGIT".to_string(),
            terminal_def!("HEXDIGIT", hex_digit, RegexFlag::default(), 0),
        ),
        (
            "INT".to_string(),
            terminal_def!("INT", integer, RegexFlag::default(), 0),
        ),
        (
            "N_INT".to_string(),
            terminal_def!("N_INT", negative, RegexFlag::default(), 0),
        ),
        (
            "SIGNED_INT".to_string(),
            terminal_def!("SIGNED_INT", signed_integer, RegexFlag::default(), 0),
        ),
        (
            "DECIMAL".to_string(),
            terminal_def!("DECIMAL", decimal, RegexFlag::default(), 0),
        ),
        (
            "LCASE_LETTER".to_string(),
            terminal_def!("LCASE_LETTER", lower_case_letter, RegexFlag::default(), 0),
        ),
        (
            "UCASE_LETTER".to_string(),
            terminal_def!("UCASE_LETTER", upper_case_letter, RegexFlag::default(), 0),
        ),
        (
            "WORD".to_string(),
            terminal_def!("WORD", word.as_str(), RegexFlag::default(), 0),
        ),
        (
            "CNAME".to_string(),
            terminal_def!("CNAME", cname, RegexFlag::default(), 0),
        ),
        (
            "WS_INLINE".to_string(),
            terminal_def!("WS_INLINE", ws_inline, RegexFlag::default(), 0),
        ),
        (
            "CR".to_string(),
            terminal_def!("CR", cr, RegexFlag::default(), 0),
        ),
        (
            "LF".to_string(),
            terminal_def!("LF", lf, RegexFlag::default(), 0),
        ),
        (
            "NEWLINE".to_string(),
            terminal_def!("NEWLINE", newline, RegexFlag::default(), 0),
        ),
        (
            "SH_COMMENT".to_string(),
            terminal_def!("SH_COMMENT", sh_comment, RegexFlag::default(), 0),
        ),
        (
            "STRING".to_string(),
            terminal_def!("STRING", string_pattern, RegexFlag::default(), 0),
        ),
        (
            "QUOTE".to_string(),
            terminal_def!("QUOTE", quote_pattern, RegexFlag::default(), 0),
        ),
    ])
}
