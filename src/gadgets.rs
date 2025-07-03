use plonky2::{
    field::{extension::Extendable, types::Field},
    hash::hash_types::RichField,
    iop::target::Target,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::GenericConfig,
    },
};
use plonky2_u32::gadgets::arithmetic_u32::U32Target;
use std::marker::PhantomData;

// Assuming U32Target is defined somewhere like this:
// pub struct U32Target(pub Target);

// Re-export the gate for convenience
pub use crate::gates::{Split4PartsGate, SplitU16Gate, SplitU8SpreadGate};

pub trait U32SplitOps<F: RichField + Extendable<D>, const D: usize> {
    /// Add a 32-bit split into 4 parts
    fn add_u32_split<const K1: usize, const K2: usize, const K3: usize>(
        &mut self,
        input: Target,
    ) -> (Target, Target, Target, Target);
    fn add_u32_split_u16(&mut self, input: Target) -> (Target, Target);
    fn add_u32_split_u8_spread(&mut self, input: Target, table_idx: usize) -> (Target, Target);


}

impl<F: RichField + Extendable<D>, const D: usize> U32SplitOps<F, D> for CircuitBuilder<F, D> {
    fn add_u32_split<const K1: usize, const K2: usize, const K3: usize>(
        &mut self,
        input: Target,
    ) -> (Target, Target, Target, Target) {
        // Create and add the gate
        let gate = Split4PartsGate::<F, D, K1, K2, K3>::new_from_config(&self.config);
        let row = self.add_gate(gate, vec![]);

        // Connect input
        self.connect(input, Target::wire(row, 0));

        // Get outputs
        let x0 = Target::wire(row, 1);
        let x1 = Target::wire(row, 2);
        let x2 = Target::wire(row, 3);
        let x3 = Target::wire(row, 4);

        // Add range checks using built-in method
        self.range_check(x0, K1); // Ensures x0 < 2^K1
        self.range_check(x1, K2 - K1); // Ensures x1 < 2^(K2-K1)
        self.range_check(x2, K3 - K2); // Ensures x2 < 2^(K3-K2)
        self.range_check(x3, 32 - K3); // Ensures x3 < 2^(32-K3)

        (x0, x1, x2, x3) // return the outputs (little endian)
    }

    fn add_u32_split_u16(&mut self, input: Target) -> (Target, Target) {
        // Create and add the gate
        let gate = SplitU16Gate::<F, D>::new_from_config(&self.config);
        let row = self.add_gate(gate, vec![]);

        // Connect input
        self.connect(input, Target::wire(row, 0));

        // Get outputs
        let lo = Target::wire(row, 1);
        let hi = Target::wire(row, 2);

        // Add range checks using built-in method
        //self.range_check(lo, 16); // Ensures lo < 2^16
        //self.range_check(hi, 16); // Ensures hi < 2^16

        (lo, hi)
    }

    fn add_u32_split_u8_spread(&mut self, input: Target, table_idx: usize) -> (Target, Target) {
        // Create and add the gate
        let gate = SplitU8SpreadGate::<F, D>::new_from_config(table_idx, &self.config);
        let row = self.add_gate(gate, vec![]);

        // Connect input
        self.connect(input, Target::wire(row, 0));


        // Get outputs
        let even = Target::wire(row, 1);
        let odd = Target::wire(row, 2);
        let even_u8 = Target::wire(row, 3);
        let odd_u8 = Target::wire(row, 4);

        let even_lookup = self.add_lookup_from_index(even_u8, table_idx);
        let odd_lookup = self.add_lookup_from_index(odd_u8, table_idx);

        self.connect(even_lookup, even);
        self.connect(odd_lookup, odd);

        (even, odd)
    }
}

/// Helper struct for building rotation circuits with U32Target
pub struct U32SplitCircuitBuilder<F: RichField + Extendable<D>, const D: usize> {
    builder: CircuitBuilder<F, D>,
}

impl<F: RichField + Extendable<D>, const D: usize> U32SplitCircuitBuilder<F, D> {
    /// Create a new rotation circuit builder with standard config
    pub fn new() -> Self {
        let config = CircuitConfig::standard_recursion_config();
        Self {
            builder: CircuitBuilder::new(config),
        }
    }

    /// Create a new rotation circuit builder with custom config
    pub fn new_with_config(config: CircuitConfig) -> Self {
        Self {
            builder: CircuitBuilder::new(config),
        }
    }

    /// Get a mutable reference to the inner CircuitBuilder
    pub fn builder(&mut self) -> &mut CircuitBuilder<F, D> {
        &mut self.builder
    }

    /// Add a virtual U32Target (input)
    pub fn add_u32_input(&mut self) -> U32Target {
        U32Target(self.builder.add_virtual_target())
    }

    /// Add multiple virtual U32Targets (inputs)
    pub fn add_u32_inputs(&mut self, count: usize) -> Vec<U32Target> {
        (0..count).map(|_| self.add_u32_input()).collect()
    }

    /// Perform a single split on U32Target
    pub fn split<const K1: usize, const K2: usize, const K3: usize>(
        &mut self,
        input: &U32Target,
    ) -> (Target, Target, Target, Target) {
        self.builder.add_u32_split::<K1, K2, K3>(input.0)
    }

    /// Build the circuit
    pub fn build<C: GenericConfig<D, F = F>>(self) -> CircuitData<F, C, D> {
        self.builder.build::<C>()
    }
}



fn xor3_u16_by_spread<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    a: &Target,
    b: &Target,
    c: &Target,
    table_idx: usize,
) -> Target {
    let (a_even, a_odd) = builder.add_u32_split_u8_spread(*a, table_idx);
    let (b_even, b_odd) = builder.add_u32_split_u8_spread(*b, table_idx);
    let (c_even, c_odd) = builder.add_u32_split_u8_spread(*c, table_idx);
    let even = builder.add_many(vec![a_even, b_even, c_even]);
    let odd = builder.add_many(vec![a_odd, b_odd, c_odd]);
    let (even_even, _even_odd) = builder.add_u32_split_u8_spread(even, table_idx);
    let (odd_even, _odd_odd) = builder.add_u32_split_u8_spread(odd, table_idx);
    let res = builder.add_many(vec![odd_even, odd_even, even_even]);
    res
}

pub fn xor3_u32_by_spread<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    a: &U32Target,
    b: &U32Target,
    c: &U32Target,
    table_idx: usize,
) -> U32Target {
    let (a_lo, a_hi) = builder.add_u32_split_u16(a.0);
    let (b_lo, b_hi) = builder.add_u32_split_u16(b.0);
    let (c_lo, c_hi) = builder.add_u32_split_u16(c.0);

    let res_lo = xor3_u16_by_spread(builder, &a_lo, &b_lo, &c_lo, table_idx);
    let res_hi = xor3_u16_by_spread(builder, &a_hi, &b_hi, &c_hi, table_idx);
    let po16 = builder.constant(F::from_canonical_u64(1u64 << 16));
    let res = builder.mul_add(res_hi, po16, res_lo);
    U32Target(res)
}



fn maj_u16_by_spread<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    a: &Target,
    b: &Target,
    c: &Target,
    table_idx: usize,
) -> Target {
    let (a_even, a_odd) = builder.add_u32_split_u8_spread(*a, table_idx);
    let (b_even, b_odd) = builder.add_u32_split_u8_spread(*b, table_idx);
    let (c_even, c_odd) = builder.add_u32_split_u8_spread(*c, table_idx);
    let even = builder.add_many(vec![a_even, b_even, c_even]);
    let odd = builder.add_many(vec![a_odd, b_odd, c_odd]);
    let (_even_even, even_odd) = builder.add_u32_split_u8_spread(even, table_idx);
    let (_odd_even, odd_odd) = builder.add_u32_split_u8_spread(odd, table_idx);
    let res = builder.add_many(vec![odd_odd, odd_odd, even_odd]);
    res
}

pub fn maj_u32_by_spread<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    a: &U32Target,
    b: &U32Target,
    c: &U32Target,
    table_idx: usize,
) -> U32Target {
    let (a_lo, a_hi) = builder.add_u32_split_u16(a.0);
    let (b_lo, b_hi) = builder.add_u32_split_u16(b.0);
    let (c_lo, c_hi) = builder.add_u32_split_u16(c.0);

    let res_lo = maj_u16_by_spread(builder, &a_lo, &b_lo, &c_lo, table_idx);
    let res_hi = maj_u16_by_spread(builder, &a_hi, &b_hi, &c_hi, table_idx);
    let po16 = builder.constant(F::from_canonical_u64(1u64 << 16));
    let res = builder.mul_add(res_hi, po16, res_lo);
    U32Target(res)
}


fn ch_u8_spread<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    a: &Target,
    b: &Target,
    c: &Target,
    table_idx: usize,
) -> Target {
    let spread_full = builder.constant(F::from_canonical_u64(0x5555u64));
    let not_a = builder.sub(spread_full, *a);
    let a_plus_b = builder.add(*a, *b);
    let not_a_plus_c = builder.add(not_a, *c);
    let (_a_plus_b_even, a_plus_b_odd) = builder.add_u32_split_u8_spread(a_plus_b, table_idx);
    let (_not_a_plus_c_even, not_a_plus_c_odd) = builder.add_u32_split_u8_spread(not_a_plus_c, table_idx);
    let odd_sum = builder.add(a_plus_b_odd, not_a_plus_c_odd);
    let (odd_sum_even, _odd_sum_odd) = builder.add_u32_split_u8_spread(odd_sum, table_idx);
    odd_sum_even
}


fn ch_u16_by_spread<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    a: &Target,
    b: &Target,
    c: &Target,
    table_idx: usize,
) -> Target {
    let (a_even, a_odd) = builder.add_u32_split_u8_spread(*a, table_idx);
    let (b_even, b_odd) = builder.add_u32_split_u8_spread(*b, table_idx);
    let (c_even, c_odd) = builder.add_u32_split_u8_spread(*c, table_idx);

    let res_even = ch_u8_spread(builder, &a_even, &b_even, &c_even, table_idx);
    let res_odd = ch_u8_spread(builder, &a_odd, &b_odd, &c_odd, table_idx);
    let res = builder.add_many(vec![res_odd, res_odd, res_even]);
    res
}


pub fn ch_u32_by_spread<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    a: &U32Target,
    b: &U32Target,
    c: &U32Target,
    table_idx: usize,
) -> U32Target {
    let (a_lo, a_hi) = builder.add_u32_split_u16(a.0);
    let (b_lo, b_hi) = builder.add_u32_split_u16(b.0);
    let (c_lo, c_hi) = builder.add_u32_split_u16(c.0);

    let res_lo = ch_u16_by_spread(builder, &a_lo, &b_lo, &c_lo, table_idx);
    let res_hi = ch_u16_by_spread(builder, &a_hi, &b_hi, &c_hi, table_idx);
    let po16 = builder.constant(F::from_canonical_u64(1u64 << 16));
    let res = builder.mul_add(res_hi, po16, res_lo);
    U32Target(res)
}