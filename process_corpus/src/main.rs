use std::{collections::HashMap, fs::File, io::{self, BufRead}, path::Path};
use anyhow::{anyhow, Result};

type WordFrequency = HashMap<String, u64>;

fn main() -> Result<()> {
    println!("Hello, world!");
    let freq = parse_file("../data/raw/1-00006-of-00014")?;
    println!("got {:?} words", freq.len());
    
    Ok(())
}

fn is_allowed_char(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '\'' || c == '_'
}

fn parse_line(line: &str, freq: &mut WordFrequency) -> Result<()> {
    let mut parts = line.split_ascii_whitespace();
    let word = parts.next().ok_or_else(|| anyhow!("no word"))?;
    if word.chars().any(|c| !is_allowed_char(c)) {
        //println!("skipping {} because disallowed char", word);
        return Ok(());
    }
    let word = if word.chars().any(|c| c == '_') {
            let final_word = trim_part_of_speech(word);
            if final_word.chars().any(|c| c == '_') {
                //println!("skipping {} because underscores in weird places", word);
                return Ok(());
            }
            final_word
        }
        else {
            word
        };
    let word = word.to_ascii_lowercase();
    // TODO - count
    freq.entry(word).and_modify(|e| *e += 1).or_insert(1);
    Ok(())
}

// skipping _NUM since we don't want numbers anyway
const SUFFIXES: &[&str] = &["_NOUN", "_VERB", "_ADJ", "_ADV", "_ADP", "_PRON", "_DET", "_CONJ", "_PRT"];
fn trim_part_of_speech(word: &str) -> &str {
    for suffix in SUFFIXES {
        if word.ends_with(suffix) {
            return &word[..word.len() - suffix.len()];
        }
    }
    return word;
}

fn parse_file(path: &str) -> Result<WordFrequency> {
    let mut freq = WordFrequency::new();
    let lines = read_lines(path)?;
    for line in lines {
        let line = line?;
        parse_line(&line, &mut freq)?;
    }

    Ok(freq)
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_invalid_char() -> Result<()> {
        let mut freq = WordFrequency::new();
        parse_line("a.b\t1960,1,1", &mut freq)?;
        assert_eq!(freq.len(), 0);
        Ok(())
    }

    #[test]
    fn test_trim_part_of_speech_no_part_of_speech() {
        assert_eq!("hello", trim_part_of_speech("hello"));
    }

    #[test]
    fn test_trim_part_of_speech_wrong_part_of_speech() {
        assert_eq!("hello_NOTREAL", trim_part_of_speech("hello_NOTREAL"));
    }

    #[test]
    fn test_trim_part_of_speech_noun() {
        assert_eq!("hello", trim_part_of_speech("hello_NOUN"));
    }

    #[test]
    fn test_trim_part_of_speech_prt() {
        assert_eq!("hello", trim_part_of_speech("hello_PRT"));
    }

    #[test]
    fn test_trim_part_of_speech_noun_but_not_at_end() {
        assert_eq!("hello_NOUN_B", trim_part_of_speech("hello_NOUN_B"));
    }
}