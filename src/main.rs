mod pomdp;
mod hex_world;

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
    println!("iterative_policy_evaluation value {:?}", iterative_policy_evaluation(pomdp, &"E".to_string(), 4));
}

