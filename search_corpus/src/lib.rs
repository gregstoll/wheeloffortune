extern crate json;
extern crate url;

use std::{collections::{HashMap, HashSet}, fs::File, io::{self, BufRead}};
use fst::IntoStreamer;
use regex::Regex;
use regex_automata::dense;
use memmap::Mmap;

// Use FST if number of questions marks is below this
// threshold. (set to 0 to never use FST, set to
// 1000 to always use FST)
const FST_QUESTION_MARK_THRESHOLD: usize = 6;

pub fn process_query_string(query: &str) -> Result<json::JsonValue, String> {
    let query_parts: HashMap<String, String> = url::form_urlencoded::parse(query.as_bytes()).into_owned().collect();
    let pattern = query_parts.get("pattern").ok_or(String::from("Internal error - no pattern specified!"))?;
    validate_pattern(pattern)?;
    let absent_letters = query_parts.get("absent_letters").ok_or(String::from("Internal error - no absent_letters specified!"))?;
    validate_absent_letters(absent_letters)?;
    let word_regex = build_regex(pattern, absent_letters)?;
    let pattern_question_marks = pattern.chars().filter(|c| *c == '?').count();
    let read_fst_file: bool = pattern_question_marks < FST_QUESTION_MARK_THRESHOLD;
    let json_results = if read_fst_file {
        let mmap = unsafe { Mmap::map(&File::open("../data/processed/word_frequency.fst").map_err(|e| e.to_string())?).map_err(|e| e.to_string())? };
        let map = fst::Map::new(mmap).map_err(|e| e.to_string())?;
        // need to strip off the ^ and $, but setting anchored to true will cover that
        let word_regex_pattern = &word_regex.as_str()[1..word_regex.as_str().len()-1];
        let dfa = dense::Builder::new().anchored(true).build(word_regex_pattern).unwrap();
        let mut results = map.search(&dfa).into_stream().into_str_vec().map_err(|e| e.to_string())?;
        results.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        results.iter().map(|entry| json::object! { "word" => entry.0.clone(), "frequency" => entry.1 }).collect()
    } else {
        let mut results = vec![];
        let mut line = String::new();
        let file = File::open("../data/processed/word_frequency.txt").map_err(|e| e.to_string())?;
        let mut reader = io::BufReader::new(file);
        while reader.read_line(&mut line).map_err(|e| e.to_string())? > 0 {
            let mut parts = line.split_ascii_whitespace();
            let word = parts.next().unwrap();
            if word_regex.is_match(word){
                // TODO - use Object constructor
                results.push(json::object! { "word" => word, "frequency" => parts.next().unwrap().parse::<u64>().unwrap() });
            }
            line.clear();
        }
        results
    };
    Ok(json::JsonValue::Array(json_results))
}

fn build_regex(pattern: &str, absent_letters: &str) -> Result<Regex, String> {
    let mut absent_letter_set = HashSet::new();
    for letter in absent_letters.chars() {
        //println!("inserting {} from absent", letter);
        absent_letter_set.insert(letter.to_ascii_lowercase());
    }
    for letter in pattern.chars() {
        if letter.is_ascii_alphabetic() {
            //println!("inserting {} from pattern", letter);
            absent_letter_set.insert(letter.to_ascii_lowercase());
        }
    }
    // regex gets cranky about turning off unicode then matching characters that aren't something 
    // (because they might be unicode characters!) so just iterate over all the possibilities here.
    let mut absent_letter_builder = "[".to_string();
    for letter in 'a'..='z' {
        if !absent_letter_set.contains(&letter) {
            absent_letter_builder.push(letter);
        }
    }
    absent_letter_builder.push(']');
    // (?-u) turns off unicode, although that's not really necessary here since we're already
    // specifying the exact characters to match.
    let mut regex_str = "^(?-u)".to_string();
    regex_str.push_str(&pattern.to_ascii_lowercase().replace("?", absent_letter_builder.as_str()));
    regex_str.push('$');
    Regex::new(&regex_str).map_err(|e| e.to_string())
}

fn is_allowed_char(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '\'' || c == '-' || c == '?'
}

fn validate_pattern(pattern: &str) -> Result<(), String> {
    if pattern.chars().any(|c| !is_allowed_char(c)) {
        return Err("Disallowed characters in pattern".to_string());
    }
    if pattern.len() > 20 {
        return Err("Pattern too long".to_string());
    }
    Ok(())
}

fn validate_absent_letters(absent_letters: &str) -> Result<(), String> {
    if absent_letters.chars().any(|c| !c.is_ascii_alphabetic()) {
        return Err("Disallowed characters in absent_letters".to_string());
    }

    Ok(())
}


mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_single_letter_missing() {
        let result = process_query_string("pattern=t?e&absent_letters=").unwrap();
        assert_eq!("the", result[0]["word"].to_string());
        assert_ne!(1, result.len());
    }

    #[test]
    fn test_single_letter_missing_and_not_the() {
        let result = process_query_string("pattern=t?e&absent_letters=h").unwrap();
        assert_eq!("tie", result[0]["word"].to_string());
        assert_ne!(1, result.len());
    }

    #[test]
    fn test_single_letter_missing_and_not_the_with_duplicate_absent_letters() {
        let result = process_query_string("pattern=t?e&absent_letters=ht").unwrap();
        assert_eq!("tie", result[0]["word"].to_string());
        assert_ne!(1, result.len());
    }

    #[test]
    fn test_single_letter_missing_and_not_the_with_extra_duplicate_absent_letters() {
        let result = process_query_string("pattern=t?e&absent_letters=htht").unwrap();
        assert_eq!("tie", result[0]["word"].to_string());
        assert_ne!(1, result.len());
    }

    #[test]
    fn test_no_letters_missing() {
        let result = process_query_string("pattern=is&absent_letters=").unwrap();
        assert_eq!("is", result[0]["word"].to_string());
        assert_eq!(1, result.len());
    }

    #[test]
    fn test_no_letters_missing_with_absent_letters() {
        let result = process_query_string("pattern=is&absent_letters=abc").unwrap();
        assert_eq!("is", result[0]["word"].to_string());
        assert_eq!(1, result.len());
    }

    #[test]
    fn test_all_results_right_length_and_descending_frequency() {
        let result = process_query_string("pattern=t???&absent_letters=h").unwrap();
        let mut last_value: u64 = 1000000000000;
        assert!(result.len() > 3);
        for i in 0..result.len() {
            assert_eq!(4, result[i]["word"].to_string().len());
            assert_eq!('t', result[i]["word"].to_string().chars().nth(0).unwrap());
            assert_ne!("this", result[i]["word"].to_string());
            let this_frequency = result[i]["frequency"].as_u64().unwrap();
            assert!(last_value >= this_frequency);
            last_value = this_frequency;
        }
    }

    #[test]
    fn test_no_reuse_letters_in_pattern() {
        let result = process_query_string("pattern=t?e?&absent_letters=").unwrap();
        let words = result.members().map(|x| x["word"].to_string()).collect::<Vec<String>>();
        assert!(!words.contains(&"tree".to_string()));
    }

    #[test]
    fn test_apostrophe() {
        let result = process_query_string("pattern=c??'t&absent_letters=").unwrap();
        let words = result.members().map(|x| x["word"].to_string()).collect::<Vec<String>>();
        assert!(words.contains(&"can't".to_string()));
    }

    #[test]
    fn test_apostrophe_not_filled_in() {
        let result = process_query_string("pattern=d???t&absent_letters=h").unwrap();
        let words = result.members().map(|x| x["word"].to_string()).collect::<Vec<String>>();
        assert!(!words.contains(&"don't".to_string()));
    }

    #[test]
    fn test_dash() {
        let result = process_query_string("pattern=n?n-?e??er&absent_letters=t").unwrap();
        let words = result.members().map(|x| x["word"].to_string()).collect::<Vec<String>>();
        assert!(words.contains(&"non-ledger".to_string()));
    }

    #[test]
    fn test_dash_not_filled_in() {
        let result = process_query_string("pattern=n?n??e??er&absent_letters=t").unwrap();
        let words = result.members().map(|x| x["word"].to_string()).collect::<Vec<String>>();
        assert!(!words.contains(&"non-ledger".to_string()));
    }

    #[test]
    fn test_all_results_right_length_with_missing_first_letter() {
        let result = process_query_string("pattern=??i?&absent_letters=h").unwrap();
        assert!(result.len() > 3);
        for i in 0..result.len() {
            assert_eq!(4, result[i]["word"].to_string().len());
            assert_eq!('i', result[i]["word"].to_string().chars().nth(2).unwrap());
            assert_ne!("this", result[i]["word"].to_string());
        }
    }

    #[test]
    fn test_giant_set_of_results_right_length_and_descending_frequency() {
        let result = process_query_string("pattern=?????&absent_letters=hx").unwrap();
        let mut last_value: u64 = 1000000000000;
        assert!(result.len() > 3);
        for i in 0..result.len() {
            assert_eq!(5, result[i]["word"].to_string().len());
            let this_frequency = result[i]["frequency"].as_u64().unwrap();
            assert!(last_value >= this_frequency);
            last_value = this_frequency;
        }
    }

    #[test]
    fn test_pattern_too_long() {
        let too_long = ".".repeat(21);
        let query = format!("pattern={}&absent_letters=h", too_long);
        let result = process_query_string(&query);
        assert!(!result.is_ok());
    }
}
