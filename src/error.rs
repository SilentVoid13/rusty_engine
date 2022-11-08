use thiserror::Error;
use wasm_bindgen::JsValue;

pub type Result<T> = std::result::Result<T, RenderError>;

#[derive(Error, Debug, PartialEq)]
pub enum RenderError {
    #[error("Syntax error inside command")]
    SyntaxError,
    #[error("Template function call error")]
    FunctionError,
    #[error("Missing command type at `{0}`")]
    MissingCommandType(String),
    #[error("Missing closing command tag at `{0}`")]
    MissingClosingTag(String),
}

impl Into<JsValue> for RenderError {
    fn into(self) -> JsValue {
        self.to_string().into()
    }
}
