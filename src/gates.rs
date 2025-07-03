use anyhow::Result;
use std::marker::PhantomData;

use plonky2::{
    field::{extension::Extendable, types::Field},
    gates::gate::{Gate, GateRef},
    hash::hash_types::RichField,
    iop::{
        ext_target::ExtensionTarget,
        generator::{GeneratedValues, SimpleGenerator, WitnessGeneratorRef},
        target::Target,
        witness::{PartitionWitness, Witness, WitnessWrite},
    },
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CommonCircuitData},
        vars::{EvaluationTargets, EvaluationVars},
    },
    util::serialization::{Buffer, IoResult, Read, Write},
};

#[derive(Copy, Clone, Debug)]
pub struct Split4PartsGate<
    F: RichField + Extendable<D>,
    const D: usize,
    const K1: usize,
    const K2: usize,
    const K3: usize,
> {
    _phantom: PhantomData<F>,
}

impl<
        F: RichField + Extendable<D>,
        const D: usize,
        const K1: usize,
        const K2: usize,
        const K3: usize,
    > Default for Split4PartsGate<F, D, K1, K2, K3>
{
    fn default() -> Self {
        Self::new_from_config(&CircuitConfig::standard_recursion_config())
    }
}

impl<
        F: RichField + Extendable<D>,
        const D: usize,
        const K1: usize,
        const K2: usize,
        const K3: usize,
    > Split4PartsGate<F, D, K1, K2, K3>
{
    pub fn new_from_config(config: &CircuitConfig) -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<
        F: RichField + Extendable<D>,
        const D: usize,
        const K1: usize,
        const K2: usize,
        const K3: usize,
    > Gate<F, D> for Split4PartsGate<F, D, K1, K2, K3>
{
    fn id(&self) -> String {
        format!("Split4Parts({K1}, {K2}, {K3})")
    }

    fn num_wires(&self) -> usize {
        5
    } // x, x0, x1, x2, x3
    fn num_constants(&self) -> usize {
        0
    }
    fn degree(&self) -> usize {
        1
    } // only linear constraints
    fn num_constraints(&self) -> usize {
        1
    }

    fn eval_unfiltered(&self, vars: EvaluationVars<F, D>) -> Vec<F::Extension> {
        // Constants
        let two_k1 = F::Extension::from_canonical_u64(1u64 << K1); // 2^K1
        let two_k2 = F::Extension::from_canonical_u64(1u64 << K2); // 2^K2
        let two_k3 = F::Extension::from_canonical_u64(1u64 << K3); // 2^K3

        let x = vars.local_wires[0];
        let x0 = vars.local_wires[1];
        let x1 = vars.local_wires[2];
        let x2 = vars.local_wires[3];
        let x3 = vars.local_wires[4];

        // c0: x - (lo + hi·2^K)
        let c0 = x - (x0 + x1 * two_k1 + x2 * two_k2 + x3 * two_k3);

        vec![c0]
    }

    fn eval_unfiltered_circuit(
        &self,
        builder: &mut CircuitBuilder<F, D>,
        vars: EvaluationTargets<D>,
    ) -> Vec<ExtensionTarget<D>> {
        // Constants
        let two_k1 = builder.constant_extension(F::Extension::from_canonical_u64(1u64 << K1)); // 2^K1
        let two_k2 = builder.constant_extension(F::Extension::from_canonical_u64(1u64 << K2)); // 2^K2
        let two_k3 = builder.constant_extension(F::Extension::from_canonical_u64(1u64 << K3)); // 2^K3

        let x = vars.local_wires[0];
        let x0 = vars.local_wires[1];
        let x1 = vars.local_wires[2];
        let x2 = vars.local_wires[3];
        let x3 = vars.local_wires[4];

        let x1_two_k1 = builder.mul_extension(x1, two_k1);
        let x2_two_k2 = builder.mul_extension(x2, two_k2);
        let x3_two_k3 = builder.mul_extension(x3, two_k3);
        let x0_plus_x1_two_k1 = builder.add_extension(x0, x1_two_k1);
        let x0_plus_x1_two_k1_plus_x2_two_k2 = builder.add_extension(x0_plus_x1_two_k1, x2_two_k2);
        let x0_plus_x1_two_k1_plus_x2_two_k2_plus_x3_two_k3 =
            builder.add_extension(x0_plus_x1_two_k1_plus_x2_two_k2, x3_two_k3);
        let c0 = builder.sub_extension(x, x0_plus_x1_two_k1_plus_x2_two_k2_plus_x3_two_k3);

        vec![c0]
    }
    fn generators(&self, row: usize, _local_constants: &[F]) -> Vec<WitnessGeneratorRef<F, D>> {
        vec![WitnessGeneratorRef::new(
            Split4PartsGenerator::<F, D, K1, K2, K3> {
                row,
                _phantom: PhantomData,
            }
            .adapter(),
        )]
    }

    // Nothing special in serialized form
    fn serialize(
        &self,
        _dst: &mut Vec<u8>,
        _common_data: &CommonCircuitData<F, D>,
    ) -> IoResult<()> {
        Ok(())
    }

    fn deserialize(_src: &mut Buffer, _common_data: &CommonCircuitData<F, D>) -> IoResult<Self> {
        Ok(Self {
            _phantom: PhantomData,
        })
    }
}

// Witness generator for the gate
#[derive(Debug, Clone)]
struct Split4PartsGenerator<
    F: RichField + Extendable<D>,
    const D: usize,
    const K1: usize,
    const K2: usize,
    const K3: usize,
> {
    row: usize,
    _phantom: PhantomData<F>,
}

impl<
        F: RichField + Extendable<D>,
        const D: usize,
        const K1: usize,
        const K2: usize,
        const K3: usize,
    > SimpleGenerator<F, D> for Split4PartsGenerator<F, D, K1, K2, K3>
{
    fn id(&self) -> String {
        format!("Split4PartsGenerator<{K1}, {K2}, {K3}>(row={})", self.row)
    }

    fn dependencies(&self) -> Vec<Target> {
        vec![Target::wire(self.row, 0)] // Only depends on x
    }

    fn run_once(
        &self,
        witness: &PartitionWitness<F>,
        out_buffer: &mut GeneratedValues<F>,
    ) -> Result<()> {
        let x_val = witness.get_target(Target::wire(self.row, 0));

        // Perform the rotation
        let x_u64 = x_val.to_canonical_u64();
        let x0 = x_u64 & ((1u64 << K1) - 1); // Lower K1 bits
        let x1 = (x_u64 >> K1) & ((1u64 << (K2 - K1)) - 1); // Upper K2-K1 bits
        let x2 = (x_u64 >> K2) & ((1u64 << (K3 - K2)) - 1); // Upper K3-K2 bits
        let x3 = (x_u64 >> K3) & ((1u64 << (32 - K3)) - 1); // Upper 32-K3 bits

        // Set the witness values
        out_buffer.set_target(Target::wire(self.row, 1), F::from_canonical_u64(x0))?;
        out_buffer.set_target(Target::wire(self.row, 2), F::from_canonical_u64(x1))?;
        out_buffer.set_target(Target::wire(self.row, 3), F::from_canonical_u64(x2))?;
        out_buffer.set_target(Target::wire(self.row, 4), F::from_canonical_u64(x3))?;

        Ok(())
    }

    fn serialize(&self, dst: &mut Vec<u8>, _common_data: &CommonCircuitData<F, D>) -> IoResult<()> {
        dst.write_usize(self.row)
    }

    fn deserialize(src: &mut Buffer, _common_data: &CommonCircuitData<F, D>) -> IoResult<Self> {
        let row = src.read_usize()?;
        Ok(Self {
            row,
            _phantom: PhantomData,
        })
    }
}



#[derive(Copy, Clone, Debug)]
pub struct SplitU8SpreadGate<F: RichField + Extendable<D>, const D: usize> {
    table_idx: usize,
    _phantom: PhantomData<F>,
}


impl<F: RichField + Extendable<D>, const D: usize> SplitU8SpreadGate<F, D> {
    pub fn new_from_config(table_idx: usize, config: &CircuitConfig) -> Self {
        Self {
            table_idx,
            _phantom: PhantomData,
        }
    }
}

impl<F: RichField + Extendable<D>, const D: usize> Gate<F, D> for SplitU8SpreadGate<F, D> {
    fn id(&self) -> String {
        format!("SplitU8Spread({})", self.table_idx)
    }

    fn num_wires(&self) -> usize {
        5
    } // x, even, odd
    fn num_constants(&self) -> usize {
        0
    }
    fn degree(&self) -> usize {
        1
    } // only linear constraints
    fn num_constraints(&self) -> usize {
        1
    }

    fn eval_unfiltered(&self, vars: EvaluationVars<F, D>) -> Vec<F::Extension> {
        // Constants
        let x = vars.local_wires[0];
        let even = vars.local_wires[1];
        let odd = vars.local_wires[2];

        let c0 = x - (even + odd * F::Extension::from_canonical_u64(2u64));

        vec![c0]
    }

    fn eval_unfiltered_circuit(
        &self,
        builder: &mut CircuitBuilder<F, D>,
        vars: EvaluationTargets<D>,
    ) -> Vec<ExtensionTarget<D>> {
        // Constants

        let x = vars.local_wires[0];
        let even = vars.local_wires[1];
        let odd = vars.local_wires[2];

        let two = builder.constant_extension(F::Extension::from_canonical_u64(2u64));
        let double_odd_add_even = builder.mul_add_extension(two, odd, even);
        let c0 = builder.sub_extension(x, double_odd_add_even);



        vec![c0]
    }
    fn generators(&self, row: usize, _local_constants: &[F]) -> Vec<WitnessGeneratorRef<F, D>> {
        vec![WitnessGeneratorRef::new(
            SplitU8SpreadGenerator::<F, D> {
                row,
                _phantom: PhantomData,
            }
            .adapter(),
        )]
    }

    // Nothing special in serialized form
    fn serialize(
        &self,
        dst: &mut Vec<u8>,
        _common_data: &CommonCircuitData<F, D>,
    ) -> IoResult<()> {
        dst.write_usize(self.table_idx)?;
        Ok(())
    }

    fn deserialize(src: &mut Buffer, _common_data: &CommonCircuitData<F, D>) -> IoResult<Self> {
        let table_idx = src.read_usize()?;
        Ok(Self {
            table_idx,
            _phantom: PhantomData,
        })
    }
}

// Witness generator for the gate
#[derive(Debug, Clone)]
struct SplitU8SpreadGenerator<F: RichField + Extendable<D>, const D: usize> {
    row: usize,
    _phantom: PhantomData<F>,
}

impl<F: RichField + Extendable<D>, const D: usize> SimpleGenerator<F, D>
    for SplitU8SpreadGenerator<F, D>
{
    fn id(&self) -> String {
        format!("SplitU8SpreadGenerator(row={})", self.row)
    }

    fn dependencies(&self) -> Vec<Target> {
        vec![Target::wire(self.row, 0)] // Only depends on x
    }

    fn run_once(
        &self,
        witness: &PartitionWitness<F>,
        out_buffer: &mut GeneratedValues<F>,
    ) -> Result<()> {
        let x_val = witness.get_target(Target::wire(self.row, 0));

        // Perform the rotation
        let x_u64 = x_val.to_canonical_u64();

        let mut even = 0;
        let mut odd = 0;
        let mut even_u8 = 0;
        let mut odd_u8 = 0;

        for i in 0..8 {
            let bit_even = (x_u64 >> (2*i)) & 1;
            let bit_odd = (x_u64 >> (2*i + 1)) & 1;
            even |= bit_even << (i*2);
            odd |= bit_odd << (i*2);
            even_u8 |= bit_even << i;
            odd_u8 |= bit_odd << i;
        }

        // Set the witness values
        out_buffer.set_target(Target::wire(self.row, 1), F::from_canonical_u64(even))?;
        out_buffer.set_target(Target::wire(self.row, 2), F::from_canonical_u64(odd))?;
        out_buffer.set_target(Target::wire(self.row, 3), F::from_canonical_u64(even_u8))?;
        out_buffer.set_target(Target::wire(self.row, 4), F::from_canonical_u64(odd_u8))?;

        Ok(())
    }

    fn serialize(&self, dst: &mut Vec<u8>, _common_data: &CommonCircuitData<F, D>) -> IoResult<()> {
        dst.write_usize(self.row)?;
        Ok(())
    }

    fn deserialize(src: &mut Buffer, _common_data: &CommonCircuitData<F, D>) -> IoResult<Self> {
        let row = src.read_usize()?;
        Ok(Self {
            row,
            _phantom: PhantomData,
        })
    }
}


#[derive(Copy, Clone, Debug)]
pub struct SplitU16Gate<F: RichField + Extendable<D>, const D: usize> {
    _phantom: PhantomData<F>,
}

impl<F: RichField + Extendable<D>, const D: usize> Default for SplitU16Gate<F, D> {
    fn default() -> Self {
        Self::new_from_config(&CircuitConfig::standard_recursion_config())
    }
}

impl<F: RichField + Extendable<D>, const D: usize> SplitU16Gate<F, D> {
    pub fn new_from_config(config: &CircuitConfig) -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<F: RichField + Extendable<D>, const D: usize> Gate<F, D> for SplitU16Gate<F, D> {
    fn id(&self) -> String {
        format!("Splitu16()")
    }

    fn num_wires(&self) -> usize {
        3
    } // x, lo, hi
    fn num_constants(&self) -> usize {
        0
    }
    fn degree(&self) -> usize {
        1
    } // only linear constraints
    fn num_constraints(&self) -> usize {
        1
    }

    fn eval_unfiltered(&self, vars: EvaluationVars<F, D>) -> Vec<F::Extension> {
        // Constants
        let x = vars.local_wires[0];
        let lo = vars.local_wires[1];
        let hi = vars.local_wires[2];

        let two_k16 = F::Extension::from_canonical_u64(1u64 << 16);
        // c0: x - (lo + hi·2^16)
        let c0 = x - (lo + hi * two_k16);

        vec![c0]
    }

    fn eval_unfiltered_circuit(
        &self,
        builder: &mut CircuitBuilder<F, D>,
        vars: EvaluationTargets<D>,
    ) -> Vec<ExtensionTarget<D>> {
        // Constants
        let two_k16 = builder.constant_extension(F::Extension::from_canonical_u64(1u64 << 16)); // 2^K1

        let x = vars.local_wires[0];
        let lo = vars.local_wires[1];
        let hi = vars.local_wires[2];

        let hi16lo = builder.mul_add_extension(hi, two_k16, lo);
        let c0 = builder.sub_extension(x, hi16lo);

        vec![c0]
    }
    fn generators(&self, row: usize, _local_constants: &[F]) -> Vec<WitnessGeneratorRef<F, D>> {
        vec![WitnessGeneratorRef::new(
            SplitU16Generator::<F, D> {
                row,
                _phantom: PhantomData,
            }
            .adapter(),
        )]
    }

    // Nothing special in serialized form
    fn serialize(
        &self,
        _dst: &mut Vec<u8>,
        _common_data: &CommonCircuitData<F, D>,
    ) -> IoResult<()> {
        Ok(())
    }

    fn deserialize(_src: &mut Buffer, _common_data: &CommonCircuitData<F, D>) -> IoResult<Self> {
        Ok(Self {
            _phantom: PhantomData,
        })
    }
}

// Witness generator for the gate
#[derive(Debug, Clone)]
struct SplitU16Generator<F: RichField + Extendable<D>, const D: usize> {
    row: usize,
    _phantom: PhantomData<F>,
}

impl<F: RichField + Extendable<D>, const D: usize> SimpleGenerator<F, D>
    for SplitU16Generator<F, D>
{
    fn id(&self) -> String {
        format!("SplitU16Generator(row={})", self.row)
    }

    fn dependencies(&self) -> Vec<Target> {
        vec![Target::wire(self.row, 0)] // Only depends on x
    }

    fn run_once(
        &self,
        witness: &PartitionWitness<F>,
        out_buffer: &mut GeneratedValues<F>,
    ) -> Result<()> {
        let x_val = witness.get_target(Target::wire(self.row, 0));

        // Perform the rotation
        let x_u64 = x_val.to_canonical_u64();
        let lo = x_u64 & ((1u64 << 16) - 1); // Lower 16 bits
        let hi = x_u64 >> 16; // Upper 16 bits

        // Set the witness values
        out_buffer.set_target(Target::wire(self.row, 1), F::from_canonical_u64(lo))?;
        out_buffer.set_target(Target::wire(self.row, 2), F::from_canonical_u64(hi))?;

        Ok(())
    }

    fn serialize(&self, dst: &mut Vec<u8>, _common_data: &CommonCircuitData<F, D>) -> IoResult<()> {
        dst.write_usize(self.row)
    }

    fn deserialize(src: &mut Buffer, _common_data: &CommonCircuitData<F, D>) -> IoResult<Self> {
        let row = src.read_usize()?;
        Ok(Self {
            row,
            _phantom: PhantomData,
        })
    }
}



