pub mod automaton;
pub mod trie;

use crate::automaton::LevenshteinAutomatonBuilder;
use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
fn fuzzytrie_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<LevenshteinAutomatonBuilder>()?;
    Ok(())
}
