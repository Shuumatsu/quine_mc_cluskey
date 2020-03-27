#![feature(bindings_after_at)]

mod lib;

use lib::{pretty_print_term, AssociatedTerm};
use std::collections::HashSet;

fn print_chart(
    primes: &HashSet<AssociatedTerm>,
    related_terms: &Vec<usize>,
    variables: &Vec<String>,
) -> String {
    let mut header: Vec<String> = vec![];
    header.push(format!("{:32}", "expression"));
    for &term in related_terms.iter() {
        header.push(format!("| {:4}", term.to_string()));
    }
    let mut mapping = std::collections::HashMap::new();
    for (idx, term) in related_terms.iter().enumerate() {
        mapping.insert(term, idx);
    }

    let mut contents: Vec<String> = vec![];
    for term @ AssociatedTerm { min_terms, .. } in primes {
        let mut content: Vec<String> = vec![format!("| {:4}", ""); header.len()];
        content[0] = format!("{:32}", pretty_print_term(term, variables));
        for min_term in min_terms {
            content[*mapping.get(min_term).unwrap() + 1] = format!("| {:4}", "V");
        }
        contents.push(content.join(""));
    }

    format!("{}\n{}", header.join(""), contents.join("\n"))
}

fn main() {
    // let mut a = std::collections::BTreeSet::new();
    // a.insert(1);
    // let mut b = std::collections::BTreeSet::new();
    // b.insert(1);
    // let mut m = std::collections::HashSet::new();
    // m.insert(a);
    // println!("~~~ {}", m.contains(&b));
    // println!("{:?}", m);

    let variables: Vec<String> = "A-B-C-D-E".split("-").map(String::from).collect();
    println!("{:?}", variables);
    let terms = vec![0, 2, 3, 8, 10, 14, 15, 22, 24, 27, 31];

    let primes = lib::minimize(terms.clone(), variables.len());

    println!("{:?}", primes);

    println!("---");
    println!("{}", print_chart(&primes, &terms, &variables));
    println!("---");

    let readable = lib::pretty_print(&primes, &variables);
    println!("{}", readable);
}
