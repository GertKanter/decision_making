mod mdp;
mod pomdp;
mod hex_world;

#[allow(dead_code)]
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

#[allow(dead_code)]
fn greedy_findmax(mdp: &mdp::MDP, u: &mut std::collections::HashMap<String, f64>, state: &String) -> (String, std::collections::HashMap<String, f64>) { 
    let maximize = findmax(&mdp.a, |action| {mdp.lookahead(&u, &state, &action)});
    let result = maximize;
    u.insert(state.to_string(), result.1);
    (result.0, u.clone())
}

#[allow(unused_macros)]
macro_rules! iterative_policy_evaluation {
    ($a:expr, $b:expr, $c:expr) => {
        iterative_policy_evaluation($a, $b, $c, std::collections::HashMap::<String, f64>::new());
    };
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        iterative_policy_evaluation($a, $b, $c, $d);
    }
}

#[allow(dead_code)]
fn iterative_policy_evaluation(mdp: mdp::MDP, pi: &String, k_max: i64, mut u: std::collections::HashMap<String, f64>) -> std::collections::HashMap<String, f64> {
    if u.len() == 0 {
        for i in 0..mdp.s.len() {
            u.insert(mdp.s[i].clone(), 0.0);
    }
    }
    for _ in 0..k_max {
        for state in &mdp.s {
            *u.entry(state.to_string()).or_insert(0.0) = mdp.lookahead(&u, &state, &pi.to_string());
        }
    }
    u
}

#[allow(dead_code)]
fn policy_evaluation(mdp: &mdp::MDP, pi: &std::collections::HashMap<String, String>, mut u: std::collections::HashMap<String, f64>) -> std::collections::HashMap<String, f64> {
    for _ in 0..1 {
        for (state, policy) in pi {
                *u.entry(state.to_string()).or_insert(0.0) = mdp.lookahead(&u, &state, &policy);
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
    #[allow(dead_code)]
    fn initialize(&mut self, mdp: &mdp::MDP, default: &String) {
        for state in mdp.s.iter() {
            self.pi.insert(state.to_string(), (&default).to_string());
        }
    }
}

#[allow(dead_code)]
fn policy_iteration(mdp: &mdp::MDP, m: &mut PolicyIteration) {
    let mut u: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for i in 0..mdp.s.len() {
        u.insert(mdp.s[i].clone(), 0.0);
    }
    let mut new_policy: std::collections::HashMap<String, String>= std::collections::HashMap::new();
    for _ in 0..m.k_max {
        u = policy_evaluation(mdp, &m.pi, u); // ("E", {})
        let mut new_state_policy;
        for (state, _) in &m.pi {
            new_state_policy = greedy_findmax(mdp, &mut u, &state);
            new_policy.insert(state.to_string(), new_state_policy.0);
        }
    }
    m.pi = new_policy;
}

#[allow(dead_code)]
fn belief_update(pomdp: pomdp::POMDP, beliefs: std::collections::HashMap<String, f64>, a: String, o: String) {
}

fn main() {
    let mdp: mdp::MDP = hex_world::create_mdp(0.9);
    //println!("{:?}", mdp);

}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_test<T>(test: T) -> () where T: FnOnce() -> () + std::panic::UnwindSafe {
        let result = std::panic::catch_unwind(|| {
            test()
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_iterative_policy_evaluation() {
        run_test(|| {
            let mdp = hex_world::create_mdp(0.9);
            let return_value = iterative_policy_evaluation!(mdp, &"E".to_string(), 4);
            let mut expected = std::collections::HashMap::<String, f64>::new();
            expected.insert("row_0_col_8".to_string(), -0.9530023239375);
            expected.insert("row_0_col_18".to_string(), -3.439);
            expected.insert("row_0_col_12".to_string(), -1.01587436495625);
            expected.insert("row_2_col_18".to_string(), 10.0);
            expected.insert("row_2_col_4".to_string(), -10.0);
            let mut delta = 0.0;
            for key in expected.keys() {
                assert!(return_value.contains_key(key));
                delta += f64::powf(expected.get(key).unwrap() - return_value.get(key).unwrap(), 2.0).sqrt();
                assert!(delta < 0.000001);
            }
            //println!("return {:?}", return_value);
            //assert_eq!(return_value, expected);
        })
    }


    #[test]
    fn test_policy_iteration() {
        run_test(|| {
            let mdp = hex_world::create_mdp(0.9);
            let mut policy = PolicyIteration { pi: std::collections::HashMap::<String, String>::new(), k_max: 10 };
            policy.initialize(&mdp, &"E".to_string());
            policy_iteration(&mdp, &mut policy);
            let mut expected = std::collections::HashMap::<String, String>::new();
            expected.insert("row_2_col_10".to_string(), "E".to_string());
            expected.insert("row_2_col_0".to_string(), "NE".to_string());
            expected.insert("row_1_col_17".to_string(), "SE".to_string());
            expected.insert("row_2_col_2".to_string(), "NW".to_string());
            expected.insert("row_0_col_16".to_string(), "SE".to_string());
            for key in expected.keys() {
                assert!(policy.pi.contains_key(key));
                assert!(policy.pi.get(key).unwrap() == expected.get(key).unwrap());
            }
            //println!("Optimal policy = {:?}", policy);
            //assert!(false);
        })
    }

}
