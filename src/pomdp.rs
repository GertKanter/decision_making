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

#[derive(Debug, Clone)]
pub struct ConditionalPlan { // tree of actions and observations
    pub action: String, // action to take at root
    pub subplans: Option<std::collections::BTreeMap<String, Box<ConditionalPlan>>>, // dictionary mapping observations to subplans
    pub alphavector: Vec<f64>, // alphavector for this plan
}
impl ConditionalPlan {
    pub fn set_configuration(&mut self, pomdp: &POMDP, configuration: &mut Vec<u32>) {
        let idx;
        match configuration.pop() {
            Some(x) => idx = x as usize,
            None => return,
        };
        self.action = pomdp.action_space[idx].to_string();
        if self.subplans == None {
        }
        else {
            for (_, value) in self.subplans.as_mut().unwrap().iter_mut() {
                value.set_configuration(pomdp, configuration);
            }
        }
    }
}
impl Default for ConditionalPlan {
    fn default() -> ConditionalPlan {
        ConditionalPlan { action: "".to_string(), subplans: None, alphavector: Vec::new() }
    }
}
impl PartialEq for ConditionalPlan {
    fn eq(&self, other: &Self) -> bool {
        if self.action == other.action {
            if self.subplans == other.subplans {
                return true;
            }
        }
        false
    }
}
pub struct ConditionalPlanIterator<'a> {
    stack: Vec<&'a ConditionalPlan>,
}
impl<'a> IntoIterator for &'a ConditionalPlan {
    type Item = &'a ConditionalPlan;
    type IntoIter = ConditionalPlanIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        let mut sortable: Vec<(u32, &'a ConditionalPlan)> = Vec::new();
        let mut vector: Vec<&'a ConditionalPlan> = Vec::new();
        let mut open_list: Vec<(u32, &'a ConditionalPlan)> = Vec::new();
        open_list.push((1, self));
        while open_list.len() > 0 { // breadth-first
            let target = open_list.remove(0);
            sortable.push((target.0, target.1));
            if target.1.subplans == None {
                continue;
            }
            let iter = target.1.subplans.as_ref().unwrap().iter();
            for (_, value) in iter {
                open_list.push((target.0 + 1, value));
            }
        }
        sortable.sort_by(|a, b| {
            if a.0 < b.0 {
                std::cmp::Ordering::Less
            }
            else if a.0 > b.0 {
                std::cmp::Ordering::Greater
            }
            else {
                a.1.action.cmp(&b.1.action)
            }
        });
        for item in sortable {
            vector.push(item.1);
        }
        
        ConditionalPlanIterator { stack: vector }
    }
}
impl<'a> Iterator for ConditionalPlanIterator<'a> {
    type Item = &'a ConditionalPlan;
    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.len() > 0 {
            return Some(self.stack.pop().unwrap());
        }
        None
    }
}


#[derive(Debug)]
pub struct LookaheadAlphaVectorPolicy {
    pub pomdp: POMDP,
    pub big_gamma: Vec<Vec<f64>>
}

pub fn find_dominating(big_gamma: &Vec<Vec<f64>>) -> Vec<bool> {
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

fn get_plan_permutations(plans: &Vec<ConditionalPlan>, observations: &Vec<String>, pomdp: &POMDP) -> Vec<std::collections::BTreeMap<String, Box<ConditionalPlan>>> {
    let mut returned_value: Vec<std::collections::BTreeMap<String, Box<ConditionalPlan>>> = Vec::new();
    let mut configuration: Vec<u32> = Vec::new();
    for _ in 0..(observations.len()) {
        configuration.push(0);
    }
    loop {
        let mut single_config: std::collections::BTreeMap<String, Box<ConditionalPlan>> = std::collections::BTreeMap::new();
        for (i, value) in configuration.iter().enumerate() {
             single_config.insert(observations[i].clone(), Box::new(plans[*value as usize].clone()));
        }
        returned_value.push(single_config);
        // increment at index
        configuration[0] += 1;
        // check if configuration correct
        if configuration[0] < pomdp.action_space.len() as u32 {
            // seems OK
        }
        else {
            // fix configuration by incrementing next one(s) and resetting previous one(s)
            let mut overflow = true;
            let mut overflow_idx = 0;
            while overflow {
                if configuration[overflow_idx] >= pomdp.action_space.len() as u32 {
                    // this one is overflowing
                    configuration[overflow_idx] = 0;
                    if overflow_idx < configuration.len() - 1 {
                        configuration[overflow_idx+1] += 1; 
                        if configuration[overflow_idx+1] >= pomdp.action_space.len() as u32 {
                            // we need to check more
                        }
                        else {
                            // think we solved it?
                            overflow = false;
                        }
                    }
                    else {
                        // finished?
                        return returned_value;
                    }
                }
                overflow_idx += 1;
            }
        }
    }
}

fn get_nodes_count(plan: &ConditionalPlan) -> u32 {
    if plan.subplans == None {
        return 1;
    }
    let mut total = 0;
    for (_, subplan) in plan.subplans.as_ref().unwrap() {
        total += get_nodes_count(subplan);
    }
    return total;
}

fn get_permutations(plan: &ConditionalPlan, pomdp: &POMDP) -> Vec<ConditionalPlan> { // all permutations of plan
    let mut returned_value: Vec<ConditionalPlan> = Vec::new();
    //find how many nodes in plan = this is how many nodes we need to permute
    let nodes = get_nodes_count(plan);
    let mut configuration: Vec<u32> = Vec::new();
    for _ in 0..(nodes * (pomdp.observation_space.len() as u32 - 1)) {
        configuration.push(0);
    }
    // we have the configuration, let's permute
    loop {
        // perform the work on this configuration as this is guaranteed to be correct
        let mut new_tree = plan.clone();
        let mut config_copy = configuration.clone();
        new_tree.set_configuration(pomdp, &mut config_copy);
        returned_value.push(new_tree);
        // operation done, now increment configuration
        // increment at index
        configuration[0] += 1;
        // check if configuration correct
        if configuration[0] < pomdp.action_space.len() as u32 {
            // seems OK
        }
        else {
            // fix configuration by incrementing next one(s) and resetting previous one(s)
            let mut overflow = true;
            let mut overflow_idx = 0;
            while overflow {
                if configuration[overflow_idx] >= pomdp.action_space.len() as u32 {
                    // this one is overflowing
                    configuration[overflow_idx] = 0;
                    if overflow_idx < configuration.len() - 1 {
                        configuration[overflow_idx+1] += 1; 
                        if configuration[overflow_idx+1] >= pomdp.action_space.len() as u32 {
                            // we need to check more
                        }
                        else {
                            // think we solved it?
                            overflow = false;
                        }
                    }
                    else {
                        // finished?
                        return returned_value;
                    }
                }
                overflow_idx += 1;
            }
        }
    }
}

fn expand_permute(plan: &ConditionalPlan, pomdp: &POMDP) -> Vec<ConditionalPlan> {
    let mut returned_plans: Vec<ConditionalPlan> = Vec::new();
    let permutations = get_permutations(plan, pomdp);
    for action in &pomdp.action_space {
        for observation in &pomdp.observation_space {
            let mut permutation_observations = pomdp.observation_space.clone();
            for (i, value) in permutation_observations.iter_mut().enumerate() {
                if value == observation {
                    permutation_observations.remove(i);
                    break;
                }
            }
            let plan_permutations = get_plan_permutations(&permutations, &permutation_observations, pomdp);
            for permutation in plan_permutations {
                let mut root = ConditionalPlan::default();
                root.action = action.to_string();
                let mut subplans = std::collections::BTreeMap::<String, Box<ConditionalPlan>>::new();
                subplans.insert(observation.to_string(), Box::new(plan.clone()));
                for (key, value) in permutation {
                    subplans.insert(key, value);
                }
                root.subplans = Some(subplans);
                returned_plans.push(root);
            }
        }
    }

    returned_plans
}

fn calculate_alphavector(plan: &mut ConditionalPlan, pomdp: &POMDP) -> Vec<f64> {
    let mut result: Vec<f64> = Vec::new();
    if plan.alphavector.len() == 0 {
        if plan.subplans == None {
            // we are at a leaf node
            for state in &pomdp.state_space {
                let reward = pomdp.reward_function.get(&(state.to_string(), plan.action.clone())).unwrap_or(&0.0);
                result.push(*reward);
            }
            plan.alphavector = result.clone();
            return result;
        }
        else {
            // get children's alphavectors
            let mut children_alphavector: HashMap<String, Vec<f64>> = HashMap::new(); // observation = alphavector
            for subplan in plan.subplans.as_mut().unwrap().iter_mut() {
                let alphavector = calculate_alphavector(subplan.1, pomdp);
                children_alphavector.insert(subplan.0.to_string(), alphavector);
            }
            // we got the children's values, we can now calculate the alphavector
            for state in &pomdp.state_space {
                let mut outer_sum: f64 = 0.0;
                for (i, state_prime) in pomdp.state_space.iter().enumerate() {
                    let mut inner_sum: f64 = 0.0;
                    for (observation, _) in plan.subplans.as_ref().unwrap() {
                        let mut children_alpha: f64 = 0.0;
                        let children = children_alphavector.get(observation);
                        match children {
                            Some(x) => children_alpha = x[i],
                            None => (),
                        }
                        let o = pomdp.observation_function.get(&(observation.to_string(), plan.action.clone(), state_prime.to_string())).unwrap_or(&0.0);
                        inner_sum += o * children_alpha;
                    }
                    let transition = pomdp.transition_function.get(&(state.to_string(), plan.action.clone()));
                    match transition {
                        Some(x) => {
                            let outer_value = x.get(state_prime).unwrap_or(&0.0);
                            outer_sum += outer_value * inner_sum;
                        },
                        None => (),
                    };
                }
                let local_reward = pomdp.reward_function.get(&(state.to_string(), plan.action.clone())).unwrap_or(&0.0);
                let state_result = local_reward + pomdp.gamma * outer_sum;
                result.push(state_result);
            }
        }
    }
    else {
        return plan.alphavector.clone();
    }

    result
}

fn expand(plans: &Vec<ConditionalPlan>, pomdp: &POMDP) -> (Vec<ConditionalPlan>, Vec<Vec<f64>>) {
    let mut returned_plans: Vec<ConditionalPlan> = Vec::new();
    let mut returned_gamma: Vec<Vec<f64>> = std::iter::repeat_with(|| Vec::<f64>::new()).take(0).collect();
    for plan in plans {
        let mut new_plans = expand_permute(plan, pomdp);
        for new_plan in new_plans.iter_mut() {
            let alphavector = calculate_alphavector(new_plan, pomdp);
            returned_gamma.push(alphavector);
        }
        for entry in new_plans {
            returned_plans.push(entry);
        }
    }
    (returned_plans, returned_gamma)
}

pub fn value_iteration(pomdp: &POMDP, k_max: u32) -> (Vec<ConditionalPlan>, Vec<Vec<f64>>) {
    let mut plans: Vec<ConditionalPlan> = Vec::new();
    let mut big_gamma: Vec<Vec<f64>> = std::iter::repeat_with(|| Vec::<f64>::new()).take(0).collect();
    for action in &pomdp.action_space {
        plans.push(ConditionalPlan { action: action.to_string(), subplans: None, alphavector: Vec::new() });
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
        result = expand(&result.0, pomdp);
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
    println!("{:?}", iteration.0);
    println!("optimal plan at depth {}: {:?}", depth, extract_policy(&iteration));
    LookaheadAlphaVectorPolicy { pomdp: pomdp, big_gamma: iteration.1 }
}
