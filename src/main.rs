#![feature(test)]

extern crate test;
use rand::seq::SliceRandom;
use std::cmp::min;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};

fn main() -> Result<(), io::Error> {
    let mut words = get_words()?;

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
        let filter = ResultFilter::new_owned(word, result);
        words = get_matching_words(&words, &filter)
            .into_iter()
            .cloned()
            .collect();
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

fn get_matching_words<'a>(words: &'a Vec<String>, filter: &ResultFilter) -> Vec<&'a String> {
    words.iter().filter(|w| filter.matches(w)).collect()
}

fn get_best_guess<'a>(words: &'a Vec<String>) -> Vec<&'a String> {
    let mut guesses = HashMap::new();
    for guess in words {
        for word in words {
            let result = get_result(guess, word);
            let filter = ResultFilter::new_borrowed(guess, result);
            let num_matches = get_matching_words(words, &filter).len();
            *guesses.entry(word).or_insert(0) += num_matches;
        }
    }
    struct Guess<'a> {
        word: &'a String,
        score: usize,
    }
    let mut guesses = guesses
        .iter()
        .map(|(word, score)| Guess {
            word: *word,
            score: *score,
        })
        .collect::<Vec<_>>();
    guesses.sort_by(|a, b| a.score.cmp(&b.score));
    guesses.iter().map(|g| g.word).collect()
}

#[derive(Clone, Debug)]
enum ResultWord<'a> {
    Borrowed(&'a str),
    Owned(String),
}

#[derive(Clone, Debug)]
struct ResultFilter<'a> {
    word: ResultWord<'a>,
    result: String,
}

impl<'a> ResultFilter<'a> {
    fn new_owned(word: String, result: String) -> ResultFilter<'a> {
        ResultFilter {
            word: ResultWord::Owned(word),
            result: result,
        }
    }

    fn new_borrowed(word: &'a str, result: String) -> ResultFilter<'a> {
        ResultFilter {
            word: ResultWord::Borrowed(word),
            result: result,
        }
    }

    fn matches(&self, candidate: &str) -> bool {
        let word = match self.word {
            ResultWord::Borrowed(w) => w,
            ResultWord::Owned(ref w) => w,
        };
        if candidate == word {
            return self.result == "ggggg";
        }
        let mut counts = HashMap::new();
        for (i, c) in self.result.chars().enumerate() {
            let expected = word.chars().nth(i).unwrap();
            match c {
                'g' => {
                    *counts.entry(expected).or_insert(0) += 1;
                    if candidate.chars().nth(i).unwrap() != expected {
                        return false;
                    }
                }
                'y' => {
                    *counts.entry(expected).or_insert(0) += 1;
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
    const MAX: usize = 500;
    const NUM_SAMPLES: usize = 48;

    println!("\nThere are {} possible words left...", words.len());
    if words.len() > MAX {
        println!("That's too many to brute force good guesses... here are some random ones:");
        let sample = words.choose_multiple(&mut rand::thread_rng(), NUM_SAMPLES);
        let mut i = 0;
        for word in sample {
            print!("{}\t", word);
            i += 1;
            if i % 12 == 0 {
                println!();
            }
        }
    } else {
        println!("Guesses that narrow it down the most are:");
        let words = get_best_guess(words);
        let mut i = 0;
        for word in &words[0..min(words.len(), NUM_SAMPLES)] {
            print!("{}\t", word);
            i += 1;
            if i % 12 == 0 {
                println!();
            }
        }
    }
    println!("\n... go guess one!\n");
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
    use test::Bencher;

    #[test]
    fn test_matches() {
        let filter = ResultFilter::new_owned("socko".to_string(), "gg...".to_string());
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
        let make_filter = |word: &str, result: &str| {
            ResultFilter::new_owned(word.to_string(), result.to_string())
        };

        let words = make_words(vec!["soare", "socko", "songs", "socks"]);
        let filter = make_filter("soare", "gg...");
        let expected = vec!["socko", "songs", "socks"];
        assert_eq!(expected, get_matching_words(&words, &filter));

        let words = make_words(vec!["socko", "songs", "socks"]);
        let filter = make_filter("socko", "gg...");
        let expected = vec!["songs"];
        assert_eq!(expected, get_matching_words(&words, &filter));
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

    #[bench]
    fn bench_get_best_guess(b: &mut Bencher) {
        let words = vec![
            "roset", "rosed", "rotes", "roles", "rotor", "rones", "roosa", "noser", "rodes",
            "robes", "eorls", "tolar", "motor", "rohes", "loser", "ropes", "doser", "roted",
            "rotas", "rokes", "royst", "poser", "roues", "hoser", "ronts", "boyar", "douar",
            "rotls", "tores", "rotan", "rores", "donor", "dorsa", "roves", "dolor", "rosti",
            "roost", "romeo", "rosit", "yores", "rosin", "roist", "robed", "dowar", "rodeo",
        ];
        let words = words
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        b.iter(|| get_best_guess(&words));
    }
}
