extern crate json;
extern crate url;

use fst::IntoStreamer;
use memmap::Mmap;
use regex::Regex;
use regex_automata::dense;
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    fs::File,
    io::{self, BufRead},
    path::Path,
};

// Use FST if number of questions marks is below this
// threshold. (set to 0 to never use FST, set to
// 1000 to always use FST)
const FST_QUESTION_MARK_THRESHOLD: usize = 6;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PatternMode {
    WheelOfFortune,
    Crossword,
    Cryptogram,
}

impl TryFrom<&str> for PatternMode {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "WheelOfFortune" => Ok(PatternMode::WheelOfFortune),
            "Crossword" => Ok(PatternMode::Crossword),
            "Cryptogram" => Ok(PatternMode::Cryptogram),
            _ => Err(()),
        }
    }
}

pub fn find_processed_file(filename: &str) -> String {
    let mut path: String = format!("data/processed/{}", filename);
    for _i in 0..5 {
        if Path::new(&path).exists() {
            return path;
        }
        path = format!("../{}", path);
    }
    panic!("Couldn't find file!");
}

///
///
/// # Examples
///
/// ```
/// use search_corpus::search_combinations;
///
/// assert_eq!(search_combinations(&vec![vec!['B', 'D'], vec!['A'], vec!['T', 'D']]),
///            Ok(vec![("bad".to_string(), 164493412), ("dad".to_string(), 33921229), ("bat".to_string(), 13047332), ("dat".to_string(), 5705367)]));
/// ```
pub fn search_combinations(parts: &[Vec<char>]) -> Result<Vec<(String, u64)>, String> {
    let mut regex_str = "(?-u)".to_string();
    for slot in parts {
        regex_str.push('[');
        for ch in slot {
            regex_str.push(ch.to_ascii_lowercase());
        }
        regex_str.push(']');
    }
    dbg!(&regex_str);
    let mmap = unsafe {
        Mmap::map(
            &File::open(find_processed_file("word_frequency.fst")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?
    };
    let map = fst::Map::new(mmap).map_err(|e| e.to_string())?;
    let dfa = dense::Builder::new()
        .anchored(true)
        .build(&regex_str)
        .unwrap();
    let mut results = map
        .search(&dfa)
        .into_stream()
        .into_str_vec()
        .map_err(|e| e.to_string())?;
    results.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    Ok(results)
}

fn is_valid_string(s: &str, pattern: &str, mode: &PatternMode) -> bool {
    if mode == &PatternMode::Cryptogram {
        // the regex crate doesn't support backreferences, so make sure
        // that capital letters in the pattern match in the result string
        // TODO - also make sure that no mappings are reused?
        let mut mappings: HashMap<char, char> = HashMap::new();
        let s_chars = s.chars().collect::<Vec<_>>();
        for (i, pattern_char) in pattern.char_indices() {
            if pattern_char.is_ascii_uppercase() {
                let entry = mappings.entry(pattern_char);
                match entry {
                    std::collections::hash_map::Entry::Occupied(occupied_entry) => {
                        if &s_chars[i] != occupied_entry.get() {
                            return false;
                        }
                    }
                    std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                        vacant_entry.insert(s_chars[i]);
                    }
                }
            }
        }
    }
    return true;
}

pub fn process_query_string(query: &str) -> Result<json::JsonValue, String> {
    let query_parts: HashMap<String, String> = url::form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect();
    let mode = query_parts
        .get("mode")
        .ok_or(String::from("Internal error - no mode specified!"))?;
    let mode = PatternMode::try_from(mode.as_str())
        .map_err(|_| String::from("Internal error - invalid mode!"))?;
    let pattern = query_parts
        .get("pattern")
        .ok_or(String::from("Internal error - no pattern specified!"))?;
    validate_pattern(pattern, &mode)?;
    // TODO - validate if in WheelOfFortune mode?
    //let absent_letters = query_parts.get("absent_letters").ok_or(String::from("Internal error - no absent_letters specified!"))?;
    let empty_absent_letters = String::from("");
    let absent_letters = query_parts
        .get("absent_letters")
        .unwrap_or(&empty_absent_letters);
    validate_absent_letters(absent_letters)?;
    let word_regex = build_regex(pattern, absent_letters, &mode)?;
    let pattern_question_marks = if mode == PatternMode::Cryptogram {
        pattern.chars().filter(|c| c.is_ascii_uppercase()).count()
    } else {
        pattern.chars().filter(|c| *c == '?').count()
    };
    let read_fst_file: bool = pattern_question_marks < FST_QUESTION_MARK_THRESHOLD;
    let json_results = if read_fst_file {
        let mmap = unsafe {
            Mmap::map(
                &File::open(find_processed_file("word_frequency.fst"))
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?
        };
        let map = fst::Map::new(mmap).map_err(|e| e.to_string())?;
        // need to strip off the ^ and $, but setting anchored to true will cover that
        let word_regex_pattern = &word_regex.as_str()[1..word_regex.as_str().len() - 1];
        let dfa = dense::Builder::new()
            .anchored(true)
            .build(word_regex_pattern)
            .unwrap();
        let mut results = map
            .search(&dfa)
            .into_stream()
            .into_str_vec()
            .map_err(|e| e.to_string())?;
        if mode == PatternMode::Cryptogram {
            results = results
                .into_iter()
                .filter(|a| is_valid_string(&a.0, &pattern, &mode))
                .collect();
        }
        results.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        results
            .iter()
            .map(|entry| json::object! { "word" => entry.0.clone(), "frequency" => entry.1 })
            .collect()
    } else {
        let mut results = vec![];
        let mut line = String::new();
        let file =
            File::open(find_processed_file("word_frequency.txt")).map_err(|e| e.to_string())?;
        let mut reader = io::BufReader::new(file);
        while reader.read_line(&mut line).map_err(|e| e.to_string())? > 0 {
            let mut parts = line.split_ascii_whitespace();
            let word = parts.next().unwrap();
            if word_regex.is_match(word) {
                if !(mode == PatternMode::Cryptogram && is_valid_string(&word, &pattern, &mode)) {
                    // TODO - use Object constructor
                    results.push(json::object! { "word" => word, "frequency" => parts.next().unwrap().parse::<u64>().unwrap() });
                }
            }
            line.clear();
        }
        results
    };
    Ok(json::JsonValue::Array(json_results))
}

fn build_regex(pattern: &str, absent_letters: &str, mode: &PatternMode) -> Result<Regex, String> {
    // (?-u) turns off unicode, although that's not really necessary here since we're already
    // specifying the exact characters to match.
    let mut regex_str = "^(?-u)".to_string();
    match mode {
        &PatternMode::WheelOfFortune => {
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
            regex_str.push_str(
                &pattern
                    .to_ascii_lowercase()
                    .replace("?", absent_letter_builder.as_str()),
            );
        }
        &PatternMode::Crossword => {
            let mut absent_letter_builder = "[".to_string();
            for letter in 'a'..='z' {
                absent_letter_builder.push(letter);
            }
            absent_letter_builder.push(']');
            regex_str.push_str(
                &pattern
                    .to_ascii_lowercase()
                    .replace("?", absent_letter_builder.as_str()),
            );
        }
        &PatternMode::Cryptogram => {
            let mut known_letters: HashSet<char> = pattern
                .chars()
                .filter(|c| !c.is_ascii_uppercase())
                .collect();
            for letter in absent_letters.chars() {
                //println!("inserting {} from absent", letter);
                known_letters.insert(letter.to_ascii_lowercase());
            }
            for pattern_char in pattern.chars() {
                if !pattern_char.is_ascii_uppercase() {
                    regex_str.push(pattern_char);
                } else {
                    regex_str.push('[');
                    for letter in 'a'..='z' {
                        // cryptogram rules - can't match this character
                        if !(known_letters.contains(&letter)
                            || letter == pattern_char.to_ascii_lowercase())
                        {
                            regex_str.push(letter);
                        }
                    }
                    regex_str.push(']');
                }
            }
        }
    }
    regex_str.push('$');
    Regex::new(&regex_str).map_err(|e| e.to_string())
}

fn is_allowed_char(c: char, mode: &PatternMode) -> bool {
    if c.is_ascii_alphabetic() || c == '\'' || c == '-' {
        return true;
    }
    return mode != &PatternMode::Cryptogram && c == '?';
}

fn validate_pattern(pattern: &str, mode: &PatternMode) -> Result<(), String> {
    if pattern.chars().any(|c| !is_allowed_char(c, &mode)) {
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
        let result =
            process_query_string("mode=WheelOfFortune&pattern=t?e&absent_letters=").unwrap();
        assert_eq!("the", result[0]["word"].to_string());
        assert_ne!(1, result.len());
    }

    #[test]
    fn test_single_letter_missing_and_not_the() {
        let result =
            process_query_string("mode=WheelOfFortune&pattern=t?e&absent_letters=h").unwrap();
        assert_eq!("tie", result[0]["word"].to_string());
        assert_ne!(1, result.len());
    }

    #[test]
    fn test_single_letter_missing_and_not_the_with_duplicate_absent_letters() {
        let result =
            process_query_string("mode=WheelOfFortune&pattern=t?e&absent_letters=ht").unwrap();
        assert_eq!("tie", result[0]["word"].to_string());
        assert_ne!(1, result.len());
    }

    #[test]
    fn test_single_letter_missing_and_not_the_with_extra_duplicate_absent_letters() {
        let result =
            process_query_string("mode=WheelOfFortune&pattern=t?e&absent_letters=htht").unwrap();
        assert_eq!("tie", result[0]["word"].to_string());
        assert_ne!(1, result.len());
    }

    #[test]
    fn test_no_letters_missing() {
        let result =
            process_query_string("mode=WheelOfFortune&pattern=is&absent_letters=").unwrap();
        assert_eq!("is", result[0]["word"].to_string());
        assert_eq!(1, result.len());
    }

    #[test]
    fn test_no_letters_missing_with_absent_letters() {
        let result =
            process_query_string("mode=WheelOfFortune&pattern=is&absent_letters=abc").unwrap();
        assert_eq!("is", result[0]["word"].to_string());
        assert_eq!(1, result.len());
    }

    #[test]
    fn test_all_results_right_length_and_descending_frequency() {
        let result =
            process_query_string("mode=WheelOfFortune&pattern=t???&absent_letters=h").unwrap();
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
    fn test_no_reuse_letters_in_pattern_for_wheeloffortune() {
        let result =
            process_query_string("mode=WheelOfFortune&pattern=t?e?&absent_letters=").unwrap();
        let words = result
            .members()
            .map(|x| x["word"].to_string())
            .collect::<Vec<String>>();
        assert!(!words.contains(&"tree".to_string()));
    }

    #[test]
    fn test_reuse_letters_in_pattern_for_crossword() {
        let result = process_query_string("mode=Crossword&pattern=t?e?&absent_letters=").unwrap();
        let words = result
            .members()
            .map(|x| x["word"].to_string())
            .collect::<Vec<String>>();
        assert!(words.contains(&"tree".to_string()));
    }

    #[test]
    fn test_apostrophe() {
        let result =
            process_query_string("mode=WheelOfFortune&pattern=c??'t&absent_letters=").unwrap();
        let words = result
            .members()
            .map(|x| x["word"].to_string())
            .collect::<Vec<String>>();
        assert!(words.contains(&"can't".to_string()));
    }

    #[test]
    fn test_apostrophe_crossword() {
        let result = process_query_string("mode=Crossword&pattern=c??'t&absent_letters=").unwrap();
        let words = result
            .members()
            .map(|x| x["word"].to_string())
            .collect::<Vec<String>>();
        assert!(words.contains(&"can't".to_string()));
    }

    #[test]
    fn test_apostrophe_not_filled_in() {
        let result =
            process_query_string("mode=WheelOfFortune&pattern=d???t&absent_letters=h").unwrap();
        let words = result
            .members()
            .map(|x| x["word"].to_string())
            .collect::<Vec<String>>();
        assert!(!words.contains(&"don't".to_string()));
    }

    #[test]
    fn test_dash() {
        let result =
            process_query_string("mode=WheelOfFortune&pattern=n?n-?e??er&absent_letters=t")
                .unwrap();
        let words = result
            .members()
            .map(|x| x["word"].to_string())
            .collect::<Vec<String>>();
        assert!(words.contains(&"non-ledger".to_string()));
    }

    #[test]
    fn test_dash_not_filled_in() {
        let result =
            process_query_string("mode=WheelOfFortune&pattern=n?n??e??er&absent_letters=t")
                .unwrap();
        let words = result
            .members()
            .map(|x| x["word"].to_string())
            .collect::<Vec<String>>();
        assert!(!words.contains(&"non-ledger".to_string()));
    }

    #[test]
    fn test_all_results_right_length_with_missing_first_letter() {
        let result =
            process_query_string("mode=WheelOfFortune&pattern=??i?&absent_letters=h").unwrap();
        assert!(result.len() > 3);
        for i in 0..result.len() {
            assert_eq!(4, result[i]["word"].to_string().len());
            assert_eq!('i', result[i]["word"].to_string().chars().nth(2).unwrap());
            assert_ne!("this", result[i]["word"].to_string());
        }
    }

    #[test]
    fn test_giant_set_of_results_right_length_and_descending_frequency() {
        let result =
            process_query_string("mode=WheelOfFortune&pattern=?????&absent_letters=hx").unwrap();
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
        let query = format!("mode=WheelOfFortune&pattern={}&absent_letters=h", too_long);
        let result = process_query_string(&query);
        assert!(!result.is_ok());
    }

    #[test]
    fn test_cryptogram_no_known() {
        let query = format!("mode=Cryptogram&pattern=ABC&absent_letters=");
        let result = process_query_string(&query).unwrap();
        assert_eq!("the", result[0]["word"].to_string());
    }

    #[test]
    fn test_cryptogram_no_known_but_letter_does_not_match() {
        let query = format!("mode=Cryptogram&pattern=TBC&absent_letters=");
        let result = process_query_string(&query).unwrap();
        // can't be "the" because T can't map to t
        assert_eq!("and", result[0]["word"].to_string());
    }

    #[test]
    fn test_cryptogram_no_known_with_repeated_letters() {
        let query = format!("mode=Cryptogram&pattern=ABCC&absent_letters=");
        let result = process_query_string(&query).unwrap();
        assert_eq!("will", result[0]["word"].to_string());
    }

    #[test]
    fn test_cryptogram_a_few_known() {
        let query = format!("mode=Cryptogram&pattern=XBch&absent_letters=");
        let result = process_query_string(&query).unwrap();
        assert_eq!("such", result[0]["word"].to_string());
    }

    #[test]
    fn test_cryptogram_do_not_reuse_letters() {
        let query = format!("mode=Cryptogram&pattern=XBCt&absent_letters=");
        let result = process_query_string(&query).unwrap();
        // not "that" because t is already used
        assert_eq!("what", result[0]["word"].to_string());
    }

    #[test]
    fn test_cryptogram_a_few_absent() {
        let query = format!("mode=Cryptogram&pattern=ABC&absent_letters=ea");
        let result = process_query_string(&query).unwrap();
        assert_eq!("for", result[0]["word"].to_string());
    }

    #[test]
    fn test_invalid_mode() {
        let result = process_query_string("mode=NotARealMode&pattern=t??&absent_letters=h");
        assert!(!result.is_ok());
    }

    #[test]
    fn test_missing_mode() {
        let result = process_query_string("pattern=t??&absent_letters=h");
        assert!(!result.is_ok());
    }
}
