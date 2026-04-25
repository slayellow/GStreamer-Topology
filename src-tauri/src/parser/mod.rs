mod normalizer;
mod pipeline;

pub use normalizer::normalize_text;
pub use pipeline::{parse_document, parse_graph};
