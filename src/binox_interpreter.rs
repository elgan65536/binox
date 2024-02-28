use std::fs;
use std::io;

use colored::Colorize;

use crate::binox::Binox;
use crate::binox::BinoxCell;
use crate::binox::BinoxSolution;

pub enum BIR {
    Normal(bool),
    Error(String),
    Exit,
    Next,
    Previous,
    Import(String),
}

pub fn interpret(mut binox: Binox, line: String) -> (Binox, BIR) {
    let words: Vec<&str> = line.split(' ').collect();
    if words.is_empty() {
        return (binox, BIR::Error("you must enter text".into()));
    }
    match words[0].to_lowercase().as_str() {
        "h" | "help" => {
            println!(
                "\n
{}

Rules:
Fill the board with {x}'s and {o}'s such that the following conditions are met:
1. Each row and column must have the same number of {x}'s as {o}'s.
2. No row or column may contain three consecutive identical symbols.
3. Each row must be unique and each column must be unique.
All cells must be filled. Each puzzle has exactly one solution.

Commands:
x (row) (column): sets an {x} in the specified cell.
o (row) (column): sets an {o} in the specified cell.
erase (row) (column): erases the specified cell.
clear: resets the puzzle to its original state.
verify: tells you whether any rules have been broken so far.
solve: solves the puzzle.
new (size): creates a blank puzzle of the specified size.
generate (size) [perfect] [extras]: generates a puzzle of the specified size with exactly one solution.
 - If perfect is specified, the puzzle will have no unnecessary clues but will take longer to generate.
 - If extras is specified, the puzzle will have extra clues equal to the specified number.
import (file name): imports puzzles from the specified file.
next: saves progress on the current puzzle and moves to the next puzzle.
previous: saves progress on the current puzzle and moves to the previous puzzle.
help: displays this list.
exit: exits the program.",
                "BINOX".bold().underline(),
                x="X".red().bold(),
                o="O".blue().bold(),
            );
            (binox, BIR::Normal(false))
        }
        "x" => {
            if words.len() < 3 {
                return (
                    binox,
                    BIR::Error("command 'x' requires arguments for row and column".into()),
                );
            };
            let col: u8 = match words[1].parse() {
                Ok(a) => a,
                Err(_) => return (binox, BIR::Error("column must be an integer".into())),
            };
            let row: u8 = match words[2].parse() {
                Ok(a) => a,
                Err(_) => return (binox, BIR::Error("row must be an integer".into())),
            };
            let result = binox.set_cell(row, col, BinoxCell::X);
            let result_text = match result {
                Ok(_) => BIR::Normal(true),
                Err(s) => BIR::Error(s.into()),
            };
            (binox.clone(), result_text)
        }
        "o" => {
            if words.len() < 3 {
                return (
                    binox,
                    BIR::Error("command 'o' requires arguments for row and column".into()),
                );
            };
            let col: u8 = match words[1].parse() {
                Ok(a) => a,
                Err(_) => return (binox, BIR::Error("column must be an integer".into())),
            };
            let row: u8 = match words[2].parse() {
                Ok(a) => a,
                Err(_) => return (binox, BIR::Error("row must be an integer".into())),
            };
            let result = binox.set_cell(row, col, BinoxCell::O);
            let result_text = match result {
                Ok(_) => BIR::Normal(true),
                Err(s) => BIR::Error(s.into()),
            };
            (binox.clone(), result_text)
        }
        "e" | "empty" | "erase" => {
            if words.len() < 3 {
                return (
                    binox,
                    BIR::Error("command 'erase' requires arguments for row and column".into()),
                );
            };
            let col: u8 = match words[1].parse() {
                Ok(a) => a,
                Err(_) => return (binox, BIR::Error("column must be an integer".into())),
            };
            let row: u8 = match words[2].parse() {
                Ok(a) => a,
                Err(_) => return (binox, BIR::Error("row must be an integer".into())),
            };
            let result = binox.set_cell(row, col, BinoxCell::EMPTY);
            let result_text = match result {
                Ok(_) => BIR::Normal(true),
                Err(s) => BIR::Error(s.into()),
            };
            (binox.clone(), result_text)
        }
        "c" | "clear" | "reset" => {
            binox.reset();
            (binox, BIR::Normal(true))
        }
        "v" | "check" | "verify" => {
            match (binox.is_full(), binox.is_valid()) {
                (true, true) => println!("{}", "the puzzle has been solved".green().bold()),
                (false, true) => println!("{}", "no mistakes so far".yellow().bold()),
                (_, false) => println!("{}", "a mistake has been made".red().bold()),
            };
            (binox, BIR::Normal(true))
        }
        "p" | "presolve" => {
            binox.presolve();
            (binox, BIR::Normal(true))
        }
        "s" | "solve" => match binox.solve(true) {
            BinoxSolution::Zero => (binox, BIR::Error("puzzle has no solution".into())),
            BinoxSolution::One(a) => (a, BIR::Normal(true)),
            BinoxSolution::Multiple(a, _) => {
                println!("{}", "multiple solutions found".yellow().bold());
                (a, BIR::Normal(true))
            }
        },
        "n" | "new" => {
            if words.len() < 2 {
                return (
                    binox,
                    BIR::Error("command 'new' requires argument for size".into()),
                );
            };
            let size: u8 = match words[1].parse() {
                Ok(num) => num,
                Err(_) => return (binox, BIR::Error("size must be an integer".into())),
            };
            match Binox::new(size) {
                Ok(binox) => (binox, BIR::Normal(true)),
                Err(s) => (binox, BIR::Error(s.into())),
            }
        }
        "g" | "generate" => {
            if words.len() < 2 {
                return (
                    binox,
                    BIR::Error("command 'generate' requires argument for size".into()),
                );
            };
            let size: u8 = match words[1].parse() {
                Ok(num) => num,
                Err(_) => return (binox, BIR::Error("size must be an integer".into())),
            };
            let extras = if words.len() > 2 {
                words[2].parse().unwrap_or(0)
            } else {
                0
            };
            let perfect = (words.len() > 3
                && (words[3].to_lowercase() == "perfect" || words[3].to_lowercase() == "p"))
                || (words.len() > 2
                    && (words[2].to_lowercase() == "perfect" || words[2].to_lowercase() == "p"));
            if perfect {
                println!("generating perfect")
            }
            match Binox::generate(size, perfect, extras) {
                Ok(binox) => (binox, BIR::Normal(true)),
                Err(s) => (binox, BIR::Error(s.into())),
            }
        }
        "i" | "import" | "l" | "load" | "open" => {
            if words.len() < 2 {
                return (
                    binox,
                    BIR::Error("command 'import' requires argument for file name".into()),
                );
            };
            (binox, BIR::Import(words[1].into()))
        }
        "ne" | "next" => (binox, BIR::Next),
        "pr" | "prev" | "previous" => (binox, BIR::Previous),
        "exit" => (binox, BIR::Exit),
        _ => (binox, BIR::Error("invalid command".into())),
    }
}

pub fn run_interpreter() {
    let mut binox = Binox::generate(8, true, 0).unwrap();
    let mut puzzles: Vec<String> = vec![binox.as_string(), "            ".into()];
    let mut selected_puzzle = 0;
    println!("{}", binox);
    loop {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        let input: String = input.trim().into();
        let (new_binox, result) = interpret(binox, input);
        binox = new_binox;
        match result {
            BIR::Normal(print) => {
                if print {
                    println!("{}", binox)
                }
            }
            BIR::Exit => {
                println!("{}", "Exiting the program".yellow().bold());
                break;
            }
            BIR::Next => {
                puzzles[selected_puzzle] = binox.as_string();
                selected_puzzle = if selected_puzzle >= puzzles.len() - 1 {
                    0
                } else {
                    selected_puzzle + 1
                };
                binox = Binox::new_from_string(puzzles[selected_puzzle].clone());
                println!("{}", binox);
            }
            BIR::Previous => {
                puzzles[selected_puzzle] = binox.as_string();
                selected_puzzle = if selected_puzzle == 0 {
                    puzzles.len() - 1
                } else {
                    selected_puzzle - 1
                };
                binox = Binox::new_from_string(puzzles[selected_puzzle].clone());
                println!("{}", binox);
            }
            BIR::Import(mut filename) => {
                if !filename.contains('.') {
                    filename.push_str(".binox")
                }
                if let Ok(contents) = fs::read_to_string(filename.clone()) {
                    let lines: Vec<&str> = contents.lines().collect::<Vec<&str>>();
                    let lines: Vec<String> = lines.iter().map(|str| str.to_string()).collect();
                    if lines.is_empty() {
                        println!("file contains no puzzles");
                    } else {
                        puzzles = lines;
                        selected_puzzle = 0;
                        binox = Binox::new_from_string(puzzles[0].clone());
                        println!("{}", binox);
                    }
                } else {
                    println!("{} {}", "file not found:".red().bold(), filename);
                };
            }
            BIR::Error(text) => println!("{}", text.red().bold()),
        }
    }
}
