pub mod parser;
pub mod ast;
pub mod resolver;
pub mod emitter;
pub mod error;
pub mod cli;

pub use parser::parse;
pub use error::{CompileError, Result};
pub use resolver::resolve_names;
pub use emitter::emit_ir;
