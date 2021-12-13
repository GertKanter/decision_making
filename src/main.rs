mod mdp;
mod pomdp;
mod hex_world;
mod crying_baby;
mod util;

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
        u = policy_evaluation(mdp, &m.pi, u);
        let mut new_state_policy;
        for (state, _) in &m.pi {
            new_state_policy = greedy_findmax(mdp, &mut u, &state);
            new_policy.insert(state.to_string(), new_state_policy.0);
        }
    }
    m.pi = new_policy;
}

#[allow(dead_code)]
fn update_beliefs(pomdp: &pomdp::POMDP, beliefs: &mut std::collections::HashMap<String, f64>, a: &String, o: &String) {
    let original_values = beliefs.clone();
    for state in &pomdp.state_space { // every target state as s'
        if pomdp.observation_function.contains_key(&(o.to_string(), a.to_string(), state.to_string())) {
            let observation = pomdp.observation_function.get(&(o.to_string(), a.to_string(), state.to_string())).unwrap();
            let mut sum = 0.0;
            for sum_state in &pomdp.state_space {
                if pomdp.transition_function.contains_key(&(sum_state.to_string(), a.to_string())) {
                    let transition = pomdp.transition_function.get(&(sum_state.to_string(), a.to_string())).unwrap();
                    if transition.contains_key(&state.to_string()) {
                        sum += transition.get(&state.to_string()).unwrap() * original_values.get(sum_state).unwrap();
                    }
                }
                let new_belief = observation * sum;
                beliefs.insert(state.to_string(), new_belief);
            }

        }
        else {
            beliefs.insert(state.to_string(), 0.0);
        }

    }
    *beliefs = util::normalize(beliefs, 1.0);
}



fn main() {
    //let mdp: mdp::MDP = hex_world::create_mdp(0.9);
    //println!("{:?}", mdp);
    let pomdp = crying_baby::create_pomdp(0.9);
    /*let mut beliefs = std::collections::HashMap::<String, f64>::new();
    for state in &pomdp.state_space {
        beliefs.insert(state.to_string(), 1.0);
    }
    beliefs = util::normalize(&mut beliefs,1.0);*/
    //pomdp::value_iteration(&pomdp, 1);
    let result = pomdp::solve(3, pomdp);
    println!("{:?}", result);

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
            /*println!("{:?}", mdp);
            assert!(false);*/
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

    #[test]
    fn test_belief_update() {
        run_test(|| {
            let pomdp = crying_baby::create_pomdp(0.9);
            let mut beliefs = std::collections::HashMap::<String, f64>::new();
            for state in &pomdp.state_space {
                beliefs.insert(state.to_string(), 1.0);
            }
            beliefs = util::normalize(&mut beliefs,1.0);
            println!("beliefs = {:?}", beliefs);
            println!("Ignore the baby!");
            update_beliefs(&pomdp, &mut beliefs, &"ignore".to_string(), &"crying".to_string());
            println!("beliefs = {:?}", beliefs);
            println!("Feed the baby!");
            update_beliefs(&pomdp, &mut beliefs, &"feed".to_string(), &"quiet".to_string());
            println!("beliefs = {:?}", beliefs);
            println!("Ignore the baby!");
            update_beliefs(&pomdp, &mut beliefs, &"ignore".to_string(), &"quiet".to_string());
            println!("beliefs = {:?}", beliefs);
            println!("Ignore the baby!");
            update_beliefs(&pomdp, &mut beliefs, &"ignore".to_string(), &"quiet".to_string());
            println!("beliefs = {:?}", beliefs);
            println!("Ignore the baby!");
            update_beliefs(&pomdp, &mut beliefs, &"ignore".to_string(), &"crying".to_string());
            println!("beliefs = {:?}", beliefs);
            println!("Sing to the baby!");
            update_beliefs(&pomdp, &mut beliefs, &"sing".to_string(), &"quiet".to_string());
            println!("beliefs = {:?}", beliefs);
            assert!(f64::powf(beliefs.get(&"sated".to_string()).unwrap() - 0.87697, 2.0).sqrt() < 0.0001);
            //assert!(false);
        })
    }

    #[test]
    fn test_one_step_pomdp_plan() {
        run_test(|| {
            let pomdp = crying_baby::create_pomdp(0.9);
            let result = pomdp::solve(1, pomdp);
            println!("{:?}", result);
            assert!(result.big_gamma[0][0] == -10.0);
            assert!(result.big_gamma[0][1] == 0.0);
        })
    }

    #[test]
    fn test_two_step_pomdp_plan() {
        run_test(|| {
            let pomdp = crying_baby::create_pomdp(0.9);
            let result = pomdp::solve(2, pomdp);
            println!("{:?}", result);
            assert!(result.big_gamma[0][0] == -15.0);
            assert!(result.big_gamma[0][1] == -5.0);
            assert!(result.big_gamma[1][0] == -19.0);
            assert!(result.big_gamma[1][1] == -0.9);
        })
    }

    #[test]
    fn test_three_step_pomdp_plan() {
        run_test(|| {
            let pomdp = crying_baby::create_pomdp(0.9);
            let result = pomdp::solve(3, pomdp);
            println!("{:?}", result);
            assert!(result.big_gamma.len() == 2);
            assert!((result.big_gamma[0][0] - (-16.179)).abs() < 0.0000001);
            assert!((result.big_gamma[0][1] - (-6.179)).abs() < 0.0000001);
            assert!((result.big_gamma[1][0] - (-24.22)).abs() < 0.0000001);
            assert!((result.big_gamma[1][1] - (-2.4831)).abs() < 0.0000001);
        })
    }
    #[test]
    fn test_set_configuration() {
        run_test(|| {
            let pomdp = crying_baby::create_pomdp(0.9);
            let mut plan = pomdp::ConditionalPlan::default();
            plan.action = "root".to_string();

            let mut sub = pomdp::ConditionalPlan::default();
            sub.action = "sub1".to_string();
            let mut sub2 = pomdp::ConditionalPlan::default();
            sub2.action = "sub2".to_string();

            let mut sub3 = pomdp::ConditionalPlan::default();
            sub3.action = "sub3".to_string();
            let mut sub_bt = std::collections::BTreeMap::<String, Box<pomdp::ConditionalPlan>>::new();
            sub_bt.insert("quiet".to_string(), Box::new(sub3));
            sub2.subplans = Some(sub_bt);

            let mut bt = std::collections::BTreeMap::<String, Box<pomdp::ConditionalPlan>>::new();
            bt.insert("quiet".to_string(), Box::new(sub));
            bt.insert("crying".to_string(), Box::new(sub2));
            plan.subplans = Some(bt);
            let mut configuration = vec![0, 2, 1, 2];
            //println!("starting config = {:?}", configuration);
            println!("plan before set_configuration == {:?}", plan);
            plan.set_configuration(&pomdp, &mut configuration);
            println!("plan after set_configuration == {:?}", plan);
            assert!(plan.action == "ignore");
            assert!(plan.subplans.as_ref().unwrap().get("quiet").unwrap().action == "feed");
            assert!(plan.subplans.as_ref().unwrap().get("crying").unwrap().action == "sing");
            assert!(plan.subplans.as_ref().unwrap().get("crying").unwrap().subplans.as_ref().unwrap().get("quiet").unwrap().action == "ignore");
        })
    }

}
