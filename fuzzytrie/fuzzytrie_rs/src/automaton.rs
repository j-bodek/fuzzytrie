use nohash_hasher::BuildNoHashHasher;
use std::collections::HashMap;
use std::sync::Arc;
use std::vec::Vec;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
struct State(u32, i32);

pub struct LevenshteinDfaState {
    offset: u32,
    max_shift: u32,
    states: Vec<State>,
}

struct LevenshteinDfa {
    dfa:
        HashMap<u32, HashMap<u16, (u32, u32, u32), BuildNoHashHasher<u16>>, BuildNoHashHasher<u16>>,
}

pub struct LevenshteinAutomaton {
    query: String,
    d: u8,
    dfa: Arc<LevenshteinDfa>,
    empty_vector: Vec<u8>,
    characteristic_vector_cache: HashMap<char, Vec<u8>>,
}

pub struct LevenshteinAutomatonBuilder {
    d: u8,
    dfa: Arc<LevenshteinDfa>,
}

impl LevenshteinDfa {
    fn new(d: u8) -> Self {
        let mut dfa: HashMap<
            u32,
            HashMap<u16, (u32, u32, u32), BuildNoHashHasher<u16>>,
            BuildNoHashHasher<u16>,
        > = HashMap::default();

        let state = Self::initial_state(d);
        let char_vectors = Self::get_characteristic_vectors(2 * d + 1);

        let mut states_ids: HashMap<Vec<State>, u32> = HashMap::new();
        dfa.insert(
            Self::get_states_id(&state.states, &mut states_ids),
            HashMap::default(),
        );
        let mut states_stack = vec![state.states];

        while states_stack.len() > 0 {
            let states = states_stack.pop().unwrap();
            let mut transitions: HashMap<u16, (u32, u32, u32), BuildNoHashHasher<u16>> =
                HashMap::default();
            for vec in char_vectors.iter() {
                let next_state = Self::normalize(Self::step(vec, &states));
                let next_state_id = Self::get_states_id(&next_state.states, &mut states_ids);
                if !dfa.contains_key(&next_state_id) {
                    dfa.insert(next_state_id, HashMap::default());
                    states_stack.push(next_state.states.clone());
                }

                transitions.insert(
                    Self::vec_to_mask(vec),
                    (next_state.offset, next_state.max_shift, next_state_id),
                );
            }

            dfa.insert(Self::get_states_id(&states, &mut states_ids), transitions);
        }

        Self { dfa: dfa }
    }

    fn get_states_id(states: &Vec<State>, states_ids: &mut HashMap<Vec<State>, u32>) -> u32 {
        if states.len() == 0 {
            return 0;
        }

        match states_ids.get(states) {
            Some(id) => *id,
            None => {
                let id = (states_ids.len() + 1) as u32;
                states_ids.insert(states.clone(), id);
                id
            }
        }
    }

    fn get_characteristic_vectors(width: u8) -> Vec<Vec<u8>> {
        fn create(vectors: Vec<Vec<u8>>, depth: u8, max: u8) -> Vec<Vec<u8>> {
            if depth == max {
                return vectors;
            }

            let mut new_vectors: Vec<Vec<u8>> = Vec::new();
            for v in vectors.into_iter() {
                new_vectors.push(v.clone().into_iter().chain(vec![1]).collect());
                new_vectors.push(v.into_iter().chain(vec![0]).collect());
            }

            create(new_vectors, depth + 1, max)
        }

        let vectors = vec![vec![1], vec![0]];
        create(vectors, 1, width)
    }

    fn transitions(vector: &Vec<u8>, state: &State) -> Vec<State> {
        match &vector[state.0 as usize..vector.len()]
            .iter()
            .position(|x| *x == 1)
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

    fn step(vector: &Vec<u8>, states: &Vec<State>) -> Vec<State> {
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

    fn vec_to_mask(vec: &[u8]) -> u16 {
        // builds bitmask from binary vector and returns u16

        let mut mask = 0u16;
        for (i, &b) in vec.iter().enumerate() {
            if b != 0 {
                mask |= 1 << i;
            }
        }

        mask
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
        let mut cache = HashMap::new();
        for c in query.chars() {
            let mut char_vec: Vec<u8> = query
                .chars()
                .map(|ch| if ch == c { 1 } else { 0 })
                .collect();
            // create bitmask for vectors
            char_vec.append(&mut vec![0; 2 * d as usize + 1]);

            cache.insert(c, char_vec);
        }
        Self {
            query: query,
            d: d,
            dfa: dfa,
            empty_vector: vec![0; d as usize * 2 + 1],
            characteristic_vector_cache: cache,
        }
    }

    pub fn initial_state(&self) -> (u32, u32, u32) {
        (0, self.d as u32, 1)
        // LevenshteinDfa::initial_state(self.d)
    }

    pub fn step(&mut self, c: char, state: &(u32, u32, u32)) -> (u32, u32, u32) {
        // self.create_characteristic_vector(c);
        let vec = self.get_characteristic_vector(c, state.0);

        match self.dfa.as_ref().dfa.get(&state.2) {
            Some(transitions) => match transitions.get(&LevenshteinDfa::vec_to_mask(vec)) {
                Some(next_state) => (state.0 + next_state.0, next_state.1, next_state.2),
                None => (0, 0, 0),
            },
            _ => (0, 0, 0),
        }
    }

    pub fn is_match(&self, state: &(u32, u32, u32)) -> bool {
        // self.query.len() as i32 - state.offset as i32 <= state.max_shift as i32
        self.query.len() as i32 - state.0 as i32 <= state.1 as i32
    }

    pub fn can_match(&self, state: &(u32, u32, u32)) -> bool {
        // state.states.len() > 0
        state.2 != 0
    }

    // fn create_characteristic_vector(&mut self, c: char) {
    //     if !self.characteristic_vector_cache.contains_key(&c) && self.query.contains(c) {
    //         let mut char_vec: Vec<bool> = self.query.chars().map(|ch| ch == c).collect();
    //         char_vec.append(&mut vec![false; 2 * self.d as usize + 1]);

    //         self.characteristic_vector_cache.insert(c, char_vec);
    //     };
    // }

    fn get_characteristic_vector(&self, c: char, offset: u32) -> &[u8] {
        match self.characteristic_vector_cache.get(&c) {
            Some(vec) => return &vec[offset as usize..(offset + 2 * self.d as u32 + 1) as usize],
            None => return &self.empty_vector[..],
        }
    }
}

impl LevenshteinAutomatonBuilder {
    pub fn new(d: u8) -> Self {
        Self {
            d: d,
            dfa: Arc::new(LevenshteinDfa::new(d)),
        }
    }

    pub fn get(&self, query: String) -> LevenshteinAutomaton {
        LevenshteinAutomaton::new(query, self.d, Arc::clone(&self.dfa))
    }
}
