use crate::automaton::{LevenshteinAutomaton, LevenshteinAutomatonBuilder, LevenshteinDfaState};
use pyo3::prelude::*;
use std::collections::HashMap;

struct Node {
    is_word: bool,
    word: Option<String>,
    nodes: HashMap<char, Node>,
}

#[pyclass(name = "Trie")]
pub struct Trie {
    automaton_builders: HashMap<u8, LevenshteinAutomatonBuilder>,
    nodes: HashMap<char, Node>,
}

impl Node {
    fn new(is_word: bool, word: Option<String>) -> Self {
        Self {
            is_word: is_word,
            word: word,
            nodes: HashMap::new(),
        }
    }
}

#[pymethods]
impl Trie {
    #[new]
    fn new() -> Self {
        Self {
            automaton_builders: HashMap::new(),
            nodes: HashMap::new(),
        }
    }

    fn init_automaton(&mut self, d: u8) {
        self.automaton_builders
            .insert(d, LevenshteinAutomatonBuilder::new(d));
    }

    fn add(&mut self, word: String) {
        let mut nodes = &mut self.nodes;
        for (i, c) in word.chars().enumerate() {
            if !nodes.contains_key(&c) {
                nodes.insert(
                    c,
                    Node::new(
                        i == word.len() - 1,
                        if i == word.len() - 1 {
                            Some(word.clone())
                        } else {
                            None
                        },
                    ),
                );
            }
            nodes = &mut nodes.get_mut(&c).unwrap().nodes;
        }
    }

    fn search(&self, d: u8, query: String) -> PyResult<Vec<String>> {
        match self.automaton_builders.get(&d) {
            Some(builder) => {
                let mut automaton = builder.get(query);
                let state = automaton.initial_state();
                Ok(self._search(&self.nodes, &state, &mut automaton))
            }
            None => Ok(vec![]),
        }
    }
}

impl Trie {
    fn _search(
        &self,
        nodes: &HashMap<char, Node>,
        state: &LevenshteinDfaState,
        automaton: &mut LevenshteinAutomaton,
    ) -> Vec<String> {
        let mut matches = vec![];

        for (c, node) in nodes.iter() {
            let new_state = automaton.step(*c, &state);
            if !automaton.can_match(&new_state) {
                continue;
            }

            if node.is_word && automaton.is_match(&new_state) {
                matches.push(node.word.as_ref().unwrap().clone());
            }

            for w in self._search(&node.nodes, &new_state, automaton).into_iter() {
                matches.push(w);
            }
        }

        matches
    }
}
