use std::fs::File;
use std::collections::HashMap;
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
        filters.push(ResultFilter::new(guess.unwrap()));
        words = words.into_iter().filter(|w| filters.iter().all(|f| f.matches(w))).collect();
        if words.len() == 1 {
            println!("Last word remaining! {}", words[0]);
            break;
        }
    }
    
    Ok(())
}

struct ResultFilter {
    word: String,
    result: String,
}

impl ResultFilter {
    fn new(input: (String, String)) -> ResultFilter {
        ResultFilter {
            word: input.0,
            result: input.1,
        }
    }

    fn matches(&self, candidate: &str) -> bool {
        if candidate == self.word {
            return self.result == "ggggg";
        }
        for (i, c) in self.result.chars().enumerate() {
            let mut counts = HashMap::new();
            let expected = self.word.chars().nth(i).unwrap();
            *counts.entry(expected).or_insert(0) += 1;
            match c {
                'g' => {
                    if candidate.chars().nth(i).unwrap() != expected {
                        return false;
                    }
                },
                'y' => {
                    if candidate.chars().nth(i).unwrap() == expected {
                        return false;
                    }
                    if candidate.chars().filter(|&c| c == expected).count() < *counts.get(&expected).unwrap() {
                        return false;
                    }
                },
                _ => {
                    if candidate.chars().filter(|&c| c == expected).count() > 0 {
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
    let guess = get_input("What was your guess? ", |s| {
        words.contains(s)
    })?;
    if let None = guess {
        println!("Invalid guess");
        return Ok(None);
    }

    let result = get_input("What was the result (format 'y..gg')? ", |s| {
        s.len() == 5
    })?;
    if let None = result {
        println!("Invalid result");
        return Ok(None);
    }

    Ok(Some((guess.unwrap().to_ascii_lowercase(), result.unwrap().to_ascii_lowercase())))
}

fn get_input<T>(prompt: &str, valid_predicate: T) -> Result<Option<String>, io::Error> 
where T: Fn(&String) -> bool {
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