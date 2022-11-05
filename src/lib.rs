extern crate wasm_bindgen;

pub mod error;
mod utils;

use std::collections::HashMap;
use std::fmt::Write;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::error::RenderError;
use crate::error::Result;
use crate::utils::log;

#[derive(Debug, PartialEq)]
pub enum CommandType {
    Interpolate,
    Execution,
}

#[derive(Debug, PartialEq)]
pub enum Whitespace {
    Single,
    Multiple,
}

#[derive(Debug, PartialEq)]
pub struct Command<'a> {
    pub r#type: CommandType,
    pub opening_whitespace: Option<Whitespace>,
    pub closing_whitespace: Option<Whitespace>,
    pub content: &'a str,
}

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    Text(&'a str),
    Command(Command<'a>),
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug)]
pub struct ParserConfig {
    opening_tag: String,
    closing_tag: String,
    pub interpolate: char,
    pub execution: char,
    pub single_whitespace: char,
    pub multiple_whitespace: char,
    global_var: String,
}

#[wasm_bindgen]
impl ParserConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(
        opt: String,
        clt: String,
        inte: char,
        ex: char,
        sw: char,
        mw: char,
        gv: String,
    ) -> ParserConfig {
        ParserConfig {
            opening_tag: opt,
            closing_tag: clt,
            interpolate: inte,
            execution: ex,
            single_whitespace: sw,
            multiple_whitespace: mw,
            global_var: gv,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn opening_tag(&self) -> String {
        self.opening_tag.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_opening_tag(&mut self, val: String) {
        self.opening_tag = val;
    }

    #[wasm_bindgen(getter)]
    pub fn closing_tag(&self) -> String {
        self.closing_tag.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_closing_tag(&mut self, val: String) {
        self.closing_tag = val;
    }

    #[wasm_bindgen(getter)]
    pub fn global_var(&self) -> String {
        self.global_var.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_global_var(&mut self, val: String) {
        self.global_var = val;
    }
}

pub struct Parser<'a> {
    content: &'a str,
    config: &'a ParserConfig,
}

impl<'a> Parser<'a> {
    pub fn new(content: &'a str, config: &'a ParserConfig) -> Self {
        Parser { content, config }
    }

    fn parse_command_tag(&self, i: &'a str) -> Result<(CommandType, &'a str)> {
        let c = i.chars().next();
        let c = match c {
            Some(c) => c,
            None => return Err(RenderError::MissingCommandTag),
        };

        // TODO: improve this
        let mut input = i;
        let cmd_type = if c == self.config.execution {
            input = &i[1..];
            CommandType::Execution
        } else if c == self.config.interpolate {
            input = &i[1..];
            CommandType::Interpolate
        } else {
            if self.config.interpolate == '\0' {
                CommandType::Interpolate
            } else if self.config.execution == '\0' {
                CommandType::Execution
            } else {
                return Err(RenderError::MissingCommandTag);
            }
        };

        Ok((cmd_type, input))
    }

    fn parse_whitespace(&self, i: &'a str) -> Result<(Option<Whitespace>, &'a str)> {
        let c = i.chars().next();
        let whitespace = match c {
            Some(c) => {
                if c == self.config.multiple_whitespace {
                    Some(Whitespace::Multiple)
                } else if c == self.config.single_whitespace {
                    Some(Whitespace::Single)
                } else {
                    None
                }
            }
            None => None,
        };
        let mut input = i;
        if whitespace.is_some() {
            input = &i[1..];
        }
        Ok((whitespace, input))
    }

    fn parse_closing_tag(&self, i: &'a str) -> Result<(&'a str, &'a str)> {
        let (content, i) = match i.split_once(&self.config.closing_tag) {
            Some(x) => x,
            None => return Err(RenderError::MissingClosingTag),
        };
        Ok((content, i))
    }

    pub fn escape_text(&self, i: &str) -> String {
        let escape_chars = HashMap::from([('\\', '\\'), ('\n', 'n'), ('\r', 'r'), ('\'', '\'')]);
        let mut res = vec![];
        for c in i.chars() {
            if escape_chars.contains_key(&c) {
                res.push('\\');
                res.push(*escape_chars.get(&c).unwrap());
            } else {
                res.push(c);
            }
        }
        res.iter().collect()
    }

    pub fn trim_whitespace<'b>(
        &self,
        i: &'b str,
        whitespace: Option<&Whitespace>,
        left: bool,
    ) -> &'b str {
        let text = match whitespace {
            Some(Whitespace::Single) => {
                let mut chars = i.chars();
                let mut revs = i.chars().rev();
                let chars: &mut dyn Iterator<Item = char> =
                    if left { &mut chars } else { &mut revs };
                let mut chars = chars.peekable();

                let x = match chars.peek() {
                    Some('\n') => {
                        let mut x = 1;
                        if !left {
                            chars.next().unwrap();
                            if let Some('\r') = chars.peek() {
                                x = 2
                            }
                        }
                        x
                    },
                    Some('\r') => {
                        let mut x = 1;
                        if left {
                            chars.next().unwrap();
                            if let Some('\n') = chars.peek() {
                                x = 2
                            }
                        }
                        x
                    }
                    _ => 0,
                };

                if left {
                    &i[x..]
                } else {
                    &i[..i.len() - x]
                }
            }
            Some(Whitespace::Multiple) => {
                if left {
                    i.trim_start()
                } else {
                    i.trim_end()
                }
            }
            None => i,
        };
        text
    }

    pub fn parse_tokens(&self) -> Result<Vec<Token>> {
        let mut tokens = vec![];
        let mut input = self.content;

        while let Some((text, i)) = input.split_once(&self.config.opening_tag) {
            if !text.is_empty() {
                tokens.push(Token::Text(text));
            }

            let (cmd_type, i) = self.parse_command_tag(i)?;
            let (opening_whitespace, i) = self.parse_whitespace(i)?;
            let (part1, i) = self.parse_closing_tag(i)?;

            // TODO: improve that
            let content_whitespace = &part1[part1.len() - 1..];
            let (closing_whitespace, _) = self.parse_whitespace(content_whitespace)?;
            let content = if closing_whitespace.is_none() {
                part1
            } else {
                &part1[..part1.len() - 1]
            };

            let command = Command {
                r#type: cmd_type,
                opening_whitespace,
                closing_whitespace,
                content,
            };
            tokens.push(Token::Command(command));

            input = i;
        }
        if !input.is_empty() {
            tokens.push(Token::Text(input));
        }

        Ok(tokens)
    }

    pub fn generate_js(&self, tokens: Vec<Token>) -> String {
        const REPLACEMENT_STR: &str = "rJ2KqXzxQg";

        // TODO: Replace this ugly hack with an array that we would await at the end and .join()
        // The problem is that this is a breaking change since '+=' doesn't work on arrays, we need
        // .push()

        let mut s = String::new();
        s += "let __prs = [];\n";
        write!(s, "let {} = '';\n", self.config.global_var).unwrap();

        let mut trim_left = &None;
        let mut prev_text = None;
        for token in tokens.iter() {
            match token {
                Token::Text(t) => {
                    prev_text = Some(t);
                }
                Token::Command(c) => {
                    if let Some(text) = prev_text {
                        let text = self.trim_whitespace(text, trim_left.as_ref(), true);
                        let text = self.trim_whitespace(text, c.opening_whitespace.as_ref(), false);
                        let text = self.escape_text(text);
                        write!(s, "{}+='{}';\n", self.config.global_var, text).unwrap();
                    }
                    trim_left = &c.closing_whitespace;
                    prev_text = None;

                    match c.r#type {
                        CommandType::Interpolate => {
                            write!(s, "__prs.push({});\n", c.content).unwrap();
                            write!(s, "{}+='{}';\n", self.config.global_var, REPLACEMENT_STR)
                                .unwrap();
                        }
                        CommandType::Execution => write!(s, "{};\n", c.content).unwrap(),
                    }
                }
            };
        }
        if let Some(text) = prev_text {
            let text = self.trim_whitespace(&text, trim_left.as_ref(), true);
            let text = self.escape_text(text);
            write!(s, "{}+='{}';\n", self.config.global_var, text).unwrap()
        }

        s += "const __rst = await Promise.all(__prs);\n";
        write!(
            s,
            "{} = {}.replace(/{}/g, () => __rst.shift());\n",
            self.config.global_var, self.config.global_var, REPLACEMENT_STR
        )
        .unwrap();
        write!(s, "return {};\n", self.config.global_var).unwrap();
        s
    }
}

#[wasm_bindgen]
pub struct Renderer {
    async_constructor: js_sys::Function,
    config: ParserConfig,
}

#[wasm_bindgen]
impl Renderer {
    #[wasm_bindgen(constructor)]
    pub fn new(config: ParserConfig) -> Self {
        let async_constructor =
            js_sys::Function::new_with_args("body, ", "return (async function(){}).constructor");
        let async_constructor = async_constructor.call0(&JsValue::NULL).unwrap();
        let async_constructor = js_sys::Function::from(async_constructor);
        Renderer {
            async_constructor,
            config,
        }
    }

    pub fn render_content(&self, content: &str, context: &JsValue) -> Result<JsValue> {
        let parser = Parser::new(content, &self.config);
        let tokens = parser.parse_tokens()?;
        let fn_body = parser.generate_js(tokens);
        let async_fn = match self
            .async_constructor
            .call2(
                &JsValue::NULL,
                &JsValue::from("tp"),
                &JsValue::from(&fn_body).into(),
            ) {
            Ok(f) => f,
            Err(_) => return Err(RenderError::SyntaxError),
        };
        let async_fn = js_sys::Function::from(async_fn);
        let res = match async_fn.call1(&JsValue::NULL, context) {
            Ok(r) => r,
            Err(_) => return Err(RenderError::FunctionError),
        };
        Ok(res)
    }
}
