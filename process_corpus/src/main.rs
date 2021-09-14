use std::{collections::HashMap, fs::File, io::{self, BufRead}, path::Path};
use anyhow::{anyhow, Result};

type WordFrequency = HashMap<String, u64>;

fn main() -> Result<()> {
    println!("Hello, world!");
    let mut freq = WordFrequency::new();
    //for i in 0..=14 {
    for i in 0..=10 {
        let path = format!("../data/raw/1-{:05}-of-00014", i);
        println!("parsing file #{}, have {} entries so far...", i, freq.len());
        parse_file(&path, &mut freq)?;
    }
    let mut entries = freq.iter().collect::<Vec<_>>();
    println!("got {:?} words", freq.len());
    entries.sort_by(|a, b| b.1.cmp(a.1));
    for i in 0..10 {
        println!("{:?}", entries[i]);
    }
    
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
    let mut count = 0;
    for entry in parts {
        // each entry is "<year>,<match count>,<volume count>"
        let mut entry_parts = entry.split(',');
        // TODO - only count stuff since 1920 or something?
        let _year = entry_parts.next().ok_or_else(|| anyhow!("no year in entry"))?;
        let entry_count = entry_parts.next().ok_or_else(|| anyhow!("no count in entry"))?;
        let entry_count = entry_count.parse::<u64>().map_err(|e| anyhow!("couldn't parse count: {}", e))?;
        count += entry_count;
    }
    freq.entry(word).and_modify(|e| *e += count).or_insert(count);
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

fn parse_file(path: &str, freq: &mut WordFrequency) -> Result<()> {
    let lines = read_lines(path)?;
    for line in lines {
        let line = line?;
        parse_line(&line, freq)?;
    }

    Ok(())
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