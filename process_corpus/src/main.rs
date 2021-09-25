use std::{collections::HashMap, fs::File, io::{self, BufRead, Write}};
use anyhow::{anyhow, Result};

type WordFrequency = HashMap<String, u64>;

const WRITE_FST_FILE: bool = true;
const FREQUENCY_CUTOFF: u64 = 10000;

fn main() -> Result<()> {
    println!("Hello, world!");
    let mut freq = WordFrequency::new();
    for i in 0..=13 {
        let path = format!("../data/raw/1-{:05}-of-00014", i);
        println!("parsing file #{}, have {} entries so far...", i, freq.len());
        parse_file(&path, &mut freq)?;
    }
    let mut entries = freq.iter().collect::<Vec<_>>();
    println!("got {:?} words", freq.len());
    entries.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    for i in 0..25 {
        println!("{:?}", entries[i]);
    }
    let mut file = File::create("../data/processed/word_frequency.txt")?;
    // There are lots of incredibly rare words, as well as a ton of typos (where
    // presumably the OCR was wrong) at low frequencies. On inspection, we're not
    // losing anything valuable if we cut off at 10000, and it reduces false positives
    // and file size.
    for entry in &entries {
        if *(entry.1) >= FREQUENCY_CUTOFF {
            writeln!(file, "{} {}", entry.0, entry.1)?;
        }
    }
    if WRITE_FST_FILE {
        let mut filtered_entries = entries.iter().filter(|e| *e.1 >= FREQUENCY_CUTOFF).collect::<Vec<_>>();
        filtered_entries.sort_by(|a, b| a.0.cmp(b.0));
        let writer = io::BufWriter::new(File::create("../data/processed/word_frequency.fst")?);
        let mut fst_builder = fst::MapBuilder::new(writer)?;
        for entry in &filtered_entries {
            fst_builder.insert(entry.0.as_bytes(), *entry.1)?;
        }
        fst_builder.finish()?;
    }
    Ok(())
}

fn is_allowed_char(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '\'' || c == '_' || c == '-'
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
        // TODO - only count stuff since 1920 or something? Or weight later years later?
        let _year = entry_parts.next().ok_or_else(|| anyhow!("no year in entry {}", entry))?;
        let entry_count = entry_parts.next().ok_or_else(|| anyhow!("no count in entry {}", entry))?;
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
    let mut line = String::new();
    let file = File::open(path)?;
    let mut reader = io::BufReader::new(file);
    while reader.read_line(&mut line)? > 0 {
        parse_line(&line, freq)?;
        line.clear();
    }

    Ok(())
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