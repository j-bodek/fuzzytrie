pub mod automaton;
pub mod trie;

use crate::trie::FuzzyTrie;
use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
fn fuzzytrie(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<FuzzyTrie>()?;
    Ok(())
}
