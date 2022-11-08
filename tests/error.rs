#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;
use wasm_bindgen::prelude::*;

use rusty_engine::{error::RenderError, Parser, ParserConfig, Renderer};

#[wasm_bindgen_test]
pub fn test_missing_closing_tag() {
    let config = ParserConfig::new("<%".into(), "%>".into(), '\0', '*', '-', '_', "tR".into());
    let content = "test\r\n<%_ test1 -%>\r\ntest\r\ntesttest <% test2";
    let err = r#"line 4 col 11:

testtest <% test2
          ^"#;

    let parser = Parser::new(content, &config);
    let tokens = parser.parse_tokens();
    assert_eq!(tokens, Err(RenderError::MissingClosingTag(err.into())));
}

#[wasm_bindgen_test]
pub fn test_missing_command_tag() {
    let config = ParserConfig::new("<%".into(), "%>".into(), '~', '*', '-', '_', "tR".into());
    let content = r#"test 
test <% test %> test"#;
    let err = r#"line 2 col 7:

test <% test %> test
      ^"#;

    let parser = Parser::new(content, &config);
    let tokens = parser.parse_tokens();
    assert_eq!(tokens, Err(RenderError::MissingCommandType(err.into())));
}

#[wasm_bindgen_test]
pub fn test_syntax_error() {
    let config = ParserConfig::new("<%".into(), "%>".into(), '\0', '*', '-', '_', "tR".into());
    let content = r#"test <%* / %> test"#;

    let renderer = Renderer::new(config);
    let res = renderer.render_content(content, &JsValue::NULL);
    assert_eq!(res, Err(RenderError::SyntaxError));
}
