pub fn run() {
    for action in super::providers::list_action_lines() {
        println!("{}", action);
    }
}
