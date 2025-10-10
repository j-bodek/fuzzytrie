use crate::automaton::{LevenshteinAutomaton, LevenshteinAutomatonBuilder, LevenshteinDfaState};
use pyo3::prelude::*;
use std::collections::HashMap;

struct Node {
    is_word: bool,
    nodes: Vec<(char, Node)>,
}

#[pyclass(name = "FuzzyTrie")]
pub struct FuzzyTrie {
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
impl FuzzyTrie {
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
        let len = word.chars().count();

        for (i, c) in word.chars().enumerate() {
            match nodes.binary_search_by(|t| t.0.cmp(&c)) {
                Ok(index) => {
                    if i == len - 1 {
                        nodes[index].1.is_word = true;
                    }
                    nodes = &mut nodes[index].1.nodes;
                }
                Err(index) => {
                    let node = Node::new(i == len - 1);
                    nodes.insert(index, (c, node));
                    nodes = &mut nodes[index].1.nodes;
                }
            }
        }
    }

    fn delete(&mut self, word: String) {
        let mut chars: Vec<char> = word.chars().rev().collect();
        Self::_delete(&mut chars, &mut self.nodes);
    }

    fn search(&self, d: u8, query: String) -> PyResult<Vec<(u16, String)>> {
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

impl FuzzyTrie {
    fn _delete(chars: &mut Vec<char>, nodes: &mut Vec<(char, Node)>) -> (bool, bool) {
        if chars.len() == 0 {
            return (true, true);
        }

        if let Ok(index) = nodes.binary_search_by(|t| t.0.cmp(&chars[chars.len() - 1])) {
            chars.pop();

            if chars.len() == 0 {
                nodes[index].1.is_word = false;
                return (nodes[index].1.nodes.len() == 0, true);
            } else {
                let (can_remove, deleted) = Self::_delete(chars, &mut nodes[index].1.nodes);
                if can_remove && nodes[index].1.nodes.len() == 1 {
                    nodes[index].1.nodes.pop();
                    return (true, deleted);
                }
                return (false, deleted);
            }
        }

        return (false, false);
    }

    fn _search(
        &self,
        prefix: &mut String,
        matches: &mut Vec<(u16, String)>,
        nodes: &Vec<(char, Node)>,
        state: &LevenshteinDfaState,
        automaton: &mut LevenshteinAutomaton,
    ) {
        for (c, node) in nodes.iter() {
            let new_state = automaton.step(*c, &state);
            if !automaton.can_match(&new_state) {
                continue;
            }

            prefix.push(*c);
            if node.is_word && automaton.is_match(&new_state) {
                matches.push((automaton.distance(&new_state), prefix.clone()));
            }

            self._search(prefix, matches, &node.nodes, &new_state, automaton);
            prefix.pop();
        }
    }
}
