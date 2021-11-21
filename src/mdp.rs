use std::collections::HashMap;

#[derive(Debug)]
pub struct MDP {
    pub gamma: f64,
    pub s: Vec<String>,
    pub a : Vec<String>,
    pub t: HashMap<(String, String), HashMap<String, f64>>, // state, action = {state' = probability} of action leading from state to state'
    pub r: HashMap<(String, String), f64> // state, action = reward for performing action in state
}

impl Default for MDP {
    fn default() -> MDP {
        let states = Vec::<String>::new();
        let actions = Vec::<String>::new();
        let transitions = HashMap::<(String, String), HashMap<String, f64>>::new();
        let rewards = HashMap::<(String, String), f64>::new();
        MDP { gamma: 1.0, s: states, a: actions, t: transitions, r: rewards}
    }
}

impl MDP {
    pub fn lookahead(&self, values: &HashMap<String, f64>, state: &String, action: &String) -> f64 {
        let mut sum: f64 = 0.0;
        let entry = self.t.get(&(state.to_string(), action.to_string()));
        match entry {
            Some(x) => {
                for (key, value) in x {
                    sum += value * values[key];
                }
            ()
            },
            None => (),
        }
        *self.r.get(&(state.to_string(), action.to_string())).unwrap_or(&0.0) + self.gamma * sum
    }
}


