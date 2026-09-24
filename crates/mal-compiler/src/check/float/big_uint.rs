use std::cmp::Ordering;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct BigUint(Vec<u32>);

impl Ord for BigUint {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .len()
            .cmp(&other.0.len())
            .then_with(|| self.0.iter().rev().cmp(other.0.iter().rev()))
    }
}

impl PartialOrd for BigUint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl BigUint {
    pub(super) fn from_u64(value: u64) -> Self {
        let mut limbs = vec![value as u32, (value >> 32) as u32];
        normalize(&mut limbs);
        Self(limbs)
    }

    pub(super) fn from_decimal(digits: &str) -> Self {
        let mut value = Self::from_u64(0);
        for digit in digits.bytes() {
            value.multiply_small(10);
            value.add_small(u32::from(digit - b'0'));
        }
        value
    }

    pub(super) fn power_of_ten(exponent: u32) -> Self {
        let mut value = Self::from_u64(1);
        for _ in 0..exponent {
            value.multiply_small(10);
        }
        value
    }

    pub(super) fn bit_length(&self) -> u32 {
        self.0.last().map_or(0, |limb| {
            (self.0.len() as u32 - 1) * 32 + (32 - limb.leading_zeros())
        })
    }

    fn add_small(&mut self, value: u32) {
        let mut carry = u64::from(value);
        for limb in &mut self.0 {
            let sum = u64::from(*limb) + carry;
            *limb = sum as u32;
            carry = sum >> 32;
            if carry == 0 {
                return;
            }
        }
        if carry != 0 {
            self.0.push(carry as u32);
        }
    }

    fn multiply_small(&mut self, value: u32) {
        let mut carry = 0_u64;
        for limb in &mut self.0 {
            let product = u64::from(*limb) * u64::from(value) + carry;
            *limb = product as u32;
            carry = product >> 32;
        }
        if carry != 0 {
            self.0.push(carry as u32);
        }
    }

    pub(super) fn multiply_u64(&self, value: u64) -> Self {
        let mut result = Vec::with_capacity(self.0.len() + 2);
        let mut carry = 0_u128;
        for limb in &self.0 {
            let product = u128::from(*limb) * u128::from(value) + carry;
            result.push(product as u32);
            carry = product >> 32;
        }
        while carry != 0 {
            result.push(carry as u32);
            carry >>= 32;
        }
        normalize(&mut result);
        Self(result)
    }

    pub(super) fn multiply(&self, other: &Self) -> Self {
        let mut result = vec![0_u32; self.0.len() + other.0.len() + 1];
        for (left_index, left) in self.0.iter().enumerate() {
            let mut carry = 0_u64;
            for (right_index, right) in other.0.iter().enumerate() {
                let index = left_index + right_index;
                let product =
                    u64::from(*left) * u64::from(*right) + u64::from(result[index]) + carry;
                result[index] = product as u32;
                carry = product >> 32;
            }
            let mut index = left_index + other.0.len();
            while carry != 0 {
                let sum = u64::from(result[index]) + carry;
                result[index] = sum as u32;
                carry = sum >> 32;
                index += 1;
            }
        }
        normalize(&mut result);
        Self(result)
    }

    pub(super) fn shift_left(&self, bits: u32) -> Self {
        if self.0.is_empty() {
            return self.clone();
        }
        let word_shift = (bits / 32) as usize;
        let bit_shift = bits % 32;
        let mut result = vec![0; word_shift];
        let mut carry = 0_u64;
        for limb in &self.0 {
            let shifted = (u64::from(*limb) << bit_shift) | carry;
            result.push(shifted as u32);
            carry = shifted >> 32;
        }
        if carry != 0 {
            result.push(carry as u32);
        }
        Self(result)
    }

    pub(super) fn subtract(&mut self, other: &Self) {
        debug_assert!(*self >= *other);
        let mut borrow = 0_i64;
        for index in 0..self.0.len() {
            let right = i64::from(other.0.get(index).copied().unwrap_or(0));
            let difference = i64::from(self.0[index]) - right - borrow;
            if difference < 0 {
                self.0[index] = (difference + (1_i64 << 32)) as u32;
                borrow = 1;
            } else {
                self.0[index] = difference as u32;
                borrow = 0;
            }
        }
        debug_assert_eq!(borrow, 0);
        normalize(&mut self.0);
    }
}

fn normalize(limbs: &mut Vec<u32>) {
    while limbs.last() == Some(&0) {
        limbs.pop();
    }
}
