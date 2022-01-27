use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};

fn main() -> Result<(), io::Error> {
    let mut words = get_words()?;
    let mut filters = Vec::new();

    loop {
        output_status(&words);
        let guess = get_guess(&words)?;
        if let None = guess {
            continue;
        }
        let (word, result) = guess.unwrap();
        if result == "ggggg" {
            println!("You won!");
        }
        filters.push(ResultFilter { word, result });
        words = get_matching_words(&words, &filters);
        match words.len() {
            0 => {
                println!("No words remaining. You... lose?");
                break;
            }
            1 => {
                println!("You win! The word is {}", words[0]);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

fn get_matching_words(words: &Vec<String>, filters: &Vec<ResultFilter>) -> Vec<String> {
    words
        .iter()
        .filter(|w| filters.iter().all(|f| f.matches(w)))
        .cloned()
        .collect()
}

struct ResultFilter {
    word: String,
    result: String,
}

impl ResultFilter {
    fn matches(&self, candidate: &str) -> bool {
        if candidate == self.word {
            return self.result == "ggggg";
        }
        let mut counts = HashMap::new();
        for (i, c) in self.result.chars().enumerate() {
            let expected = self.word.chars().nth(i).unwrap();
            *counts.entry(expected).or_insert(0) += 1;
            match c {
                'g' => {
                    if candidate.chars().nth(i).unwrap() != expected {
                        return false;
                    }
                }
                'y' => {
                    if candidate.chars().nth(i).unwrap() == expected {
                        return false;
                    }
                    if candidate.chars().filter(|&c| c == expected).count()
                        < *counts.get(&expected).unwrap()
                    {
                        return false;
                    }
                }
                _ => {
                    if candidate.chars().nth(i).unwrap() == expected {
                        return false;
                    }
                    if candidate.chars().filter(|&c| c == expected).count()
                        > *counts.entry(expected).or_insert(0)
                    {
                        return false;
                    }
                }
            }
        }
        true
    }
}

fn output_status(words: &Vec<String>) {
    println!("There are {} possible words left", words.len());
    let mut i = 0;
    for word in words {
        print!("{}\t", word);
        i += 1;
        if i % 12 == 0 {
            println!();
        }
    }
    println!("... go guess one!\n");
}

fn get_guess(words: &Vec<String>) -> Result<Option<(String, String)>, io::Error> {
    let guess = get_input("What was your guess?", |s| words.contains(s))?;
    if let None = guess {
        println!("That's not in the word list!");
        return Ok(None);
    }

    let result = get_input("What was the result (format 'y..gg')?", |s| s.len() == 5)?;
    if let None = result {
        println!("That doesn't match the format expected! 5 characters, g or y or anything else.");
        return Ok(None);
    }

    Ok(Some((
        guess.unwrap().to_ascii_lowercase(),
        result.unwrap().to_ascii_lowercase(),
    )))
}

fn get_result(guess: &str, word: &str) -> String {
    let mut result = String::new();
    let mut counts = HashMap::new();
    for (i, c) in guess.chars().enumerate() {
        let expected = guess.chars().nth(i).unwrap();
        *counts.entry(expected).or_insert(0) += 1;

        if c == word.chars().nth(i).unwrap() {
            result.push('g');
        } else if word.chars().filter(|&c| c == expected).count() >= *counts.get(&expected).unwrap()
        {
            result.push('y');
        } else {
            result.push('.');
        }
    }
    result
}

fn get_input<T>(prompt: &str, valid_predicate: T) -> Result<Option<String>, io::Error>
where
    T: Fn(&String) -> bool,
{
    println!("{}", prompt);
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input)?;
    input = input.trim().to_string();
    if valid_predicate(&input) {
        Ok(Some(input))
    } else {
        Ok(None)
    }
}

pub fn get_words() -> Result<Vec<String>, io::Error> {
    let file = File::open("words.txt")?;
    let lines = io::BufReader::new(file).lines();
    lines.collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches() {
        let filter = ResultFilter {
            word: "socko".to_string(),
            result: "gg...".to_string(),
        };
        assert!(!filter.matches("socko"));
        assert!(filter.matches("soare"));
        assert!(filter.matches("songs"));
    }

    #[test]
    fn test_get_matching_words() {
        let make_words = |v: Vec<&str>| {
            v.into_iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        };
        let filter = |word: &str, result: &str| ResultFilter {
            word: word.to_string(),
            result: result.to_string(),
        };

        let mut filters = Vec::new();

        let words = make_words(vec!["soare", "socko", "songs", "socks"]);
        filters.push(filter("soare", "gg..."));
        let expected = make_words(vec!["socko", "songs", "socks"]);
        assert_eq!(expected, get_matching_words(&words, &filters));

        filters.push(filter("socko", "gg..."));
        let expected = make_words(vec!["songs"]);
        assert_eq!(expected, get_matching_words(&words, &filters));
    }

    #[test]
    fn test_get_result() {
        let mut words = HashMap::new();
        let mut cases = HashMap::new();
        cases.insert("llama", "yy...");
        cases.insert("lards", "y....");
        cases.insert("chalk", "...gy");
        cases.insert("knoll", "ggggg");
        words.insert("knoll", cases);

        let mut cases = HashMap::new();
        cases.insert("soare", "gg...");
        cases.insert("socko", "gg...");
        words.insert("songs", cases);

        for (word, cases) in words {
            for (guess, result) in cases {
                assert_eq!(
                    get_result(guess, word),
                    result,
                    "guess: {} word: {}",
                    guess,
                    word
                );
            }
        }
    }
}
