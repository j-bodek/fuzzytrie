use pyo3::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::vec::Vec;

#[derive(Debug, Ord, PartialOrd, Clone, Hash)]
struct State(u32, i32);

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl Eq for State {}

#[pyclass(name = "LevenshteinDfaState")]
struct LevenshteinDfaState {
    offset: u32,
    max_shift: u32,
    states: Vec<State>,
}

#[pymethods]
impl LevenshteinDfaState {
    fn str(&self) -> String {
        format!("({}, {:?})", self.offset, self.states)
    }
}

struct LevenshteinDfa {
    dfa: HashMap<Vec<State>, HashMap<Vec<bool>, LevenshteinDfaState>>,
}

#[pyclass(name = "LevenshteinAutomaton")]
struct LevenshteinAutomaton {
    query: String,
    d: u8,
    dfa: Arc<LevenshteinDfa>,
    empty_vector: Vec<bool>,
    characteristic_vector_cache: HashMap<char, Vec<bool>>,
}

#[pyclass(name = "LevenshteinAutomatonBuilder")]
pub struct LevenshteinAutomatonBuilder {
    d: u8,
    dfa: Arc<LevenshteinDfa>,
}

impl LevenshteinDfa {
    fn new(d: u8) -> Self {
        let mut dfa: HashMap<Vec<State>, HashMap<Vec<bool>, LevenshteinDfaState>> = HashMap::new();

        let state = Self::initial_state(d);
        let char_vectors = Self::get_characteristic_vectors(2 * d + 1);

        dfa.insert(state.states.clone(), HashMap::new());
        let mut states_stack = vec![state.states];

        while states_stack.len() > 0 {
            let states = states_stack.pop().unwrap();
            let mut transitions: HashMap<Vec<bool>, LevenshteinDfaState> = HashMap::new();
            for vec in char_vectors.iter() {
                let next_state = Self::normalize(Self::step(vec, &states));
                if !dfa.contains_key(&next_state.states) {
                    dfa.insert(next_state.states.clone(), HashMap::new());
                    states_stack.push(next_state.states.clone());
                }

                transitions.insert(vec.clone(), next_state);
            }

            dfa.insert(states, transitions);
        }

        Self { dfa: dfa }
    }

    fn get_characteristic_vectors(width: u8) -> Vec<Vec<bool>> {
        fn create(vectors: Vec<Vec<bool>>, depth: u8, max: u8) -> Vec<Vec<bool>> {
            if depth == max {
                return vectors;
            }

            let mut new_vectors: Vec<Vec<bool>> = Vec::new();
            for v in vectors.into_iter() {
                new_vectors.push(v.clone().into_iter().chain(vec![true]).collect());
                new_vectors.push(v.into_iter().chain(vec![false]).collect());
            }

            create(new_vectors, depth + 1, max)
        }

        let vectors = vec![vec![true], vec![false]];
        create(vectors, 1, width)
    }

    fn transitions(vector: &Vec<bool>, state: &State) -> Vec<State> {
        match &vector[state.0 as usize..vector.len()]
            .iter()
            .position(|x| *x == true)
        {
            Some(index) => {
                if *index as u32 == 0 {
                    return vec![State(state.0 + 1, state.1)];
                } else {
                    return vec![
                        State(state.0, state.1 - 1),
                        State(state.0 + 1, state.1 - 1),
                        State(state.0 + *index as u32 + 1, state.1 - *index as i32),
                    ];
                }
            }
            None => return vec![State(state.0, state.1 - 1), State(state.0 + 1, state.1 - 1)],
        }
    }

    fn step(vector: &Vec<bool>, states: &Vec<State>) -> Vec<State> {
        let mut next_states: Vec<State> = Vec::new();

        for s in states.iter() {
            let mut transitions: Vec<State> = Self::transitions(&vector, &s);
            while transitions.len() > 0 {
                let state = transitions.pop().unwrap();
                if state.1 >= 0 && !next_states.contains(&state) {
                    next_states.push(state);
                }
            }
        }

        next_states
    }

    fn initial_state(d: u8) -> LevenshteinDfaState {
        Self::normalize(vec![State(0, d as i32)])
    }

    fn normalize(states: Vec<State>) -> LevenshteinDfaState {
        if states.len() == 0 {
            return LevenshteinDfaState {
                offset: 0,
                max_shift: 0,
                states: Vec::new(),
            };
        }

        let min_offset =
            states.iter().fold(
                states[0].0,
                |offset, state| if offset < state.0 { offset } else { state.0 },
            );

        let mut states: Vec<State> = states
            .iter()
            .map(|s| State(s.0 - min_offset, s.1))
            .collect();

        states.sort_by(|s1, s2| s1.cmp(&s2));

        let max_shift = states
            .iter()
            .fold(states[0].0 as i32 + states[0].1, |offset, state| {
                if offset > state.0 as i32 + state.1 {
                    offset
                } else {
                    state.0 as i32 + state.1
                }
            });

        LevenshteinDfaState {
            offset: min_offset,
            max_shift: max_shift as u32,
            states: states,
        }
    }
}

impl LevenshteinAutomaton {
    fn new(query: String, d: u8, dfa: Arc<LevenshteinDfa>) -> Self {
        Self {
            query: query,
            d: d,
            dfa: dfa,
            empty_vector: vec![false; d as usize * 2 + 1],
            characteristic_vector_cache: HashMap::new(),
        }
    }

    fn create_characteristic_vector(&mut self, c: char) {
        if self.query.contains(c) && !self.characteristic_vector_cache.contains_key(&c) {
            let mut char_vec: Vec<bool> = self.query.chars().map(|ch| ch == c).collect();
            char_vec.append(&mut vec![false; 2 * self.d as usize + 1]);

            self.characteristic_vector_cache.insert(c, char_vec);
        };
    }

    fn get_characteristic_vector(&self, c: char, offset: u32) -> &[bool] {
        match self.characteristic_vector_cache.get(&c) {
            Some(vec) => return &vec[offset as usize..(offset + 2 * self.d as u32 + 1) as usize],
            None => return &self.empty_vector[..],
        }
    }
}

#[pymethods]
impl LevenshteinAutomaton {
    fn initial_state(&self) -> PyResult<LevenshteinDfaState> {
        Ok(LevenshteinDfa::initial_state(self.d))
    }

    fn step(&mut self, c: char, state: &LevenshteinDfaState) -> PyResult<LevenshteinDfaState> {
        self.create_characteristic_vector(c);
        let vec = self.get_characteristic_vector(c, state.offset);

        match self.dfa.as_ref().dfa.get(&state.states) {
            Some(transitions) => match transitions.get(vec) {
                Some(next_state) => Ok(LevenshteinDfaState {
                    offset: state.offset + next_state.offset,
                    max_shift: next_state.max_shift,
                    states: next_state.states.clone(),
                }),
                None => Ok(LevenshteinDfaState {
                    offset: 0,
                    max_shift: 0,
                    states: vec![],
                }),
            },
            _ => Ok(LevenshteinDfaState {
                offset: 0,
                max_shift: 0,
                states: vec![],
            }),
        }
    }

    fn is_match(&self, state: &LevenshteinDfaState) -> PyResult<bool> {
        return Ok(self.query.len() as i32 - state.offset as i32 <= state.max_shift as i32);
    }

    fn can_match(&self, state: &LevenshteinDfaState) -> PyResult<bool> {
        Ok(state.states.len() > 0)
    }
}

#[pymethods]
impl LevenshteinAutomatonBuilder {
    #[new]
    fn new(d: u8) -> Self {
        Self {
            d: d,
            dfa: Arc::new(LevenshteinDfa::new(d)),
        }
    }

    fn get(&self, query: String) -> PyResult<LevenshteinAutomaton> {
        Ok(LevenshteinAutomaton::new(
            query,
            self.d,
            Arc::clone(&self.dfa),
        ))
    }
}
