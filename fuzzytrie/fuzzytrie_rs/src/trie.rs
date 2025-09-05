use crate::automaton::{LevenshteinAutomaton, LevenshteinAutomatonBuilder, LevenshteinDfaState};
use pyo3::prelude::*;
use std::collections::HashMap;

struct Node {
    is_word: bool,
    nodes: Vec<(char, Node)>,
}

#[pyclass(name = "Trie")]
pub struct Trie {
    automaton_builders: HashMap<u8, LevenshteinAutomatonBuilder>,
    nodes: Vec<(char, Node)>,
}

impl Node {
    fn new(is_word: bool) -> Self {
        Self {
            is_word: is_word,
            nodes: Vec::new(),
        }
    }
}

#[pymethods]
impl Trie {
    #[new]
    fn new() -> Self {
        Self {
            automaton_builders: HashMap::new(),
            nodes: Vec::new(),
        }
    }

    fn init_automaton(&mut self, d: u8) {
        self.automaton_builders
            .insert(d, LevenshteinAutomatonBuilder::new(d));
    }

    fn add(&mut self, word: String) {
        let mut nodes = &mut self.nodes;
        for (i, c) in word.chars().enumerate() {
            match nodes.binary_search_by(|t| t.0.cmp(&c)) {
                Ok(index) => {
                    nodes = &mut nodes[index].1.nodes;
                }
                Err(index) => {
                    let node = Node::new(i == word.len() - 1);
                    nodes.insert(index, (c, node));
                    nodes = &mut nodes[index].1.nodes;
                }
            }
        }
    }

    fn search(&self, d: u8, query: String) -> PyResult<Vec<String>> {
        match self.automaton_builders.get(&d) {
            Some(builder) => {
                let mut automaton = builder.get(query);
                let state = automaton.initial_state();
                let mut prefix = String::new();
                let mut matches = Vec::new();
                self._search(
                    &mut prefix,
                    &mut matches,
                    &self.nodes,
                    &state,
                    &mut automaton,
                );
                Ok(matches)
            }
            None => Ok(vec![]),
        }
    }
}

impl Trie {
    fn _search(
        &self,
        prefix: &mut String,
        matches: &mut Vec<String>,
        nodes: &Vec<(char, Node)>,
        state: &(u32, u32, u32),
        automaton: &mut LevenshteinAutomaton,
    ) {
        for (c, node) in nodes.iter() {
            let new_state = automaton.step(*c, &state);
            // println!(
            //     "{}, {:?}, {}, {}",
            //     c,
            //     new_state,
            //     automaton.can_match(&new_state),
            //     automaton.is_match(&new_state)
            // );

            if !automaton.can_match(&new_state) {
                continue;
            }

            prefix.push(*c);
            if node.is_word && automaton.is_match(&new_state) {
                matches.push(prefix.clone());
            }

            self._search(prefix, matches, &node.nodes, &new_state, automaton);
            prefix.pop();
        }
    }
}
