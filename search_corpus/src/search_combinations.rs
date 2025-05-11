use search_corpus::search_combinations;
use std::env;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let parts = &args[1..];
    let char_parts: Vec<Vec<char>> = parts
        .iter()
        .map(|w| w.chars().collect::<Vec<char>>())
        .collect();
    let results = search_combinations(&char_parts)?;
    println!("Got {} results", results.len());
    for result in results.iter() {
        println!("{}: {}", result.0, result.1);
    }
    Ok(())
}
