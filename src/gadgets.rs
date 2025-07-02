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
pub use crate::gates::Split4PartsGate;
use crate::gates::SplitNibbleGate;

pub trait U32SplitOps<F: RichField + Extendable<D>, const D: usize> {
    /// Add a 32-bit split into 4 parts
    fn add_u32_split<const K1: usize, const K2: usize, const K3: usize>(
        &mut self,
        input: Target,
    ) -> (Target, Target, Target, Target);
    fn add_u32_split_nibble(&mut self, input: Target) -> Vec<Target>;
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

    fn add_u32_split_nibble(&mut self, input: Target) -> Vec<Target> {
        // Create and add the gate
        let gate = SplitNibbleGate::<F, D>::new_from_config(&self.config);
        let row = self.add_gate(gate, vec![]);

        // Connect input
        self.connect(input, Target::wire(row, 0));

        // Get outputs
        let x0 = Target::wire(row, 1);
        let x1 = Target::wire(row, 2);
        let x2 = Target::wire(row, 3);
        let x3 = Target::wire(row, 4);
        let x4 = Target::wire(row, 5);
        let x5 = Target::wire(row, 6);
        let x6 = Target::wire(row, 7);
        let x7 = Target::wire(row, 8);

        // Add range checks using built-in method
        self.range_check(x0, 4); // Ensures x0 < 2^4
        self.range_check(x1, 4); // Ensures x1 < 2^4
        self.range_check(x2, 4); // Ensures x2 < 2^4
        self.range_check(x3, 4); // Ensures x3 < 2^4
        self.range_check(x4, 4); // Ensures x4 < 2^4
        self.range_check(x5, 4); // Ensures x5 < 2^4
        self.range_check(x6, 4); // Ensures x6 < 2^4
        self.range_check(x7, 4); // Ensures x7 < 2^4

        vec![x0, x1, x2, x3, x4, x5, x6, x7] // return the outputs (little endian)
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
