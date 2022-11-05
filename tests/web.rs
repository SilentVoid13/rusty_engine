#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

use rusty_engine::{Command, CommandType, Parser, ParserConfig, Token, Whitespace};

#[wasm_bindgen_test]
pub fn test_parse_tokens() {
    let config = ParserConfig::new(
        "<%".into(),
        "%>".into(),
        '\0',
        '*',
        '-',
        '_',
        "tR".into(),
    );
    let content = r#"
test<%_ test %>test
<%- test _%>
test
<%*_ test -%> test <% test %>
test"#;

    let parser = Parser::new(content, &config);
    let tokens = parser.parse_tokens().unwrap();

    assert_eq!(
        tokens,
        vec![
            Token::Text("\ntest"),
            Token::Command(Command {
                r#type: CommandType::Interpolate,
                content: " test ",
                opening_whitespace: Some(Whitespace::Multiple),
                closing_whitespace: None,
            }),
            Token::Text("test\n"),
            Token::Command(Command {
                r#type: CommandType::Interpolate,
                content: " test ",
                opening_whitespace: Some(Whitespace::Single),
                closing_whitespace: Some(Whitespace::Multiple),
            }),
            Token::Text("\ntest\n"),
            Token::Command(Command {
                r#type: CommandType::Execution,
                content: " test ",
                opening_whitespace: Some(Whitespace::Multiple),
                closing_whitespace: Some(Whitespace::Single),
            }),
            Token::Text(" test "),
            Token::Command(Command {
                r#type: CommandType::Interpolate,
                content: " test ",
                opening_whitespace: None,
                closing_whitespace: None,
            }),
            Token::Text("\ntest"),
        ]
    );
}

#[wasm_bindgen_test]
pub fn test_generate_js() {
    let config = ParserConfig::new(
        "<%".into(),
        "%>".into(),
        '\0',
        '*',
        '-',
        '_',
        "tR".into(),
    );
    let content = "<%- test -%>\ntest\n\n<%*_ test %>'test'<% test %>";
    let parser = Parser::new(content, &config);
    let tokens = parser.parse_tokens().unwrap();
    let js_func = parser.generate_js(tokens);
    assert_eq!(
        js_func,
        String::from(
            r#"let __prs = [];
let tR = '';
__prs.push( test );
tR+='rJ2KqXzxQg';
tR+='test';
 test ;
tR+='\'test\'';
__prs.push( test );
tR+='rJ2KqXzxQg';
const __rst = await Promise.all(__prs);
tR = tR.replace(/rJ2KqXzxQg/g, () => __rst.shift());
return tR;
"#
        )
    );
}

#[wasm_bindgen_test]
pub fn test_whitespace_control() {
    let config = ParserConfig::new(
        "<%".into(),
        "%>".into(),
        '\0',
        '*',
        '-',
        '_',
        "tR".into(),
    );
    let content = "\ntest\n\n<%_ test -%>\r\n\ntest\n\r<%*- test _%>\rtest\r\n<%*- test -%> test <% test %>\ntest";

    let parser = Parser::new(content, &config);
    let tokens = parser.parse_tokens().unwrap();
    let res = parser.generate_js(tokens);
    assert_eq!(
        res,
        String::from(
            r#"let __prs = [];
let tR = '';
tR+='\ntest';
__prs.push( test );
tR+='rJ2KqXzxQg';
tR+='\ntest\n';
 test ;
tR+='test';
 test ;
tR+=' test ';
__prs.push( test );
tR+='rJ2KqXzxQg';
tR+='\ntest';
const __rst = await Promise.all(__prs);
tR = tR.replace(/rJ2KqXzxQg/g, () => __rst.shift());
return tR;
"#
        )
    );
}
