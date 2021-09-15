extern crate cgi;
extern crate json;
extern crate url;

use std::{collections::{HashMap, HashSet}, fs::File, io::{self, BufRead}, path::Path};
use regex::Regex;

fn process_query_string(query: &str) -> Result<json::JsonValue, String> {
    let query_parts: HashMap<String, String> = url::form_urlencoded::parse(query.as_bytes()).into_owned().collect();
    let pattern = query_parts.get("pattern").ok_or(String::from("Internal error - no pattern specified!"))?;
    validate_pattern(pattern)?;
    let absent_letters = query_parts.get("absent_letters").ok_or(String::from("Internal error - no absent_letters specified!"))?;
    validate_absent_letters(absent_letters)?;
    let word_regex = build_regex(pattern, absent_letters)?;
    let lines = read_lines(Path::new("../data/processed/word_frequency.txt")).map_err(|e| e.to_string())?;
    let mut results = vec![];
    for line in lines {
        let line = line.map_err(|e| e.to_string())?;
        let mut parts = line.split_ascii_whitespace();
        let word = parts.next().unwrap();
        if word_regex.is_match(word){
            // TODO - use Object constructor
            results.push(json::object! { "word" => word, "frequency" => parts.next().unwrap().parse::<u64>().unwrap() });
        }
    }
    Ok(json::JsonValue::Array(results))
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
    let mut absent_letter_builder = "[^-'".to_string();
    for absent_letter in absent_letter_set {
        absent_letter_builder.push(absent_letter);
    }
    absent_letter_builder.push(']');

    let mut regex_str = "^".to_string();
    regex_str.push_str(&pattern.to_ascii_lowercase().replace("?", absent_letter_builder.as_str()));
    regex_str.push('$');
    //println!("{}", regex_str);
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

fn error(s: &str) -> cgi::Response {
    cgi::binary_response(200, "application/json", (json::object!{"error": s.clone()}).dump().as_bytes().to_vec())
}

fn json_response_cross_origin(body: Vec<u8>) -> cgi::Response {
    let mut response = cgi::http::response::Builder::new()
        .status(200)
        .header(cgi::http::header::CONTENT_LENGTH, format!("{}", body.len()).as_str());
    response = response.header(cgi::http::header::CONTENT_TYPE, "application/json");
    response = response.header(cgi::http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*");

    response.body(body).unwrap()
}

fn success(s: json::JsonValue) -> cgi::Response {
    json_response_cross_origin(s.dump().as_bytes().to_vec())
}

// https://doc.rust-lang.org/stable/rust-by-example/std_misc/file/read_lines.html
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn process_request(request: &cgi::Request) -> Result<json::JsonValue, String> {
    let query = request.uri().query().ok_or(String::from("Internal error - no query string?"))?;
    return process_query_string(query);
}

cgi::cgi_main! { |request: cgi::Request| {
    let result = process_request(&request);
    match result {
        Ok(val) => success(val),
        Err(err) => error(&err)
    }
} }


mod tests {
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

