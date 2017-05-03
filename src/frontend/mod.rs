use ::ir::compilation_unit::CompilationUnit;
use ::errors::*;

pub type Frontend = fn(&str) -> Result<CompilationUnit>;

pub mod protocol_json;
pub mod protocol_spec;
