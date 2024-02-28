use core::fmt;
use std::ops::Add;

use crate::binox::row::BinRow;
use crate::binox::BinoxSolution::*;

use colored::*;
use rand::prelude::SliceRandom;
use rand::Rng;

mod row;

#[derive(Clone, Debug)]
pub struct Binox {
    size: u8,
    x_rows: Vec<BinRow>,
    o_rows: Vec<BinRow>,
    x_cols: Vec<BinRow>,
    o_cols: Vec<BinRow>,
    default_rows: Vec<BinRow>,
}

#[derive(PartialEq, Eq)]
pub enum BinoxCell {
    X,
    O,
    EMPTY,
}

pub enum PresolveResult {
    Good,
    Bad,
}

pub enum BinoxSolution {
    Zero,
    One(Binox),
    Multiple(Binox, Binox),
}

impl Add for BinoxSolution {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Multiple(..) => self,
            One(a) => match rhs {
                Zero => One(a),
                One(b) => Multiple(a, b),
                Multiple(..) => rhs,
            },
            Zero => rhs,
        }
    }
}

impl From<BinoxCell> for char {
    fn from(cell: BinoxCell) -> Self {
        match cell {
            BinoxCell::X => 'X',
            BinoxCell::O => 'O',
            BinoxCell::EMPTY => ' ',
        }
    }
}

impl From<BinoxCell> for ColoredString {
    fn from(cell: BinoxCell) -> Self {
        match cell {
            BinoxCell::X => "X".red(),
            BinoxCell::O => "O".blue(),
            BinoxCell::EMPTY => " ".into(),
        }
    }
}

impl Binox {
    pub fn new(size: u8) -> Result<Self, &'static str> {
        if size > 16 {
            return Err("size must be at most 16");
        }
        if size < 4 {
            return Err("size must be at least 4");
        }
        if size % 2 == 1 {
            return Err("size must be even");
        }
        Ok(Binox {
            size,
            x_rows: vec![BinRow::new(size).unwrap(); size.into()],
            o_rows: vec![BinRow::new(size).unwrap(); size.into()],
            x_cols: vec![BinRow::new(size).unwrap(); size.into()],
            o_cols: vec![BinRow::new(size).unwrap(); size.into()],
            default_rows: vec![BinRow::new(size).unwrap(); size.into()],
        })
    }

    pub fn new_from_string(str: String) -> Self {
        let mut size = (str.len() as f64).sqrt().floor() as u8;
        if size < 4 {
            size = 4;
        }
        if size > 16 {
            size = 16;
        }
        if size % 2 == 1 {
            size += 1;
        }
        let mut binox = Binox::new(size).unwrap();
        let (mut i, mut j) = (0, 0);
        for c in str.chars() {
            match c {
                'x' => {
                    binox.set_x(j, i).unwrap();
                }
                'X' => {
                    binox.set_x(j, i).unwrap();
                    binox.set_default(j, i, true).unwrap()
                }
                'o' => {
                    binox.set_o(j, i).unwrap();
                }
                'O' => {
                    binox.set_o(j, i).unwrap();
                    binox.set_default(j, i, true).unwrap()
                }
                _ => (),
            };
            i += 1;
            if i >= size {
                i = 0;
                j += 1;
            }
            if j >= size {
                break;
            }
        }
        binox
    }

    fn set_x(&mut self, row: u8, col: u8) -> Result<(), &'static str> {
        if row >= self.size || col >= self.size {
            return Err("attempted to set x out of range");
        }
        self.x_rows[row as usize].set_one(col).unwrap();
        self.o_rows[row as usize].set_zero(col).unwrap();
        self.x_cols[col as usize].set_one(row).unwrap();
        self.o_cols[col as usize].set_zero(row).unwrap();
        Ok(())
    }

    fn set_o(&mut self, row: u8, col: u8) -> Result<(), &'static str> {
        if row >= self.size || col >= self.size {
            return Err("attempted to set o out of range");
        }
        self.x_rows[row as usize].set_zero(col).unwrap();
        self.o_rows[row as usize].set_one(col).unwrap();
        self.x_cols[col as usize].set_zero(row).unwrap();
        self.o_cols[col as usize].set_one(row).unwrap();
        Ok(())
    }

    fn set_empty(&mut self, row: u8, col: u8) -> Result<(), &'static str> {
        if row >= self.size || col >= self.size {
            return Err("attempted to set empty out of range");
        }
        self.x_rows[row as usize].set_zero(col).unwrap();
        self.o_rows[row as usize].set_zero(col).unwrap();
        self.x_cols[col as usize].set_zero(row).unwrap();
        self.o_cols[col as usize].set_zero(row).unwrap();
        Ok(())
    }

    pub fn set_cell(&mut self, row: u8, col: u8, cell: BinoxCell) -> Result<(), &'static str> {
        if row >= self.size || col >= self.size {
            return Err("attempted to set cell out of range");
        }
        if self.is_default(row, col).unwrap() {
            return Err("this cell cannot be modified.");
        }
        match cell {
            BinoxCell::X => self.set_x(row, col),
            BinoxCell::O => self.set_o(row, col),
            BinoxCell::EMPTY => self.set_empty(row, col),
        }
    }

    fn set_default(&mut self, row: u8, col: u8, value: bool) -> Result<(), &'static str> {
        if row >= self.size || col >= self.size {
            return Err("attempted to set default out of range");
        }
        self.default_rows[row as usize]
            .set_bool(col, value)
            .unwrap();
        Ok(())
    }

    fn get_cell(&self, row: u8, col: u8) -> Result<BinoxCell, &'static str> {
        if row >= self.size || col >= self.size {
            return Err("attempted to get cell out of range");
        }
        match (
            self.x_rows[row as usize].get(col).unwrap(),
            self.o_rows[row as usize].get(col).unwrap(),
        ) {
            (true, false) => Ok(BinoxCell::X),
            (false, true) => Ok(BinoxCell::O),
            _ => Ok(BinoxCell::EMPTY),
        }
    }

    fn is_default(&self, row: u8, col: u8) -> Result<bool, &'static str> {
        if row >= self.size || col >= self.size {
            return Err("attempted to get default out of range");
        }
        Ok(self.default_rows[row as usize].get(col).unwrap())
    }

    pub fn is_valid_simple(&self) -> bool {
        [&self.x_rows, &self.o_cols, &self.x_cols, &self.o_cols]
            .iter()
            .flat_map(|&x| x)
            .all(|row| row.is_valid_simple())
    }

    pub fn is_valid(&self) -> bool {
        if ![&self.x_rows, &self.o_cols, &self.x_cols, &self.o_cols]
            .iter()
            .flat_map(|&x| x)
            .all(|row| row.is_valid())
        {
            return false;
        }
        let mut sorted_x_rows = self.x_rows.clone();
        let mut sorted_o_rows = self.o_rows.clone();
        let mut sorted_x_cols = self.x_cols.clone();
        let mut sorted_o_cols = self.o_cols.clone();
        sorted_x_rows.sort();
        sorted_o_rows.sort();
        sorted_x_cols.sort();
        sorted_o_cols.sort();
        for i in 0..(self.size - 1) {
            if sorted_x_rows[i as usize].data == sorted_x_rows[(i + 1) as usize].data
                && sorted_x_rows[i as usize].count == self.size / 2
            {
                return false;
            }
            if sorted_o_rows[i as usize].data == sorted_o_rows[(i + 1) as usize].data
                && sorted_o_rows[i as usize].count == self.size / 2
            {
                return false;
            }
            if sorted_x_cols[i as usize].data == sorted_x_cols[(i + 1) as usize].data
                && sorted_x_cols[i as usize].count == self.size / 2
            {
                return false;
            }
            if sorted_o_cols[i as usize].data == sorted_o_cols[(i + 1) as usize].data
                && sorted_o_cols[i as usize].count == self.size / 2
            {
                return false;
            }
        }
        true
    }

    pub fn is_full(&self) -> bool {
        (0..self.size)
            .all(|i| self.x_rows[i as usize].count + self.o_rows[i as usize].count == self.size)
    }

    pub fn is_solved(&self) -> bool {
        self.is_full() && self.is_valid()
    }

    pub fn as_string(&self) -> String {
        let mut result = String::new();
        for row in 0..self.size {
            for col in 0..self.size {
                let mut c = char::from(self.get_cell(row, col).unwrap());
                match (c, self.is_default(row, col)) {
                    ('X', Ok(false)) => c = 'x',
                    ('O', Ok(false)) => c = 'o',
                    (' ', _) => c = '.',
                    _ => (),
                };
                result.push(c);
            }
        }
        result
    }

    pub fn reset(&mut self) {
        for row in 0..self.size {
            for col in 0..self.size {
                if !self.is_default(row, col).unwrap() {
                    self.set_empty(row, col).unwrap();
                }
            }
        }
    }

    pub fn presolve(&mut self) -> PresolveResult {
        for row in 0..self.size {
            for col in 0..self.size {
                if self.get_cell(row, col).unwrap() == BinoxCell::EMPTY {
                    self.set_x(row, col).unwrap();
                    let x_valid = self.is_valid();
                    self.set_o(row, col).unwrap();
                    let o_valid = self.is_valid();
                    match (x_valid, o_valid) {
                        (true, false) => self.set_x(row, col).unwrap(),
                        (false, true) => self.set_o(row, col).unwrap(),
                        (false, false) => {
                            self.set_empty(row, col).unwrap();
                            return PresolveResult::Bad;
                        }
                        (true, true) => self.set_empty(row, col).unwrap(),
                    }
                }
            }
        }
        PresolveResult::Good
    }

    fn presolve_simple(&mut self) -> PresolveResult {
        for row in 0..self.size {
            for col in 0..self.size {
                if self.get_cell(row, col).unwrap() == BinoxCell::EMPTY {
                    self.set_x(row, col).unwrap();
                    let x_valid = self.is_valid_simple();
                    self.set_o(row, col).unwrap();
                    let o_valid = self.is_valid_simple();
                    match (x_valid, o_valid) {
                        (true, false) => self.set_x(row, col).unwrap(),
                        (false, true) => self.set_o(row, col).unwrap(),
                        (false, false) => {
                            self.set_empty(row, col).unwrap();
                            return PresolveResult::Bad;
                        }
                        (true, true) => self.set_empty(row, col).unwrap(),
                    }
                }
            }
        }
        PresolveResult::Good
    }

    pub fn solve(&self, multiple: bool) -> BinoxSolution {
        match (self.is_full(), self.is_valid()) {
            (true, true) => return One(self.clone()),
            (false, true) => (),
            (_, false) => return Zero,
        }
        let mut x_clone = self.clone();
        match x_clone.presolve() {
            PresolveResult::Good => (),
            PresolveResult::Bad => return Zero,
        };
        let (mut empty_cell_row, mut empty_cell_column) = (0, 0);
        'a: for row in alternated_range(self.size) {
            for col in alternated_range(self.size) {
                if x_clone.get_cell(row, col).unwrap() == BinoxCell::EMPTY {
                    (empty_cell_row, empty_cell_column) = (row, col);
                    break 'a;
                }
            }
        }
        let mut o_clone = x_clone.clone();
        x_clone.set_x(empty_cell_row, empty_cell_column).unwrap();
        o_clone.set_o(empty_cell_row, empty_cell_column).unwrap();
        let x_solved = x_clone.solve(multiple);
        match (x_solved, multiple) {
            (Zero, true) => o_clone.solve(true),
            (Zero, false) => o_clone.solve(false),
            (One(a), true) => One(a) + o_clone.solve(false),
            (One(a), false) => One(a),
            (Multiple(a, b), true) => Multiple(a, b),
            (Multiple(a, _), false) => One(a),
        }
    }

    pub fn generate(size: u8, perfect: bool, extras: usize) -> Result<Binox, &'static str> {
        //phase 1 - add some symbols randomly to get started
        let mut binox = Binox::new(size)?;
        let mut rows = (0u8..size).collect::<Vec<u8>>();
        let cols = (0u8..size).collect::<Vec<u8>>();
        rows.shuffle(&mut rand::thread_rng());
        for i in 0..size {
            if rand::random() {
                binox.set_x(rows[i as usize], cols[i as usize]).unwrap();
            } else {
                binox.set_o(rows[i as usize], cols[i as usize]).unwrap();
            }
        }

        //phase 2 - continue adding symbols until there is only one solution
        loop {
            match binox.solve(true) {
                Zero => return Err("something went wrong"),
                One(_) => break,
                Multiple(a, b) => {
                    let diff = a.get_differences(b)?;
                    if diff.is_empty() {
                        break;
                    }
                    let pair = diff
                        .get(rand::thread_rng().gen_range(0..diff.len()))
                        .ok_or("something went wrong")?;
                    if rand::random() {
                        binox.set_x(pair.0, pair.1)?;
                    } else {
                        binox.set_o(pair.0, pair.1)?;
                    }
                }
            }
        }

        //phase 3 - remove symbols that are not needed to find the solution
        for row in 0..size {
            for col in 0..size {
                if binox.get_cell(row, col)? != BinoxCell::EMPTY {
                    let current_cell = binox.get_cell(row, col)?;
                    let mut clone = binox.clone();
                    clone.set_empty(row, col)?;
                    clone.presolve();
                    if clone.get_cell(row, col)? == current_cell {
                        binox.set_empty(row, col)?;
                    }
                }
            }
        }

        //phase 3 - if perfect generation is set, remove even more symbols that are not needed to find the solution
        if perfect {
            for row in 0..size {
                for col in 0..size {
                    if binox.get_cell(row, col)? != BinoxCell::EMPTY {
                        let current_cell = binox.get_cell(row, col)?;
                        binox.set_empty(row, col)?;
                        if let Multiple(..) = binox.solve(true) {
                            binox.set_cell(row, col, current_cell)?;
                        }
                    }
                }
            }
        }

        //phase 5 - add more cells if specified
        if extras > 0 {
            let mut clone = binox.clone();
            clone.presolve_simple();
            let mut empties = clone.get_empties();
            let num = if empties.len() > extras {
                extras
            } else {
                empties.len()
            };
            empties.shuffle(&mut rand::thread_rng());
            clone = match clone.solve(true) {
                Zero => return Err("something went wrong"),
                One(a) => a,
                Multiple(a, _) => a,
            };

            for (row, col) in empties.iter().take(num) {
                binox.set_cell(*row, *col, clone.get_cell(*row, *col).unwrap())?;
            }
        }

        binox.make_cells_unmodifiable();
        Ok(binox)
    }

    fn get_differences(&self, other: Binox) -> Result<Vec<(u8, u8)>, &'static str> {
        if self.size != other.size {
            return Err("must be same size");
        }
        let mut result = Vec::new();
        for row in 0..self.size {
            for col in 0..self.size {
                if self.get_cell(row, col) != other.get_cell(row, col) {
                    result.push((row, col));
                }
            }
        }
        Ok(result)
    }

    fn get_empties(&self) -> Vec<(u8, u8)> {
        let mut result = Vec::new();
        for row in 0..self.size {
            for col in 0..self.size {
                if self.get_cell(row, col).unwrap() == BinoxCell::EMPTY {
                    result.push((row, col));
                }
            }
        }
        result
    }

    fn make_cells_unmodifiable(&mut self) {
        for row in 0..self.size {
            for col in 0..self.size {
                if self.get_cell(row, col).unwrap() != BinoxCell::EMPTY {
                    self.set_default(row, col, true).unwrap();
                }
            }
        }
    }
}

fn alternated_range(n: u8) -> std::vec::IntoIter<u8> {
    let mut result = Vec::new();
    for i in 0..n {
        if i % 2 == 0 {
            result.push(i);
        }
    }
    for i in 0..n {
        if i % 2 == 1 {
            result.push(i);
        }
    }
    result.into_iter()
}

impl fmt::Display for Binox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "   |")?;
        for i in 0..self.size {
            write!(f, "{i:>2} |")?;
        }
        writeln!(f)?;
        for _ in 0..self.size + 1 {
            write!(f, "---+")?;
        }

        for i in 0..self.size {
            writeln!(f)?;
            write!(f, "{i:>2} |")?;
            for j in 0..self.size {
                let mut c: ColoredString = self.get_cell(i, j).unwrap().into();
                if self.is_default(i, j).unwrap() {
                    c = c.bold();
                }
                write!(f, " {} |", c)?;
            }
            writeln!(f)?;
            for _ in 0..self.size + 1 {
                write!(f, "---+")?;
            }
        }

        write!(f, "")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn full_valid_solved() {
        let b = Binox::new_from_string("xx x            ".into());
        assert!(!b.is_full());
        assert!(!b.is_valid());
        assert!(!b.is_solved());
        let b = Binox::new_from_string("xxooooxxxxooooxx".into());
        assert!(b.is_full());
        assert!(!b.is_valid());
        assert!(!b.is_solved());
        let b = Binox::new_from_string("xx  oo          ".into());
        assert!(!b.is_full());
        assert!(b.is_valid());
        assert!(!b.is_solved());
        let b = Binox::new_from_string("xxooxoxooxoxooxx".into());
        assert!(b.is_full());
        assert!(b.is_valid());
        assert!(b.is_solved());
    }
}
