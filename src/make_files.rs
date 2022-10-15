use crate::binox::Binox;
use std::{fs::File, io::Write};

pub fn create_binox_file(name: &str, size: u8, perfect: bool, extras: usize, amount: u32) {
    let mut file = File::create(format!("{size}x{size}_{name}.binox")).unwrap();
    for _ in 0..amount {
        let binox = Binox::generate(size, perfect, extras).unwrap();
        file.write_all(binox.as_string().as_bytes())
            .expect("fail to write");
        file.write_all("\n".as_bytes()).expect("fail to write");
    }
}

pub fn create_default_files() {
    let expert = [0, 0, 0, 0, 0, 0, 0];
    let hard = [1, 1, 2, 3, 4, 5, 6];
    let medium = [2, 2, 4, 6, 8, 10, 12];
    let easy = [3, 3, 6, 9, 12, 15, 18];
    let sizes = [4, 6, 8, 10, 12, 14, 16];
    for (i, &size) in sizes.iter().enumerate() {
        create_binox_file("easy", size, true, easy[i], 32);
        create_binox_file("medium", size, true, medium[i], 32);
        create_binox_file("hard", size, true, hard[i], 32);
        create_binox_file("expert", size, true, expert[i], 32);
    }
}
