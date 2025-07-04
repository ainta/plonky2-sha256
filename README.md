# plonky2-sha256

This repository contains [SNARK](https://en.wikipedia.org/wiki/Non-interactive_zero-knowledge_proof) circuits of a
cryptographic hash function [SHA-256](https://en.wikipedia.org/wiki/SHA-2) implemented
with [Plonky2](https://github.com/mir-protocol/plonky2).

Run benchmarks

```console
RUSTFLAGS=-Ctarget-cpu=native cargo run --release --package plonky2_sha256 --bin plonky2_sha256
```

Benchmark on a Macbook Pro (M1), preimage message size = 2828 (block count = 45)

```console
block count: 45
Constructing inner proof with 261980 gates
[INFO  plonky2::plonk::circuit_builder] Degree before blinding & padding: 262019
[INFO  plonky2::plonk::circuit_builder] Degree after blinding & padding: 262144
[DEBUG plonky2::plonk::circuit_builder] Building circuit took 14.396462s
[DEBUG plonky2::util::timing] 16.7942s to prove
[DEBUG plonky2::util::timing] 0.0064s to verify
```

# SHA-256 Circuit Optimization for Plonky2

## Overview of SHA-256

SHA-256 is a cryptographic hash function widely used for data integrity and security. The hash algorithm processes data blocks of 512 bits, outputting a 256-bit hash value. The compression function, at the core of SHA-256, operates through 64 rounds on input values (A, B, C, D, E, F, G, H), involving several bitwise operations including `Ch`, `Maj`, `Σ0`, and `Σ1`.


want to prove: 2895 bytes = 23160 bits = 46 blocks
problem: time(~50s), memory (~10GB)

## Compression Round Operations

Each compression round performs these on 32-bit unsigned integers:

- **Choice (`Ch`) function:**
```

Ch(E,F,G) = (E ∧ F) ⊕ (¬E ∧ G)

```

- **Majority (`Maj`) function:**
```

Maj(A,B,C) = (A ∧ B) ⊕ (A ∧ C) ⊕ (B ∧ C)

```

- **Σ Functions:**
```

Σ₀(A) = (A ⋙ 2) ⊕ (A ⋙ 13) ⊕ (A ⋙ 22)
Σ₁(E) = (E ⋙ 6) ⊕ (E ⋙ 11) ⊕ (E ⋙ 25)

```

### Modular Addition

Not too expensive with current U32Target.

## SHA-256 Optimizations for Plonky2

### Decomposition of `u32` Values

Operations (`xor3` for Σ functions, Ch, Maj) require frequent decompositions into bits. Three strategies for optimization:

1. **Bit-by-bit XOR** 
2. **4-bit Limbs with Lookup Table** 
3. **Spread Representation** (`b₂0b₁0b₀` format, initially promising but expensive due to LUT constraints)

### Spread Representation

Spread representation maps 8-bit values to 16-bit with interleaved ßzeros, facilitating bitwise operations via lookup tables. However, due to Plonky2 constraints (LUT keys/values must be `u16`), using spread representation for 32-bit XOR3 operations significantly increases circuit complexity.

calculate 
101 ^ 110 ^ 010

Add(010001, 010100, 000100)
(101001) -> even digits are 001


### Lookup Table (4-bit Limbs)

A lookup table approach decomposing values into 4-bit limbs efficiently computes XOR3 operations. Lookup table size:

- `2¹² = 4096` entries mapping `(a, b, c) → a⊕b⊕c`

This method provides similar performance to original implementations with manageable complexity.

### Optimization for Ch and Maj Functions

Both Ch and Maj functions involve bitwise operations suited for spread representation or lookup table optimizations:

- **Ch:** `(a & b) | (~a & c)`
- **Maj:** `(a & b) ^ (b & c) ^ (c & a)`

Lookup table implementations using smaller limbs (4 bits or 8 bits) generally provide optimal performance.

## Final Recommendations

- **4-bit limb decomposition with lookup tables** is optimal for bitwise XOR3 operations in Σ, Ch, and Maj functions.
- **Spread representations**, while initially appealing, substantially increase circuit complexity due to Plonky2 limitations.
- **Custom gates** for decomposition (linear operations plus minimal range checks) significantly improve overall circuit efficiency.

Employing these optimized methods achieves performance similar or superior to traditional bitwise methods with significantly reduced overhead.
