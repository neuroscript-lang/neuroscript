//! NeuroScript - Neural Architecture Composition Language
//!
//! A language for defining composable neural architectures.
//!
//! # Example
//!
//! ```neuroscript
//! neuron MLP(dim):
//!   in: [*, dim]
//!   out: [*, dim]
//!   graph:
//!     in ->
//!       Linear(dim, dim * 4)
//!       GELU()
//!       Linear(dim * 4, dim)
//!       out
//! ```

pub mod ir;
pub mod lexer;
pub mod parser;

pub use ir::*;
pub use parser::Parser;

/// Parse a NeuroScript source string into a Program.
pub fn parse(source: &str) -> Result<Program, parser::ParseError> {
    Parser::parse(source)
}
