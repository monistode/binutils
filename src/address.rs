use bitvec::vec::BitVec;

#[derive(Debug, Clone, Copy)]
pub struct Address(pub usize); // always in bits for now

impl std::ops::Add<usize> for Address {
    type Output = Address;
    fn add(self, rhs: usize) -> Self::Output {
        Address(self.0 + rhs)
    }
}

impl std::ops::Sub<Address> for Address {
    type Output = i64;

    fn sub(self, rhs: Address) -> Self::Output {
        self.0 as i64 - rhs.0 as i64
    }
}

pub trait AddressIndexable<T> {
    fn index(&self, index: Address) -> T;
    fn write(&mut self, index: Address, value: T);
}

impl AddressIndexable<u16> for BitVec {
    fn index(&self, index: Address) -> u16 {
        let mut result = 0u16;
        for i in 0..16 {
            result <<= 1;
            if index.0 + i < self.len() && self[index.0 + i] {
                result |= 1;
            }
        }
        result
    }

    fn write(&mut self, index: Address, value: u16) {
        let mut value = value;
        for i in 0..16 {
            if index.0 + i < self.len() {
                self.set(index.0 + 15 - i, value & 1 == 1);
                value >>= 1;
            }
        }
    }
}
