use std::collections::HashMap;

#[allow(dead_code)]
pub fn create_pomdp(gamma: f64) -> super::pomdp::POMDP {
    let state_space = Vec::from(["hungry".to_string(), "sated".to_string()]);

    let action_space = Vec::from(["feed".to_string(), "sing".to_string(), "ignore".to_string()]);

    let observation_space = Vec::from(["crying".to_string(), "quiet".to_string()]);

    let mut transition_function: std::collections::HashMap<(String, String), HashMap<String, f64>> = std::collections::HashMap::new();
    transition_function.insert(("hungry".to_string(), "feed".to_string()), vec![("sated".to_string(), 1.0)].into_iter().collect());
    transition_function.insert(("hungry".to_string(), "sing".to_string()), vec![("hungry".to_string(), 1.0)].into_iter().collect());
    transition_function.insert(("hungry".to_string(), "ignore".to_string()), vec![("hungry".to_string(), 1.0)].into_iter().collect());
    transition_function.insert(("sated".to_string(), "feed".to_string()), vec![("sated".to_string(), 1.0)].into_iter().collect());
    transition_function.insert(("sated".to_string(), "sing".to_string()), vec![("hungry".to_string(), 0.1), ("sated".to_string(), 0.9)].into_iter().collect());
    transition_function.insert(("sated".to_string(), "ignore".to_string()), vec![("hungry".to_string(), 0.1), ("sated".to_string(), 0.9)].into_iter().collect());
    

    let mut reward_function: std::collections::HashMap<(String, String), f64> = std::collections::HashMap::new();
    reward_function.insert(("hungry".to_string(), "feed".to_string()), -15.0); // Reward -10-5 if feed baby in hungry
    reward_function.insert(("sated".to_string(), "feed".to_string()), -5.0);
    reward_function.insert(("sated".to_string(), "sing".to_string()), -0.5);
    reward_function.insert(("hungry".to_string(), "sing".to_string()), -10.5);
    reward_function.insert(("sated".to_string(), "ignore".to_string()), 0.0);
    reward_function.insert(("hungry".to_string(), "ignore".to_string()), -10.0);

    let mut observation_function: HashMap<(String, String, String), f64> = std::collections::HashMap::new(); 
    observation_function.insert(("crying".to_string(), "feed".to_string(), "hungry".to_string()), 0.8);
    observation_function.insert(("quiet".to_string(), "feed".to_string(), "hungry".to_string()), 0.2);
    observation_function.insert(("crying".to_string(), "feed".to_string(), "sated".to_string()), 0.1);
    observation_function.insert(("quiet".to_string(), "feed".to_string(), "sated".to_string()), 0.9);

    observation_function.insert(("crying".to_string(), "ignore".to_string(), "hungry".to_string()), 0.8);
    observation_function.insert(("quiet".to_string(), "ignore".to_string(), "hungry".to_string()), 0.2);
    observation_function.insert(("crying".to_string(), "ignore".to_string(), "sated".to_string()), 0.1);
    observation_function.insert(("quiet".to_string(), "ignore".to_string(), "sated".to_string()), 0.9);

    observation_function.insert(("crying".to_string(), "sing".to_string(), "hungry".to_string()), 0.9);
    observation_function.insert(("quiet".to_string(), "sing".to_string(), "hungry".to_string()), 0.1);
    observation_function.insert(("crying".to_string(), "sing".to_string(), "sated".to_string()), 0.0);
    observation_function.insert(("quiet".to_string(), "sing".to_string(), "sated".to_string()), 1.0);

    super::pomdp::POMDP { gamma: gamma, state_space: state_space, action_space: action_space, observation_space: observation_space, transition_function: transition_function, reward_function: reward_function, observation_function: observation_function}
}
