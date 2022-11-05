use thiserror::Error;
use wasm_bindgen::JsValue;

pub type Result<T> = std::result::Result<T, RenderError>;

#[derive(Error, Debug, PartialEq)]
pub enum RenderError {
    #[error("Syntax error")]
    SyntaxError,
    #[error("Function error")]
    FunctionError,
    #[error("Missing command tag")]
    MissingCommandTag,
    #[error("Missing closing tag")]
    MissingClosingTag,
}

impl Into<JsValue> for RenderError {
    fn into(self) -> JsValue {
        self.to_string().into()
    }
}
