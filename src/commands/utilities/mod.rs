use crate::types::FnCommands;

mod compile;
mod ping;
mod translate;

pub fn commands() -> FnCommands {
    vec![
        ping::ping,
        translate::translate_ctx_menu,
        translate::translate,
        compile::compile,
    ]
}
