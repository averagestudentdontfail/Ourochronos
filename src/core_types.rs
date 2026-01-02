//! Core types for the OUROCHRONOS virtual machine.
//!
//! This module defines the fundamental data types:
//! - Value: 64-bit unsigned integers with provenance tracking
//! - Address: 16-bit memory indices
//! - Memory: The memory state (65536 cells)

use crate::provenance::Provenance;
use std::ops::{Add, Sub, Mul, Div, Rem, Not, BitAnd, BitOr, BitXor};
use std::fmt;

/// Memory address (16-bit index).
pub type Address = u16;

/// The size of the memory space (2^16 = 65536 cells).
pub const MEMORY_SIZE: usize = 65536;

/// A value in OUROCHRONOS: 64-bit unsigned integer with provenance tracking.
/// 
/// All arithmetic is performed modulo 2^64 (wrapping semantics).
/// Provenance tracks which anamnesis cells influenced this value.
#[derive(Clone, PartialEq, Eq, Hash, Default)]
pub struct Value {
    /// The numeric value (64-bit unsigned).
    pub val: u64,
    /// Causal provenance (which oracle cells influenced this value).
    pub prov: Provenance,
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{:?}]", self.val, self.prov)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.val)
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.val.cmp(&other.val)
    }
}

impl Value {
    /// The zero value with no provenance.
    pub const ZERO: Value = Value { val: 0, prov: Provenance::none() };
    
    /// The one value with no provenance.
    pub const ONE: Value = Value { val: 1, prov: Provenance::none() };
    
    /// Create a new value with no provenance.
    pub fn new(v: u64) -> Self {
        Value { val: v, prov: Provenance::none() }
    }
    
    /// Create a value with explicit provenance.
    pub fn with_provenance(v: u64, prov: Provenance) -> Self {
        Value { val: v, prov }
    }
    
    /// Check if this value is temporally pure.
    pub fn is_pure(&self) -> bool {
        self.prov.is_pure()
    }
    
    /// Convert to boolean (0 = false, nonzero = true).
    pub fn to_bool(&self) -> bool {
        self.val != 0
    }
    
    /// Create boolean value (1 or 0) with merged provenance.
    pub fn from_bool_with_prov(b: bool, prov: Provenance) -> Self {
        Value { val: if b { 1 } else { 0 }, prov }
    }
}

// Arithmetic operations with provenance merging

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
    /// Division with zero-divisor handling per specification §5.3:
    /// v₁ ⟨Div⟩ v₂ = v₁ ÷ v₂ if v₂ ≠ 0, else 0
    fn div(self, rhs: Self) -> Self::Output {
        Value {
            val: if rhs.val == 0 { 0 } else { self.val.wrapping_div(rhs.val) },
            prov: self.prov.merge(&rhs.prov),
        }
    }
}

impl Rem for Value {
    type Output = Self;
    /// Modulo with zero-divisor handling per specification §5.3:
    /// v₁ ⟨Mod⟩ v₂ = v₁ mod v₂ if v₂ ≠ 0, else 0
    fn rem(self, rhs: Self) -> Self::Output {
        Value {
            val: if rhs.val == 0 { 0 } else { self.val.wrapping_rem(rhs.val) },
            prov: self.prov.merge(&rhs.prov),
        }
    }
}

// Bitwise operations

impl Not for Value {
    type Output = Self;
    fn not(self) -> Self::Output {
        Value {
            val: !self.val,
            prov: self.prov,
        }
    }
}

impl BitAnd for Value {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Value {
            val: self.val & rhs.val,
            prov: self.prov.merge(&rhs.prov),
        }
    }
}

impl BitOr for Value {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Value {
            val: self.val | rhs.val,
            prov: self.prov.merge(&rhs.prov),
        }
    }
}

impl BitXor for Value {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Value {
            val: self.val ^ rhs.val,
            prov: self.prov.merge(&rhs.prov),
        }
    }
}

/// A snapshot of the memory state.
/// 
/// Memory consists of MEMORY_SIZE (65536) cells, each holding a Value.
/// Initially all cells are zero with no provenance.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Memory {
    cells: Vec<Value>,
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Only show non-zero cells
        let nonzero: Vec<_> = self.cells.iter()
            .enumerate()
            .filter(|(_, v)| v.val != 0)
            .collect();
        
        if nonzero.is_empty() {
            write!(f, "Memory{{all zero}}")
        } else {
            write!(f, "Memory{{")?;
            for (i, (addr, val)) in nonzero.iter().enumerate() {
                if i > 0 { write!(f, ", ")?; }
                write!(f, "[{}]={}", addr, val.val)?;
            }
            write!(f, "}}")
        }
    }
}

impl Memory {
    /// Create a new memory state with all cells set to zero.
    pub fn new() -> Self {
        Self {
            cells: vec![Value::ZERO; MEMORY_SIZE],
        }
    }
    
    /// Read the value at the given address.
    pub fn read(&self, addr: Address) -> Value {
        self.cells[addr as usize].clone()
    }
    
    /// Write a value to the given address.
    pub fn write(&mut self, addr: Address, val: Value) {
        self.cells[addr as usize] = val;
    }
    
    /// Check if two memory states are equal (by value, ignoring provenance).
    /// This is the fixed-point check: P_final = A_initial.
    pub fn values_equal(&self, other: &Memory) -> bool {
        self.cells.iter()
            .zip(other.cells.iter())
            .all(|(v1, v2)| v1.val == v2.val)
    }
    
    /// Find addresses where values differ between two memory states.
    pub fn diff(&self, other: &Memory) -> Vec<Address> {
        self.cells.iter()
            .zip(other.cells.iter())
            .enumerate()
            .filter(|(_, (v1, v2))| v1.val != v2.val)
            .map(|(i, _)| i as Address)
            .collect()
    }
    
    /// Find all non-zero cells.
    pub fn non_zero_cells(&self) -> Vec<(Address, &Value)> {
        self.cells.iter()
            .enumerate()
            .filter(|(_, v)| v.val != 0)
            .map(|(i, v)| (i as Address, v))
            .collect()
    }
    
    /// Get a hash of the memory state (for cycle detection).
    pub fn state_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        for cell in &self.cells {
            cell.val.hash(&mut hasher);
        }
        hasher.finish()
    }
    
    /// Collect all provenance information from written cells.
    pub fn collect_provenance(&self) -> Provenance {
        let mut result = Provenance::none();
        for cell in &self.cells {
            if cell.prov.is_temporal() {
                result = result.merge(&cell.prov);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_value_arithmetic() {
        let a = Value::new(10);
        let b = Value::new(3);
        
        assert_eq!((a.clone() + b.clone()).val, 13);
        assert_eq!((a.clone() - b.clone()).val, 7);
        assert_eq!((a.clone() * b.clone()).val, 30);
        assert_eq!((a.clone() / b.clone()).val, 3);
        assert_eq!((a.clone() % b.clone()).val, 1);
    }
    
    #[test]
    fn test_division_by_zero_returns_zero() {
        let a = Value::new(42);
        let zero = Value::new(0);
        
        assert_eq!((a.clone() / zero.clone()).val, 0);
        assert_eq!((a.clone() % zero.clone()).val, 0);
    }
    
    #[test]
    fn test_wrapping_arithmetic() {
        let max = Value::new(u64::MAX);
        let one = Value::new(1);
        
        assert_eq!((max + one).val, 0); // Wraps around
    }
    
    #[test]
    fn test_memory_values_equal() {
        let mut m1 = Memory::new();
        let mut m2 = Memory::new();
        
        assert!(m1.values_equal(&m2));
        
        m1.write(100, Value::new(42));
        assert!(!m1.values_equal(&m2));
        
        m2.write(100, Value::new(42));
        assert!(m1.values_equal(&m2));
    }
    
    #[test]
    fn test_memory_diff() {
        let mut m1 = Memory::new();
        let mut m2 = Memory::new();
        
        m1.write(5, Value::new(10));
        m2.write(5, Value::new(20));
        m2.write(10, Value::new(30));
        
        let diff = m1.diff(&m2);
        assert!(diff.contains(&5));
        assert!(diff.contains(&10));
    }
}
