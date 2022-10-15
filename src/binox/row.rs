#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BinRow {
    pub data: u16,
    size: u8,
    pub count: u8,
}

impl BinRow {
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
        Ok(BinRow {
            size,
            data: 0,
            count: 0,
        })
    }

    pub fn set_one(&mut self, position: u8) -> Result<(), &str> {
        if position >= self.size {
            return Err("attempted to set one out of range");
        }
        if self.data & (1 << position) == 0 {
            self.count += 1
        }
        self.data |= 1 << position;
        Ok(())
    }

    pub fn set_zero(&mut self, position: u8) -> Result<(), &str> {
        if position >= self.size {
            return Err("attempted to set zero out of range");
        }
        if self.data & (1 << position) != 0 {
            self.count -= 1
        }
        self.data &= !(1 << position);
        Ok(())
    }

    pub fn set_bool(&mut self, position: u8, value: bool) -> Result<(), &str> {
        if value {
            self.set_one(position)
        } else {
            self.set_zero(position)
        }
    }

    pub fn get(&self, position: u8) -> Result<bool, &str> {
        if position >= self.size {
            return Err("attempted to get out of range");
        }
        Ok((self.data & 1 << position) > 0)
    }

    pub fn is_valid_simple(&self) -> bool {
        self.data & self.data << 1 & self.data >> 1 == 0 && self.count <= self.size / 2
    }

    pub fn is_valid(&self) -> bool {
        self.data & self.data << 1 & self.data >> 1 == 0
            && self.count <= self.size / 2
            && !(self.count == self.size / 2
                && (self.data ^ ((1 << self.size) - 1))
                    & ((self.data ^ ((1 << self.size) - 1)) << 1)
                    & ((self.data ^ ((1 << self.size) - 1)) >> 1)
                    != 0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn basics() {
        assert!(BinRow::new(18).is_err());
        assert!(BinRow::new(2).is_err());
        assert!(BinRow::new(7).is_err());
        assert!(BinRow::new(6).is_ok());
        let mut row = BinRow::new(8).unwrap();
        assert_eq!(row.data, 0);
        assert_eq!(row.count, 0);
        row.set_zero(1).unwrap();
        assert_eq!(row.data, 0);
        assert_eq!(row.count, 0);
        row.set_one(1).unwrap();
        row.set_one(3).unwrap();
        row.set_one(1).unwrap();
        assert_eq!(row.data, 0b00001010);
        assert_eq!(row.count, 2);
        assert!(row.set_one(12).is_err());
        assert!(row.is_valid());
        row.set_one(2).unwrap();
        assert!(!row.is_valid());
        row.set_zero(3).unwrap();
        row.set_one(4).unwrap();
        row.set_one(5).unwrap();
        assert!(row.is_valid());
        row.set_one(7).unwrap();
        assert!(!row.is_valid());
    }
}
