use crate::provenance::Provenance;
use std::ops::{Add, Sub, Mul, Div, Rem};

/// Ourochronos operates on 64-bit unsigned integers modulo 2^64.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Value {
    pub val: u64,
    pub prov: Provenance,
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.val.partial_cmp(&other.val)
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.val.cmp(&other.val)
    }
}

impl Value {
    pub const ZERO: Value = Value { val: 0, prov: Provenance { deps: None } };
    
    pub fn new(v: u64) -> Self {
        Value { val: v, prov: Provenance::none() }
    }

    pub fn with_provenance(v: u64, prov: Provenance) -> Self {
        Value { val: v, prov }
    }
}

// Implement arithmetic that merges provenance.
impl Add for Value {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Value {
            val: self.val.wrapping_add(rhs.val),
            prov: self.prov.merge(&rhs.prov),
        }
    }
}

impl Sub for Value {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Value {
            val: self.val.wrapping_sub(rhs.val),
            prov: self.prov.merge(&rhs.prov),
        }
    }
}

impl Mul for Value {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Value {
            val: self.val.wrapping_mul(rhs.val),
            prov: self.prov.merge(&rhs.prov),
        }
    }
}

impl Div for Value {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        if rhs.val == 0 {
             panic!("Division by zero");
        }
        Value {
            val: self.val.wrapping_div(rhs.val),
            prov: self.prov.merge(&rhs.prov),
        }
    }
}

impl Rem for Value {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self::Output {
         if rhs.val == 0 {
             panic!("Division by zero");
        }
        Value {
            val: self.val.wrapping_rem(rhs.val),
            prov: self.prov.merge(&rhs.prov),
        }
    }
}

/// Addresses are 16-bit indices into memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Address(pub u16);

impl From<u16> for Address {
    fn from(v: u16) -> Self {
        Address(v)
    }
}

impl From<Address> for usize {
    fn from(a: Address) -> Self {
        a.0 as usize
    }
}

/// The size of the memory space (2^16 cells).
pub const MEMORY_SIZE: usize = 65536;

/// A snapshot of the memory state.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Memory {
    cells: Vec<Value>,
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            cells: vec![Value::ZERO; MEMORY_SIZE],
        }
    }
}

impl Memory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read(&self, addr: Address) -> Value {
        self.cells[addr.0 as usize].clone()
    }

    pub fn write(&mut self, addr: Address, val: Value) {
        self.cells[addr.0 as usize] = val;
    }
    
    // For fixed point check: P_final == A_initial
    // Note: Equality checks VALUES only, not provenance!
    // The universe cares about the timeline content, not the causal history (usually).
    // Spec §6: "The consistency function... P_final = A_initial".
    // "Value" domain is integers.
    pub fn is_equal_to(&self, other: &Memory) -> bool {
        // We only compare the .val, ignoring provenance for consistency check?
        // Or does provenance also match?
        // Spec implies values. Let's assume values.
        for (v1, v2) in self.cells.iter().zip(other.cells.iter()) {
            if v1.val != v2.val {
                return false;
            }
        }
        true
    }
}
