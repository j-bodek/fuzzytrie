pub mod automaton;
pub mod trie;

use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
fn fuzzytrie_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
