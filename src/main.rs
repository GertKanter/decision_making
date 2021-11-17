mod pomdp;
mod hex_world;

fn findmax<F: Fn(String) -> f64>(collection: &Vec<String>, function: F) -> (String, f64) {
    // try each item in collection as arg for function and record the best result
    let mut best = std::f64::MIN;
    let mut best_item = &collection[0];
    for item in collection.iter() {
        let result = function(item.clone());
        if result > best {
            best = result;
            best_item = item;
        }
    }
    // return function value with maximizing argument, the maximizing argument
    let result = (best_item.to_string(), best);
    result
}

fn greedy_findmax(pomdp: &pomdp::POMDP, u: &mut std::collections::HashMap<String, f64>, state: &String) -> (String, std::collections::HashMap<String, f64>) { 
    let maximize = findmax(&pomdp.a, |action| {pomdp.lookahead(&u, &state, &action)});
    let result = maximize;
    u.insert(state.to_string(), result.1);
    (result.0, u.clone())
}

macro_rules! iterative_policy_evaluation {
    ($a:expr, $b:expr, $c:expr) => {
        iterative_policy_evaluation($a, $b, $c, std::collections::HashMap::<String, f64>::new());
    };
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        iterative_policy_evaluation($a, $b, $c, $d);
    }
}
fn iterative_policy_evaluation(pomdp: pomdp::POMDP, pi: &String, k_max: i64, mut u: std::collections::HashMap<String, f64>) -> std::collections::HashMap<String, f64> {
    if u.len() == 0 {
        for i in 0..pomdp.s.len() {
            u.insert(pomdp.s[i].clone(), 0.0);
    }
    }
    for _ in 0..k_max {
        for state in &pomdp.s {
            *u.entry(state.to_string()).or_insert(0.0) = pomdp.lookahead(&u, &state, &pi.to_string());
        }
    }
    u
}

fn policy_evaluation(pomdp: &pomdp::POMDP, pi: &std::collections::HashMap<String, String>, mut u: std::collections::HashMap<String, f64>) -> std::collections::HashMap<String, f64> {
    //u.insert("row_0_col_0".to_string(), -123.4);
    for _ in 0..1 {
        for (state, policy) in pi {
                *u.entry(state.to_string()).or_insert(0.0) = pomdp.lookahead(&u, &state, &policy);
        }
    }
    u
}

#[derive(Debug)]
struct PolicyIteration {
    pi: std::collections::HashMap<String, String>, // initial policy
    k_max: u32 // max iterations
}

impl PolicyIteration {
    fn initialize(&mut self, pomdp: &pomdp::POMDP, default: &String) {
        for state in pomdp.s.iter() {
            self.pi.insert(state.to_string(), (&default).to_string());
        }
    }
}

fn policy_iteration(pomdp: &pomdp::POMDP, m: &mut PolicyIteration) {
    let mut u: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for i in 0..pomdp.s.len() {
        u.insert(pomdp.s[i].clone(), 0.0);
    }
    let mut new_policy: std::collections::HashMap<String, String>= std::collections::HashMap::new();
    for _ in 0..m.k_max {
        u = policy_evaluation(pomdp, &m.pi, u); // ("E", {})
        let mut new_state_policy;
        for (state, _) in &m.pi {
            new_state_policy = greedy_findmax(pomdp, &mut u, &state);
            new_policy.insert(state.to_string(), new_state_policy.0);
        }
    }
    m.pi = new_policy;
}


fn main() {
    let pomdp: pomdp::POMDP = hex_world::create_pomdp(0.9);
    //println!("{:?}", pomdp);

    //println!("iterative_policy_evaluation value {:?}", iterative_policy_evaluation!(pomdp, &"E".to_string(), 4));

    let mut policy = PolicyIteration { pi: std::collections::HashMap::<String, String>::new(), k_max: 10 };
    policy.initialize(&pomdp, &"E".to_string());
    policy_iteration(&pomdp, &mut policy);
    println!("Optimal policy = {:?}", policy);

}

