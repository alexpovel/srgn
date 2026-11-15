const MESSAGE: &str = "needle";
fn needle() {}

#[cfg(test)]
mod checks {
    use super::*;

    #[test]
    fn needle() {
        assert_eq!(MESSAGE, "needle");
    }
}

fn after() {
    println!("needle");
}
