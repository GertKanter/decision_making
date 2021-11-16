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

fn greedy_findmax(pomdp: pomdp::POMDP, u: &mut std::collections::HashMap<String, f64>, state: &String) -> (String, std::collections::HashMap<String, f64>) { 
    let maximize = findmax(&pomdp.a, |action| {pomdp.lookahead(&u, &state, &action)});
    let result = maximize;
    u.insert(state.to_string(), result.1);
    (result.0, u.clone())
}

fn iterative_policy_evaluation(pomdp : pomdp::POMDP, pi: &String, k_max: i64) -> std::collections::HashMap<String, f64> {
    let mut u: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for i in 0..pomdp.s.len() {
        u.insert(pomdp.s[i].clone(), 0.0);
    }
    for _ in 0..k_max {
        for state in &pomdp.s {
            *u.entry(state.to_string()).or_insert(0.0) = pomdp.lookahead(&u, &state, &pi.to_string());
        }
    }
    u
}


fn main() {
    let pomdp: pomdp::POMDP = hex_world::create_pomdp(0.9);
    //println!("{:?}", pomdp);

    //println!("iterative_policy_evaluation value {:?}", iterative_policy_evaluation(pomdp, &"E".to_string(), 4));

    let mut u: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for i in 0..pomdp.s.len() {
        u.insert(pomdp.s[i].clone(), 0.0);
    }
    println!("greedy_findmax = {:?}", greedy_findmax(pomdp, &mut u, &"row_0_col_0".to_string()));
}

