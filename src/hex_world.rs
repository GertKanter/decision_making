use std::collections::HashMap;

pub fn create_pomdp(gamma: f64) -> super::pomdp::POMDP {
    let mut states = Vec::new();//Vec::from(["r0c1" "state2"]);
    // Skip some cells. See "Algorithms for Decision Making" book, appendix F.1
    let skip: Vec<String> = Vec::from(["row_1_col_7".to_string(), "row_1_col_11".to_string(), "row_1_col_13".to_string(), "row_2_col_8".to_string(), "row_2_col_16".to_string()]);
    let cols = 10u32;
    for row in 0..3u32 {
        let mut col = 0;
        while col < cols*2 {
            let mut column = col;
            if row % 2 == 1 {
                column+=1;
                if column >= (cols-1)*2 {
                    break;
                }
            }
            let string = String::from("row_") + &row.to_string() + &String::from("_col_") + &column.to_string();
            if !skip.contains(&string) {
                states.push(string);
            }
            col += 2;
        }
    }

    let actions = Vec::from(["E".to_string(), "SE".to_string(), "SW".to_string(), "W".to_string(), "NW".to_string(), "NE".to_string()]);

    let mut transitions: std::collections::HashMap<(String, String), HashMap<String, f64>> = std::collections::HashMap::new();
    for from in states.iter() {
        for i in 0..actions.len() {
            let prev = if i == 0 { actions.len() - 1 } else { i - 1 };
            let next = if i + 1 == actions.len() { 0 } else { i + 1 };
            let action_triple = Vec::from([&actions[prev], &actions[i], &actions[next]]);
            for action in action_triple.iter() {
                let result = make_target_state(from, action, &states);
                let mut probability: f64 = 0.7;
                if **action != actions[i] {
                    probability = 0.15;
                }
                match result {
                    Some(x) => {
                        if transitions.contains_key(&(from.to_string(), actions[i].to_string())) {
                            let entry = transitions.get_mut(&(from.to_string(), actions[i].to_string())).unwrap();
                            if entry.contains_key(&x) {
                                let previous_value = entry[&x];
                                entry.insert(from.to_string(), previous_value + probability);
                            }
                            else {
                                // insert into entry
                                entry.insert(x.to_string(), probability);
                            }
                        }
                        else {
                            let mut new = HashMap::<String, f64>::new();
                            new.insert(x.to_string(), probability);
                            transitions.insert((from.to_string(), actions[i].to_string()), new);
                        }
                        ()
                    },
                    None => {
                        // Can't go there... add a transition to same state
                        if transitions.contains_key(&(from.to_string(), actions[i].to_string())) {
                            let entry = transitions.get_mut(&(from.to_string(), actions[i].to_string())).unwrap();
                            if entry.contains_key(from) {
                                let previous_value = entry[from];
                                entry.insert(from.to_string(), previous_value + probability);
                            }
                            else {
                                // insert into entry
                                entry.insert(from.to_string(), probability);
                            }
                        }
                        else {
                            let mut new = HashMap::<String, f64>::new();
                            new.insert(from.to_string(), probability);
                            transitions.insert((from.to_string(), actions[i].to_string()), new);
                        }
                        ()
                    },
                }
            }
        }
    }

    let mut rewards: std::collections::HashMap<(String, String), f64> = std::collections::HashMap::new();
    // state, action = reward
    for state in states.iter() {
        // Bumping into walls is -1 reward
        for action in actions.iter() {
            // get probability of action having the same end state as the start state (i.e., bump into wall)
            if transitions.contains_key(&(state.to_string(), action.to_string())) {
                let entry = transitions.get(&(state.to_string(), action.to_string())).unwrap();
                if entry.contains_key(&state.to_string()) {
                    rewards.insert((state.to_string(), action.to_string()), -1.0 * entry.get(&state.to_string()).unwrap());
                    continue;
                }
            }
        }
    }

    // Add termination states and rewards. See F.1 in DM
    let termination = Vec::from([("row_1_col_1", 5.0), ("row_2_col_4", -10.0), ("row_2_col_18", 10.0)]);
    states.push("terminal".to_string());
    for state in termination.iter() {
        for action in actions.iter() {
            // remove existing transition from the penultimate state
            transitions.remove(&(state.0.to_string(), action.to_string()));
            // add transition to terminal state
            let mut termination_transitions = HashMap::<String, f64>::new();
            termination_transitions.insert("terminal".to_string(), 1.0);
            transitions.insert((state.0.to_string(), action.to_string()), termination_transitions);
            // add reward
            rewards.insert((state.0.to_string(), action.to_string()), state.1);
        }
    }

    super::pomdp::POMDP { gamma: gamma, s: states, a: actions, t: transitions, r: rewards}
}

fn make_target_state(from: &String, action: &String, states: &Vec<String>) -> Option<String> {
    let split: Vec<&str> = from.split("_").collect();
    let mut row: i32 = split[1].parse().unwrap();
    let mut col: i32 = split[3].parse().unwrap();
    if action == "NE" {
        row -= 1;
        col += 1;
    }
    else if action == "E" {
        col += 2;
    }
    else if action == "SE" {
        row += 1;
        col += 1;
    }
    else if action == "SW" {
        row += 1;
        col -= 1;
    }
    else if action == "W" {
        col -= 1;
    }
    else if action == "NW" {
        row -= 1;
        col -= 1;
    }
    let string = String::from("row_") + &row.to_string() + &String::from("_col_") + &col.to_string();
    for state in states.iter() {
        if string == *state {
            return Some(string);
        }
    }
    None
}
