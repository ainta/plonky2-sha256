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
pub struct SplitNibbleGate<F: RichField + Extendable<D>, const D: usize> {
    _phantom: PhantomData<F>,
}

impl<F: RichField + Extendable<D>, const D: usize> Default for SplitNibbleGate<F, D> {
    fn default() -> Self {
        Self::new_from_config(&CircuitConfig::standard_recursion_config())
    }
}

impl<F: RichField + Extendable<D>, const D: usize> SplitNibbleGate<F, D> {
    pub fn new_from_config(config: &CircuitConfig) -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<F: RichField + Extendable<D>, const D: usize> Gate<F, D> for SplitNibbleGate<F, D> {
    fn id(&self) -> String {
        format!("SplitNibble()")
    }

    fn num_wires(&self) -> usize {
        9
    } // x, x0, x4, x8, x12, x16, x20, x24, x28
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
        let x0 = vars.local_wires[1];
        let x4 = vars.local_wires[2];
        let x8 = vars.local_wires[3];
        let x12 = vars.local_wires[4];
        let x16 = vars.local_wires[5];
        let x20 = vars.local_wires[6];
        let x24 = vars.local_wires[7];
        let x28 = vars.local_wires[8];

        let two_k4 = F::Extension::from_canonical_u64(1u64 << 4);
        let two_k8 = F::Extension::from_canonical_u64(1u64 << 8);
        let two_k12 = F::Extension::from_canonical_u64(1u64 << 12);
        let two_k16 = F::Extension::from_canonical_u64(1u64 << 16);
        let two_k20 = F::Extension::from_canonical_u64(1u64 << 20);
        let two_k24 = F::Extension::from_canonical_u64(1u64 << 24);
        let two_k28 = F::Extension::from_canonical_u64(1u64 << 28);

        // c0: x - (lo + hi·2^K)
        let c0 = x
            - (x0
                + x4 * two_k4
                + x8 * two_k8
                + x12 * two_k12
                + x16 * two_k16
                + x20 * two_k20
                + x24 * two_k24
                + x28 * two_k28);

        vec![c0]
    }

    fn eval_unfiltered_circuit(
        &self,
        builder: &mut CircuitBuilder<F, D>,
        vars: EvaluationTargets<D>,
    ) -> Vec<ExtensionTarget<D>> {
        // Constants
        let two_k4 = builder.constant_extension(F::Extension::from_canonical_u64(1u64 << 4)); // 2^K1
        let two_k8 = builder.constant_extension(F::Extension::from_canonical_u64(1u64 << 8)); // 2^K2
        let two_k12 = builder.constant_extension(F::Extension::from_canonical_u64(1u64 << 12)); // 2^K3
        let two_k16 = builder.constant_extension(F::Extension::from_canonical_u64(1u64 << 16)); // 2^K4
        let two_k20 = builder.constant_extension(F::Extension::from_canonical_u64(1u64 << 20)); // 2^K5
        let two_k24 = builder.constant_extension(F::Extension::from_canonical_u64(1u64 << 24)); // 2^K6
        let two_k28 = builder.constant_extension(F::Extension::from_canonical_u64(1u64 << 28)); // 2^K7

        let x = vars.local_wires[0];
        let x0 = vars.local_wires[1];
        let x4 = vars.local_wires[2];
        let x8 = vars.local_wires[3];
        let x12 = vars.local_wires[4];
        let x16 = vars.local_wires[5];
        let x20 = vars.local_wires[6];
        let x24 = vars.local_wires[7];
        let x28 = vars.local_wires[8];

        let x4_two_k4 = builder.mul_extension(x4, two_k4);
        let x8_two_k8 = builder.mul_extension(x8, two_k8);
        let x12_two_k12 = builder.mul_extension(x12, two_k12);
        let x16_two_k16 = builder.mul_extension(x16, two_k16);
        let x20_two_k20 = builder.mul_extension(x20, two_k20);
        let x24_two_k24 = builder.mul_extension(x24, two_k24);
        let x28_two_k28 = builder.mul_extension(x28, two_k28);
        let x0_plus_x4_two_k4 = builder.add_extension(x0, x4_two_k4);
        let x0_plus_x4_two_k4_plus_x8_two_k8 = builder.add_extension(x0_plus_x4_two_k4, x8_two_k8);
        let x0_plus_x4_two_k4_plus_x8_two_k8_plus_x12_two_k12 =
            builder.add_extension(x0_plus_x4_two_k4_plus_x8_two_k8, x12_two_k12);
        let x0_plus_x4_two_k4_plus_x8_two_k8_plus_x12_two_k12_plus_x16_two_k16 = builder
            .add_extension(
                x0_plus_x4_two_k4_plus_x8_two_k8_plus_x12_two_k12,
                x16_two_k16,
            );
        let x0_plus_x4_two_k4_plus_x8_two_k8_plus_x12_two_k12_plus_x16_two_k16_plus_x20_two_k20 =
            builder.add_extension(
                x0_plus_x4_two_k4_plus_x8_two_k8_plus_x12_two_k12_plus_x16_two_k16,
                x20_two_k20,
            );
        let x0_plus_x4_two_k4_plus_x8_two_k8_plus_x12_two_k12_plus_x16_two_k16_plus_x20_two_k20_plus_x24_two_k24 =
            builder.add_extension(
                x0_plus_x4_two_k4_plus_x8_two_k8_plus_x12_two_k12_plus_x16_two_k16_plus_x20_two_k20,
                x24_two_k24,
            );
        let x0_plus_x4_two_k4_plus_x8_two_k8_plus_x12_two_k12_plus_x16_two_k16_plus_x20_two_k20_plus_x24_two_k24_plus_x28_two_k28 = builder.add_extension(x0_plus_x4_two_k4_plus_x8_two_k8_plus_x12_two_k12_plus_x16_two_k16_plus_x20_two_k20_plus_x24_two_k24, x28_two_k28);
        let c0 = builder.sub_extension(x, x0_plus_x4_two_k4_plus_x8_two_k8_plus_x12_two_k12_plus_x16_two_k16_plus_x20_two_k20_plus_x24_two_k24_plus_x28_two_k28);

        vec![c0]
    }
    fn generators(&self, row: usize, _local_constants: &[F]) -> Vec<WitnessGeneratorRef<F, D>> {
        vec![WitnessGeneratorRef::new(
            SplitNibbleGenerator::<F, D> {
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
struct SplitNibbleGenerator<F: RichField + Extendable<D>, const D: usize> {
    row: usize,
    _phantom: PhantomData<F>,
}

impl<F: RichField + Extendable<D>, const D: usize> SimpleGenerator<F, D>
    for SplitNibbleGenerator<F, D>
{
    fn id(&self) -> String {
        format!("SplitNibbleGenerator(row={})", self.row)
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
        let x0 = x_u64 & ((1u64 << 4) - 1); // Lower 4 bits
        let x4 = (x_u64 >> 4) & ((1u64 << 4) - 1); // Upper 4 bits
        let x8 = (x_u64 >> 8) & ((1u64 << 4) - 1); // Upper 4 bits
        let x12 = (x_u64 >> 12) & ((1u64 << 4) - 1); // Upper 4 bits
        let x16 = (x_u64 >> 16) & ((1u64 << 4) - 1); // Upper 4 bits
        let x20 = (x_u64 >> 20) & ((1u64 << 4) - 1); // Upper 4 bits
        let x24 = (x_u64 >> 24) & ((1u64 << 4) - 1); // Upper 4 bits
        let x28 = (x_u64 >> 28) & ((1u64 << 4) - 1); // Upper 4 bits

        // Set the witness values
        out_buffer.set_target(Target::wire(self.row, 1), F::from_canonical_u64(x0))?;
        out_buffer.set_target(Target::wire(self.row, 2), F::from_canonical_u64(x4))?;
        out_buffer.set_target(Target::wire(self.row, 3), F::from_canonical_u64(x8))?;
        out_buffer.set_target(Target::wire(self.row, 4), F::from_canonical_u64(x12))?;
        out_buffer.set_target(Target::wire(self.row, 5), F::from_canonical_u64(x16))?;
        out_buffer.set_target(Target::wire(self.row, 6), F::from_canonical_u64(x20))?;
        out_buffer.set_target(Target::wire(self.row, 7), F::from_canonical_u64(x24))?;
        out_buffer.set_target(Target::wire(self.row, 8), F::from_canonical_u64(x28))?;

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
