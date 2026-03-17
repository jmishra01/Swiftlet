# Grammar Reference

A grammar defines the structure of a language and determines how text
input is transformed into a **parse tree** or **AST (Abstract Syntax
Tree)**.

The grammar syntax is inspired by [EBNF](https://en.wikipedia.org/wiki/Extended_Backus–Naur_form) and [Lark](https://lark-parser.readthedocs.io/en/stable/index.html), while being
optimized for high-performance parsing in Rust.

A grammar consists of two main components:

-   **Rules** --- define hierarchical language structure
-   **Terminals** --- define atomic tokens recognized by the lexer

------------------------------------------------------------------------

# 1. Grammar Structure

A grammar file consists of **rule definitions**, **terminal
definitions**, and **directives**.

Example:

    start: expr

    expr: expr "+" term
        | term

    term: NUMBER

    NUMBER: /\d+/

    %ignore " "

In this grammar:

  Element          Description
  ---------------- ---------------------------
  `start`          entry point of the parser
  `expr`, `term`   grammar rules
  `NUMBER`         terminal token
  `%ignore`        grammar directive

------------------------------------------------------------------------

# 2. Rules

Rules define **syntactic structures**.

Rule names must be **lowercase**.

### Syntax

    rule_name: definition

Example:

    sentence: noun_phrase verb_phrase
    noun_phrase: "the" noun
    verb_phrase: verb noun_phrase

Rules can reference:

-   terminals
-   other rules
-   string literals
-   grouped expressions

Rules form the **hierarchical structure of the parse tree**.

Example: [gr_rule.rs](crates/swiftlet/examples/gr_rule.rs)

------------------------------------------------------------------------

# 3. Terminals

Terminals represent **tokens produced by the lexer**.

Terminal names must be **uppercase**.

### Syntax

    TERMINAL: definition

Example:

    NUMBER: /\d+/
    PLUS: "+"
    IDENTIFIER: /[a-zA-Z_][a-zA-Z0-9_]*/

Terminals can be defined using:

-   string literals
-   regular expressions
-   combinations of other terminals

Example: [gr_terminals.rs](crates/swiftlet/examples/gr_terminals.rs)

------------------------------------------------------------------------

# 4. String Literals

String literals match exact text in the input.

Example:

    PLUS: "+"
    MINUS: "-"
    LPAREN: "("

They are useful for operators and keywords.

Example: [gr_string_literals.rs](crates/swiftlet/examples/gr_string_literals.rs)

------------------------------------------------------------------------

# 5. Regular Expressions

Regular expressions allow flexible token definitions.

Example:

    NUMBER: /\d+/
    FLOAT: /\d+\.\d+/
    NAME: /[a-zA-Z_]+/

Regex terminals are compiled into the lexer for efficient matching.

Example: [gr_regular_expression.rs](crates/swiftlet/examples/gr_regular_expression.rs)

------------------------------------------------------------------------

# 6. Alternatives

Rules may contain multiple alternatives using `|`.

Example:

    expr: expr "+" term
        | expr "-" term
        | term

This means that `expr` can match any of the listed alternatives.


Example: [gr_alternatives.rs](crates/swiftlet/examples/gr_alternatives.rs)

------------------------------------------------------------------------

# 7. Repetition Operators

Grammar expressions support repetition operators.

| Operator | Meaning      |
|----------|--------------|
| `*`      | zero or more |
| `+`      | one or more  |
| `?`      | optional     |

Example:

    list: NUMBER ("," NUMBER)*

Matches:

    1
    1,2
    1,2,3

Example: [gr_repetition_operators.rs](crates/swiftlet/examples/gr_repetition_operators.rs)

------------------------------------------------------------------------

# 8. Grouping

Parentheses allow grouping expressions.

Example:

    expr: term ("+" term)*

Grouping controls:

-   repetition scope
-   precedence
-   expression structure


Example:
* [gr_repetition_operators.rs](crates/swiftlet/examples/gr_repetition_operators.rs)
* [gr_grouping.rs](crates/swiftlet/examples/gr_grouping.rs)

------------------------------------------------------------------------

# 9. Terminal Priority

When multiple terminals match the same text, **priority determines which
token wins**.

### Syntax

    TOKEN.priority: pattern

Example:

    KEYWORD.10: "select"
    IDENTIFIER: /[a-zA-Z_]+/

Higher priority terminals are matched first.

------------------------------------------------------------------------

# 10. Regex Flags

Regular expressions support optional flags.

Example:

    SELECT: "select"i

Matches:

    select
    SELECT
    Select

Supported flags:

| Flag | Description         |
|------|---------------------|
| `i`  | ignore case         |
| `m`  | multiline           |
| `s`  | dot matches newline |
| `u`  | unicode             |
| `x`  | verbose regex       |


Example: [gr_regex_flags.rs](crates/swiftlet/examples/gr_regex_flags.rs)


------------------------------------------------------------------------

# 11. Token Matching Priority

When multiple terminals match the same input, resolution follows this
order:

1.  Highest **priority**
2.  **Longest regex match**
3.  **Longest literal**
4.  **Terminal definition order**

This ensures deterministic token selection.

------------------------------------------------------------------------

# 12. Grammar Directives

Directives modify grammar behavior.

They start with `%`.

## Ignore

The `%ignore` directive tells the lexer to ignore certain tokens.

Example:

    %ignore " "
    %ignore /\t+/
    %ignore /\n+/

Typically used for:

-   whitespace
-   comments
-   formatting characters

Ignored tokens do not appear in the parse tree.

Example: [gr_ignore.rs](crates/swiftlet/examples/gr_ignore.rs)

------------------------------------------------------------------------

## Import

The `%import` directive imports grammar definitions from other modules.

Example:

    %import NUMBER
    %import WS

Or

    %import ( NUMBER, WS )

This allows grammar reuse and modular design.

**List of Common Terminals**

| Terminal Name | Description                  |
|---------------|------------------------------|
| CWORD         | Match words separated by `_` |
| DECIMAL       | Match decimal number         |
| DIGIT         | Match single digit number    |
| HEXDIGIT      | Match hex-digit number       |
| INT           | Match number                 |
| LCASE_LETTER  | Match lower-case characters  |
| NEWLINE       | Match new line               |
| N_INT         | Match negative number        |
| QUOTE         | Match word wrapped with `'`  |
| SIGNED_INT    | Match Signed number          |
| STRING        | Match word wrapped with `"`  |
| UCASE_LETTER  | Match upper-case characters  |
| WORD          | Match characters             |
| WS            | Match white space            |

Example: [gr_import.rs](crates/swiftlet/examples/gr_import.rs)

------------------------------------------------------------------------

# 13. Parse Tree Construction

The parser builds a **parse tree** according to the rule hierarchy.

Example grammar:

    expr: expr "+" term
        | term

    term: NUMBER

Input:

    3 + 5

Parse tree:

    expr
     ├─ expr
     │   └─ term
     │       └─ NUMBER(3)
     ├─ "+"
     └─ term
         └─ NUMBER(5)

Each rule becomes a **node in the parse tree**.

------------------------------------------------------------------------

# 14. Operator Precedence Example

Example grammar supporting operator precedence:

    start: expr

    expr: expr "+" term
        | expr "-" term
        | term

    term: term "*" factor
        | term "/" factor
        | factor

    factor: NUMBER
          | "(" expr ")"

    NUMBER: /\d+/

    %import WS
    %ignore WS

Input:

    3 + 4 * 5

The parse tree correctly reflects multiplication having higher
precedence than addition.

------------------------------------------------------------------------

# 15. Best Practices

To write maintainable grammars:

1. Keep rules small `Avoid very long rules`.
2. Avoid ambiguous terminals `Ensure tokens do not overlap unnecessarily`.
3. Use priorities when necessary `Resolve token conflicts explicitly`.
4. Use imports for large grammars `Break grammars into reusable modules`.
5. Ignore whitespace `Always define whitespace rules`.

------------------------------------------------------------------------

# 16. Complete Example Grammar

    start: statement+

    statement: assignment
             | expr

    assignment: NAME "=" expr

    expr: expr "+" term
        | expr "-" term
        | term

    term: term "*" factor
        | term "/" factor
        | factor

    factor: NUMBER
          | NAME
          | "(" expr ")"

    NAME: /[a-zA-Z_]+/
    NUMBER: /\d+/
  
    %import WS
    %ignore WS
    %ignore /\t+/

Example input:

    x = 10
    y = x + 5 * 3

This grammar supports:

-   assignments
-   arithmetic expressions
-   variables

------------------------------------------------------------------------

# 17. Summary

A grammar describes how text input is structured and parsed.

Key concepts include:

-   **rules** for structure
-   **terminals** for tokens
-   **operators** for repetition
-   **directives** for grammar behavior

The grammar system enables building **robust, high-performance parsers**
for programming languages, data formats, and DSLs.
