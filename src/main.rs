use std::collections::HashMap;

#[derive(Debug)]
struct POMDP {
    gamma: f64,
    s: Vec<&'static str>,
    a: Vec<&'static str>,
    t: HashMap<(&'static str, &'static str, &'static str), f64>, // state, action, state' = probability of action leading from state to state'
    r: HashMap<(&'static str, &'static str), f64> // state, action = reward for performing action in state
}

impl POMDP {
    fn lookahead(&mut self, value: &Vec<f64>, state: &str, action: &str) -> f64 {
        let mut sum: f64 = 0.0;
        for i in 0..value.len() {
            //println!("i = {}", i);
            sum += *self.t.get(&(state, action, self.s[i])).unwrap_or(&0.0) * value[i];
        }
        *self.r.get(&(state, action)).unwrap_or(&0.0) + self.gamma * sum
    }
}

fn iterative_policy_evaluation(pomdp : &mut POMDP, pi: &str, k_max: i64) -> Vec<f64> {
    let mut u: Vec<f64> = Vec::new();
    for i in 0..pomdp.s.len() {
        u.push(0.0);
    }
    for i in 0..k_max {
        println!("STEP {}", i);
        for j in 0..u.len() {
            u[j] = pomdp.lookahead(&u, pomdp.s[j], pi);
        }
        println!("U {:?}", u);
    }
    u
}


fn main() {
    let mut transitions: HashMap<(&str, &str, &str), f64> = HashMap::new();
    transitions.insert(("state1", "move", "state2"), 1.0);
    let mut rewards: HashMap<(&str, &str), f64> = HashMap::new();
    rewards.insert(("state1", "move"), -1.0);
    let mut pomdp = POMDP { gamma: 0.9, s: Vec::from(["state1", "state2"]), a: Vec::from(["move"]), t: transitions, r: rewards};
    println!("{:?}", pomdp);
    println!("iteraive_policy_evaluation value {:?}", iterative_policy_evaluation(&mut pomdp, "move", 2));
}

