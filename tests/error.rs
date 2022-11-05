#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;
use wasm_bindgen::prelude::*;

use rusty_engine::{error::RenderError, Parser, ParserConfig, Renderer};

#[wasm_bindgen_test]
pub fn test_missing_closing_tag() {
    let config = ParserConfig::new("<%".into(), "%>".into(), '\0', '*', '-', '_', "tR".into());
    let content = r#"test
<%_ test -%>
test
<% test"#;

    let parser = Parser::new(content, &config);
    let tokens = parser.parse_tokens();
    assert_eq!(tokens, Err(RenderError::MissingClosingTag));
}

#[wasm_bindgen_test]
pub fn test_missing_command_tag() {
    let config = ParserConfig::new("<%".into(), "%>".into(), '~', '*', '-', '_', "tR".into());
    let content = r#"test <% test %> test"#;

    let parser = Parser::new(content, &config);
    let tokens = parser.parse_tokens();
    assert_eq!(tokens, Err(RenderError::MissingCommandTag));
}

#[wasm_bindgen_test]
pub fn test_syntax_error() {
    let config = ParserConfig::new("<%".into(), "%>".into(), '\0', '*', '-', '_', "tR".into());
    let content = r#"test <%* / %> test"#;

    let renderer = Renderer::new(config);
    let res = renderer.render_content(content, &JsValue::NULL);
    assert_eq!(res, Err(RenderError::SyntaxError));
}
