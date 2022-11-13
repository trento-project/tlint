use super::types::Check;
use colored::*;

fn check_header(head: &str) -> String {
    let padding = 16 - head.len() - 4;
    let mut header = format!("  {}  ", head);
    for _ in 0..padding {
        header.push(' ');
    }

    header.on_green().black().to_string()
}

pub fn print_check(check: Check) -> () {
    println!("{}  {}", check_header(&check.id), check.name);
    println!("{}  {}", check_header("Group"), check.group);
    println!("{}  {}", check_header("Description"), check.description);
    println!("\n{}", check_header("Remediation"));
    println!("  {}", check.remediation.replace("\n", "\n  "));
    println!("\n{}", check_header("Facts"));

    check.facts.into_iter().for_each(|fact| {
        println!("\n  {}  {}", check_header("Name"), fact.name);
        println!("  {}  {}", check_header("Gatherer"), fact.gatherer);
    });

    println!("\n{}", check_header("Expectations"));

    ()
}
