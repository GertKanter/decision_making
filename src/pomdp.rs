use std::collections::HashMap;

#[derive(Debug)]
pub struct POMDP {
    pub gamma: f64,
    pub state_space: Vec<String>,
    pub action_space : Vec<String>,
    pub observation_space : Vec<String>,
    pub transition_function: HashMap<(String, String), HashMap<String, f64>>, // state, action = {state' = probability} of action leading from state to state'
    pub reward_function: HashMap<(String, String), f64>, // state, action = reward for performing action in state
    pub observation_function: HashMap<(String, String), HashMap<String, f64>> // action, state' = {observation = probability} of observing this observation in this state (e.g., O(crying | ignore, hungry))
}

impl Default for POMDP {
    fn default() -> POMDP {
        let states = Vec::<String>::new();
        let actions = Vec::<String>::new();
        let observation_space = Vec::<String>::new();
        let transitions = HashMap::<(String, String), HashMap<String, f64>>::new();
        let rewards = HashMap::<(String, String), f64>::new();
        let observations = HashMap::<(String, String), HashMap<String, f64>>::new();
        POMDP { gamma: 1.0, state_space: states, action_space: actions, observation_space: observation_space, transition_function: transitions, reward_function: rewards, observation_function: observations }
    }
}

impl POMDP {
    #[allow(dead_code)]
    pub fn lookahead(&self, values: &HashMap<String, f64>, state: &String, action: &String) -> f64 {
        let mut sum: f64 = 0.0;
        let entry = self.transition_function.get(&(state.to_string(), action.to_string()));
        match entry {
            Some(x) => {
                for (key, value) in x {
                    sum += value * values[key];
                }
            ()
            },
            None => (),
        }
        *self.reward_function.get(&(state.to_string(), action.to_string())).unwrap_or(&0.0) + self.gamma * sum
    }
}


