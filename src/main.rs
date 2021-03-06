#![feature(test)]

extern crate test;
use rand::seq::SliceRandom;
use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead};

fn main() -> Result<(), io::Error> {
    let mut words = get_words()?;
    let usage_frequency = get_word_frequencies()?;

    loop {
        output_status(&words, &usage_frequency);
        let guess = get_guess()?;
        if let None = guess {
            continue;
        }
        let (word, result) = guess.unwrap();
        if result == "ggggg" {
            println!("You won!");
            break;
        }
        let filter = ResultFilter::new_owned(word, result);
        words = get_matching_words(&words, &filter)
            .into_iter()
            .cloned()
            .collect();
        match words.len() {
            0 => {
                println!("No words remaining. You... made typo?");
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

fn get_matching_words<'a>(
    words: &'a Vec<String>,
    filter: &'a ResultFilter,
) -> impl Iterator<Item = &'a String> + 'a {
    words.iter().filter(|w| filter.matches(w))
}

type FrequencyMaps = (HashMap<usize, HashMap<char, usize>>, HashMap<char, usize>);
fn get_letter_frequencies(words: &Vec<String>) -> FrequencyMaps {
    let mut pos_freq = HashMap::new();
    let mut letter_freq = HashMap::new();

    for word in words {
        for (i, c) in word.chars().enumerate() {
            let map = pos_freq.entry(i).or_insert(HashMap::new());
            *map.entry(c).or_insert(0) += 1;
            *letter_freq.entry(c).or_insert(0) += 1;
        }
    }

    (pos_freq, letter_freq)
}

fn get_best_guess<'a>(words: &'a Vec<String>) -> Vec<&'a String> {
    let mut guesses = HashMap::new();
    for guess in words {
        for word in words {
            let result = get_result(guess, word);
            let filter = ResultFilter::new_borrowed(guess, result);
            let num_matches = get_matching_words(words, &filter).count();
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

fn get_best_guess_by_letter_frequency<'a>(
    words: &'a Vec<String>,
    position: bool,
) -> Vec<&'a String> {
    let (pos_freq, letter_freq) = get_letter_frequencies(words);

    let mut guesses = HashMap::new();
    for word in words {
        let mut pos_score = 0;
        let mut letter_score = 0;
        for (i, c) in word.chars().enumerate() {
            pos_score += pos_freq
                .get(&i)
                .unwrap_or(&HashMap::new())
                .get(&c)
                .unwrap_or(&0);
            letter_score += letter_freq.get(&c).unwrap_or(&0);
        }
        guesses.insert(word, (pos_score, letter_score));
    }

    struct Guess<'a> {
        word: &'a String,
        score: usize,
    }

    let mut guesses = guesses
        .iter()
        .map(|(word, score)| Guess {
            word: *word,
            score: if position { score.0 } else { score.1 },
        })
        .collect::<Vec<_>>();
    guesses.sort_by(|a, b| b.score.cmp(&a.score));
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
        // let mut counts = HashMap::new();
        for (i, c) in self.result.chars().enumerate() {
            let char = word.chars().nth(i).unwrap();
            let at_least_count = word
                .chars()
                .zip(self.result.chars())
                .filter(|(w, r)| *w == char && (*r == 'g' || *r == 'y'))
                .count();
            match c {
                'g' => {
                    if candidate.chars().nth(i).unwrap() != char {
                        return false;
                    }
                }
                'y' => {
                    if candidate.chars().nth(i).unwrap() == char {
                        return false;
                    }
                    if candidate.chars().filter(|&c| c == char).count() < at_least_count {
                        return false;
                    }
                }
                _ => {
                    if candidate.chars().nth(i).unwrap() == char {
                        return false;
                    }
                    if candidate.chars().filter(|&c| c == char).count() > at_least_count {
                        return false;
                    }
                }
            }
        }
        true
    }
}

fn output_status(words: &Vec<String>, usage_frequency: &HashMap<String, usize>) {
    const MAX: usize = 400;
    const NUM_SAMPLES: usize = 48;

    fn print(words: &Vec<&String>) {
        let mut i = 0;
        for word in &words[0..min(words.len(), NUM_SAMPLES)] {
            print!("\t{}", word);
            i += 1;
            if i % 12 == 0 {
                println!();
            }
        }
        if i % 12 != 0 {
            println!();
        }
    }

    fn is_unique(word: &str) -> bool {
        let mut seen = HashSet::new();
        for c in word.chars() {
            if seen.contains(&c) {
                return false;
            }
            seen.insert(c);
        }
        true
    }

    let mut scored = HashMap::new();
    let mut score = |words: &Vec<&String>, val: usize| {
        for word in &words[0..min(words.len(), NUM_SAMPLES)] {
            *scored.entry(word.to_string()).or_insert(0) += val;
        }
    };

    println!("\nThere are {} possible words left...", words.len());
    {
        let (pos_freq, _) = get_letter_frequencies(&words);
        println!("Most common letters by position are...");
        for i in 0..5_usize {
            let mut letters = pos_freq[&i].iter().collect::<Vec<_>>();
            letters.sort_by(|a, b| b.1.cmp(&a.1));
            let letters = letters.iter().map(|l| l.0.to_string()).collect::<Vec<_>>();
            println!("\t{}: {}", i, letters.join(", "));
        }
    }

    {
        println!("Highest by english language usage frequency are:");
        let mut words: Vec<_> = words.iter().collect();
        words.sort_by(|a, b| {
            let a_freq = usage_frequency.get(*a).unwrap_or(&0);
            let b_freq = usage_frequency.get(*b).unwrap_or(&0);
            b_freq.cmp(a_freq)
        });
        score(&words, 1);
        print(&words);
    }
    {
        println!("Highest by letter frequency in position are:");
        let words = get_best_guess_by_letter_frequency(words, true);
        score(&words, 1);
        print(&words);
        println!("Without duplicates...");
        let words = words.into_iter().filter(|w| is_unique(w)).collect();
        score(&words, 1);
        print(&words);
    }
    {
        println!("Highest by letter frequency absolutely are:");
        let words = get_best_guess_by_letter_frequency(words, false);
        score(&words, 1);
        print(&words);
        println!("Without duplicates...");
        let words = words.into_iter().filter(|w| is_unique(w)).collect();
        score(&words, 1);
        print(&words);
    }
    if words.len() > MAX {
        println!("That's too many to brute force good guesses... here are some random ones:");
        let sample = words
            .choose_multiple(&mut rand::thread_rng(), NUM_SAMPLES)
            .collect();
        score(&sample, 1);
        print(&sample);
    } else {
        println!("Guesses that narrow it down the most are:");
        let words = get_best_guess(words);
        score(&words, 1);
        print(&words);
    }
    {
        println!("Highest by score:");
        let mut words: Vec<_> = scored.keys().collect();
        words.sort_by(|a, b| {
            let a_score = scored.get(*a).unwrap_or(&0);
            let b_score = scored.get(*b).unwrap_or(&0);
            b_score.cmp(a_score)
        });
        // words.sort_by(|a, b| {
        //     let a_freq = usage_frequency.get(*a).unwrap_or(&0);
        //     let b_freq = usage_frequency.get(*b).unwrap_or(&0);
        //     b_freq.cmp(a_freq)
        // });
        print(&words);
    }

    println!("\n... go guess one!\n");
}

fn get_guess() -> Result<Option<(String, String)>, io::Error> {
    let guess = get_input("What was your guess?")?;
    if let None = guess {
        println!("That's not in the word list!");
        return Ok(None);
    }

    let result = get_input("What was the result (format 'y..gg')?")?;
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
    let mut g_counts = HashMap::new();
    let mut c_counts = HashMap::new();

    for (i, c) in guess.chars().enumerate() {
        let char = guess.chars().nth(i).unwrap();

        if c == word.chars().nth(i).unwrap() {
            result.push('g');
            *g_counts.entry(char).or_insert(0) += 1;
        } else if word.chars().filter(|&c| c == char).count() > 0 {
            result.push('?');
        } else {
            result.push('.');
        }
    }

    let mut final_result = String::new();
    for (i, c) in result.chars().enumerate() {
        if c == '?' {
            let guess_char = guess.chars().nth(i).unwrap();
            let c_count = c_counts.entry(guess_char).or_insert(0);
            let g_count = g_counts.entry(guess_char).or_insert(0);
            let current_total = *c_count + *g_count;
            let expected_count = word.chars().filter(|&c| c == guess_char).count();

            if current_total < expected_count {
                final_result.push('y');
                *c_count += 1;
            } else {
                final_result.push('.');
            }
        } else {
            final_result.push(c);
        }
    }

    final_result
}

fn get_input(prompt: &str) -> Result<Option<String>, io::Error> {
    println!("{}", prompt);
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input)?;
    input = input.trim().to_string();
    if input.len() == 5 {
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

pub fn get_word_frequencies() -> Result<HashMap<String, usize>, io::Error> {
    let file = File::open("unigram_freq.csv")?;
    let lines = io::BufReader::new(file).lines();
    let mut frequencies = HashMap::new();
    for line in lines {
        let line = line?;
        let (word, count) = line.split_once(",").unwrap();
        if word.len() == 5 {
            frequencies.insert(word.to_string(), count.parse::<usize>().unwrap());
        }
    }
    Ok(frequencies)
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

        let mut words = HashMap::new();
        let mut cases = HashMap::new();
        cases.insert("blush", "gy.y.");
        words.insert("balls", cases);

        let mut cases = HashMap::new();
        cases.insert("balls", "g.y.y");
        words.insert("blush", cases);

        let mut cases = HashMap::new();
        cases.insert("soare", "..g.g");
        cases.insert("plane", "..g.g");
        cases.insert("adage", ".yg.g");
        cases.insert("weave", ".ygyg");
        words.insert("evade", cases);

        for (word, cases) in words {
            for (guess, result) in cases {
                let filter = ResultFilter::new_owned(guess.to_string(), result.to_string());
                assert!(
                    filter.matches(word),
                    "filter {} {} should match {}",
                    guess,
                    result,
                    word
                );
            }
        }
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
        assert_eq!(
            expected,
            get_matching_words(&words, &filter).collect::<Vec<_>>()
        );

        let words = make_words(vec!["socko", "songs", "socks"]);
        let filter = make_filter("socko", "gg...");
        let expected = vec!["songs"];
        assert_eq!(
            expected,
            get_matching_words(&words, &filter).collect::<Vec<_>>()
        );
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

        let mut cases = HashMap::new();
        cases.insert("balls", "g.y.y");
        words.insert("blush", cases);

        let mut cases = HashMap::new();
        cases.insert("blush", "gy.y.");
        words.insert("balls", cases);

        let mut cases = HashMap::new();
        cases.insert("soare", "..g.g");
        cases.insert("plane", "..g.g");
        cases.insert("adage", ".yg.g");
        cases.insert("weave", ".ygyg");
        words.insert("evade", cases);

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
