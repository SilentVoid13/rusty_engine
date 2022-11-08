pub fn count_newlines(s: &str) -> usize {
    s.as_bytes().iter().filter(|&c| *c == b'\n').count()
}

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

pub(crate) use log;
