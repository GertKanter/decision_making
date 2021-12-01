use std::collections::HashMap;
use good_lp::{default_solver, Solution, SolverModel, variable, variables, Expression};

#[derive(Debug)]
pub struct POMDP {
    pub gamma: f64,
    pub state_space: Vec<String>,
    pub action_space : Vec<String>,
    pub observation_space : Vec<String>,
    pub transition_function: HashMap<(String, String), HashMap<String, f64>>, // state, action = {state' = probability} of action leading from state to state'
    pub reward_function: HashMap<(String, String), f64>, // state, action = reward for performing action in state
    pub observation_function: HashMap<(String, String, String), f64> // observation, action, state' = probability of observing this observation in this state (e.g., O(crying | ignore, hungry))
}

impl Default for POMDP {
    fn default() -> POMDP {
        let states = Vec::<String>::new();
        let actions = Vec::<String>::new();
        let observation_space = Vec::<String>::new();
        let transitions = HashMap::<(String, String), HashMap<String, f64>>::new();
        let rewards = HashMap::<(String, String), f64>::new();
        let observations = HashMap::<(String, String, String), f64>::new();
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

#[derive(Debug, Clone)]
pub struct ConditionalPlan { // tree of actions and observations
    pub action: String, // action to take at root
    pub subplans: HashMap<String, ConditionalPlan> // dictionary mapping observations to subplans
}
impl Default for ConditionalPlan {
    fn default() -> ConditionalPlan {
        ConditionalPlan { action: "".to_string(), subplans: HashMap::<String, ConditionalPlan>::new() }
    }
}

#[derive(Debug)]
pub struct LookaheadAlphaVectorPolicy {
    pub pomdp: POMDP,
    pub big_gamma: Vec<Vec<f64>>
}

pub fn find_dominating(big_gamma: &Vec<Vec<f64>>) -> Vec<bool> { // returns (plans, big_gamma)
    let n = big_gamma.len();
    let mut candidates: Vec<bool> = Vec::new();
    let mut dominating: Vec<bool> = Vec::new();
    for _ in 0..n {
        candidates.push(true);
        dominating.push(false);
    }
    while candidates.iter_mut().any(|x| *x) {
        let index = candidates.iter().position(|x| *x).unwrap();
        let mut dominated: Vec<Vec<f64>> = std::iter::repeat_with(|| Vec::<f64>::new()).take(0).collect();
        for i in 0..n {
            if dominating[i] {
                dominated.push(big_gamma[i].clone()); // slow??
            }
        }
        let b = find_maximal_belief(&big_gamma[index], &dominated);
        if b.len() == 0 {
            candidates[index] = false;
        }
        else {
            // we need to figure out the argmax now
            let mut k = 0;
            let mut max = std::f64::MIN;
            for i in 0..n {
                if candidates[i] {
                    let mut value = 0.0;
                    for j in 0..big_gamma[i].len() {
                        value += b[j] * big_gamma[i][j];
                    }
                    if value > max {
                        max = value;
                        k = i;
                    }
                }
            }
            if max > std::f64::MIN {
                candidates[k] = false;
                dominating[k] = true;
            }
        }
    }
    dominating
}

pub fn prune(plans: &Vec<ConditionalPlan>, big_gamma: &Vec<Vec<f64>>) -> (Vec<ConditionalPlan>, Vec<Vec<f64>>) {
    let d = find_dominating(&big_gamma);
    let mut returned_plans: Vec<ConditionalPlan> = Vec::new();
    let mut returned_gamma: Vec<Vec<f64>> = std::iter::repeat_with(|| Vec::<f64>::new()).take(0).collect();
    for i in 0..big_gamma.len() {
        if d[i] {
            returned_plans.push(plans[i].clone());
            returned_gamma.push(big_gamma[i].clone());
        }
    }
    (returned_plans, returned_gamma)
}

fn combine_lookahead(pomdp: &POMDP, state: &String, action: &String, big_gamma: &Vec<Vec<f64>>) -> f64 {
    let mut outer_sum = 0.0;
    let transition = pomdp.transition_function.get(&(state.to_string(), action.to_string()));
    for (i, state_prime) in pomdp.state_space.iter().enumerate() {
        let mut sum = 0.0;
        for (o, alpha) in pomdp.observation_space.iter().zip(big_gamma) {
            sum += pomdp.observation_function.get(&(o.to_string(), action.to_string(), state_prime.to_string())).unwrap_or(&0.0) * alpha[i];
        }
        match transition {
            Some(x) => {
                let value = x.get(state_prime).unwrap_or(&0.0);
                outer_sum += value * sum;
                ()
            },
            None => (),
        }
    }
    pomdp.reward_function.get(&(state.to_string(), action.to_string())).unwrap_or(&0.0) + pomdp.gamma * outer_sum
}

fn combine_alphavector(pomdp: &POMDP, action: &String, big_gamma: &Vec<Vec<f64>>) -> Vec<f64> {
    let mut return_value: Vec<f64> = Vec::new();
    for state in &pomdp.state_space {
        return_value.push(combine_lookahead(pomdp, &state.to_string(), &action.to_string(), big_gamma));
    }
    return_value
}

pub fn expand(plans: &Vec<ConditionalPlan>, big_gamma: &Vec<Vec<f64>>, pomdp: &POMDP) -> (Vec<ConditionalPlan>, Vec<Vec<f64>>) {
    let mut returned_plans: Vec<ConditionalPlan> = Vec::new();
    let mut returned_gamma: Vec<Vec<f64>> = std::iter::repeat_with(|| Vec::<f64>::new()).take(0).collect();
    for action in &pomdp.action_space {
        for (_, plan) in plans.iter().enumerate() {
            let mut subplans: HashMap<String, ConditionalPlan> = HashMap::new();
            for o in &pomdp.observation_space {
                subplans.insert(o.to_string(), (*plan).clone());
            }
            let condplan = ConditionalPlan { action: action.to_string(), subplans: subplans };
            returned_plans.push(condplan);
        }
        let combalpha = combine_alphavector(pomdp, action, big_gamma);
        returned_gamma.push(combalpha);
    }
    (returned_plans, returned_gamma)
}

pub fn value_iteration(pomdp: &POMDP, k_max: u32) -> (Vec<ConditionalPlan>, Vec<Vec<f64>>) {
    let mut plans: Vec<ConditionalPlan> = Vec::new();
    let mut big_gamma: Vec<Vec<f64>> = std::iter::repeat_with(|| Vec::<f64>::new()).take(0).collect();
    for action in &pomdp.action_space {
        plans.push(ConditionalPlan { action: action.to_string(), subplans: HashMap::<String, ConditionalPlan>::new() });
        let mut state_set: Vec<f64> = Vec::new();
        for state in &pomdp.state_space {
            if pomdp.reward_function.contains_key(&(state.to_string(), action.to_string())) {
                state_set.push(*pomdp.reward_function.get(&(state.to_string(), action.to_string())).unwrap());
            }
            else {
                state_set.push(0.0);
            }
        }
        big_gamma.push(state_set);
    }
    let mut result = prune(&plans, &big_gamma);
    for _ in 1..k_max {
        result = expand(&result.0, &result.1, pomdp);
        println!("EXPAND = {:?}", result);
        if result.0.len() > 0 {
            result = prune(&result.0, &result.1);
        }
    }
    result
}

pub fn find_maximal_belief(alpha: &Vec<f64>, big_gamma: &Vec<Vec<f64>>) -> Vec<f64> {
    let mut return_value: Vec<f64> = Vec::new();
    if big_gamma.len() == 0 {
        for _ in 0..alpha.len() {
            return_value.push(1.0 / alpha.len() as f64);
        }
    }
    else {
        let mut problem_variables = variables!();
        let delta = problem_variables.add(variable());
        let mut vars: Vec<variable::Variable> = Vec::new();
        let mut sum = Expression::default();
        for i in 0..alpha.len() {
            let var = problem_variables.add(variable().min(0));
            vars.push(var);
            if i == 0 {
                sum = Expression::from(vars[0]);
            }
            if i >= 1 {
                sum += Expression::from(vars[i]);
            }
        }
        let mut solution = problem_variables.maximise(delta).using(default_solver);
        for a in big_gamma {
            let mut constraint = Expression::default();
            for i in 0..alpha.len() {
                let coeff = alpha[i] - a[i];
                if i == 0 {
                    constraint = Expression::from(coeff*vars[i]);
                }
                else {
                    constraint += Expression::from(coeff*vars[i]);
                }
            }
            let geq = constraint.clone().geq(delta);
            solution.add_constraint(geq);
        }
        let constraint = sum.eq(Expression::from(1.0));
        solution.add_constraint(constraint);
       let result = solution.solve();
        match result {
            Ok(x) => {
                if x.value(delta) > 0.0 {
                    for i in 0..vars.len() {
                        return_value.push(x.value(vars[i]));
                    }
                }
            },
            Err(x) => {
                println!("Solver gave error: {:?}", x);
            },
        }
    }
    return_value
}

fn extract_policy(iteration: &(Vec<ConditionalPlan>, Vec<Vec<f64>>)) -> HashMap<String, Vec<f64>> {
    let mut result = HashMap::<String, Vec<f64>>::new();
    for (i, plan) in iteration.0.iter().enumerate() {
        result.insert(plan.action.clone(), iteration.1[i].clone());
    }
    result
}

pub fn solve(depth: u32, pomdp: POMDP) -> LookaheadAlphaVectorPolicy {
    let iteration = value_iteration(&pomdp, depth);
    println!("optimal plan at depth {}: {:?}", depth, extract_policy(&iteration));
    LookaheadAlphaVectorPolicy { pomdp: pomdp, big_gamma: iteration.1 }
}
