# OUROCHRONOS Specification & Addendums

## Core Specification

### Mathematical Foundations (§1–2)
Formal definitions of partial orders, complete partial orders, Scott continuity, and the Kleene fixed-point theorem; the Deutschian CTC model as the theoretical basis; semantic domains for values (64-bit unsigned integers modulo 2⁶⁴), addresses (16-bit), memory states, stacks, and program states.

### Temporal Memory Model (§3)
The dual-memory architecture comprising Present (read-write, constructed during execution) and Anamnesis (read-only, the "message from the future"); the formal consistency condition P_final = A_initial; the prophecy-fulfilment duality as the programming paradigm.

### Syntax (§4)
Complete lexical structure, EBNF grammar for the concrete syntax, and abstract syntax definitions. The instruction set includes temporal operations (ORACLE for reading the future, PROPHECY for fulfilling it), standard stack manipulation, arithmetic and logic, structured control flow, and explicit PARADOX for signalling inconsistency.

### Operational Semantics (§5)
Small-step semantics for every instruction; epoch execution as deterministic state transformation; formal treatment of expression evaluation with modular arithmetic.

### Fixed-Point Computation (§6)
The consistency function F_{Π,I} : Mem → Mem × Output × Status; naive iteration, widening with narrowing for non-contractive programs, and canonical fixed-point selection via lexicographic ordering; convergence analysis and the iteration bound guarantee.

### Determinism and Consistency (§7)
Proofs of epoch determinism and fixed-point determinism; classification of programs into trivially consistent, self-fulfilling, paradoxical, and divergent.

### Turing Completeness (§8)
Proof via TM simulation; proof that consistency existence is undecidable (reduction from halting).

### Compiler Implementation (§9)
Complete bytecode format with opcode table; VM state structure; epoch execution loop and fixed-point driver in pseudocode; optimisations including anamnesis dependency analysis, incremental fixed-point computation, and memoisation.

## Addendum 1: Refinements & Theory

### 1. Formal Temporal Computation Theory (§1)
The addendum establishes that OURO = PSPACE, connecting OUROCHRONOS to the Aaronson-Watrous theorem on closed timelike curves. This is not merely an assertion; the document includes full proofs of both containment directions. The reduction from TQBF (True Quantified Boolean Formula) demonstrates that the fixed-point mechanism allows encoding of alternating quantifiers: existential variables are "guessed" via oracle, universal variables are verified exhaustively.

I have also defined a temporal complexity hierarchy OURO[d] based on causal depth, showing that OURO[0] = P (no temporal features), OURO[1] ⊇ NP ∪ coNP (single-level witness patterns), and the union across all depths yields PSPACE.

The fixed-point multiplicity theory (§1.4–1.5) formalises what it means for programs to be confluent, divergent, or ambiguous, and establishes that programs compute relations rather than functions under nondeterministic semantics.

### 2. Revised Execution Semantics (§2)
Three semantic modes are now defined:

*   **Pure Mode**: Nondeterministic fixed-point selection with unbounded iteration. The interpreter may not terminate, which is philosophically correct: an inconsistent program represents an impossible timeline, and non-termination reflects the universe's failure to find a consistent history.
*   **Bounded Mode**: Nondeterministic selection with configurable limits, cycle detection, and perturbation strategies for escaping local structure.
*   **Diagnostic Mode**: Exhaustive exploration of the fixed-point spectrum, discovering all consistent executions and characterising non-convergent behaviour.

The initial anamnesis selection strategies (Zero, Random, Seeded, Guided, Adversarial) allow control over which fixed points are reachable.

### 3. Type System and Causal Tracking (§3)
The type system introduces provenance as a first-class concept. Each value carries metadata about which anamnesis cells influenced its computation. The provenance lattice allows static analysis of temporal dependencies, enabling:

*   Identification of "temporally pure" regions (no oracle dependency) that can be optimised
*   Type-based verification that certain cells will stabilise
*   Gradual typing for incremental adoption

The causal graph construction algorithm tracks how information flows from oracle reads to present writes, enabling precise diagnosis of consistency failures.

### 4. Diagnostic Infrastructure (§4)
Comprehensive tooling for practical development:

*   Trajectory recording: Complete history of all epochs with instruction-level traces
*   Convergence reports: Detailed analysis of stability, oscillation, and divergence patterns
*   Causal summaries: Feedback loop detection, critical path identification, influence matrices
*   Interactive debugging: Step-through execution, breakpoints, state inspection, perturbation commands
*   Visualisation outputs: DOT format causal graphs, ASCII convergence timelines, stability heatmaps

### 5. Canonical Temporal Patterns (§6)
Five fundamental patterns for temporal programming:

*   Witness Pattern: Receive and verify (NP problems)
*   Bootstrap Pattern: Self-referential computation
*   Oracle Table Pattern: Self-consistent lookup tables (dynamic programming)
*   Constraint Satisfaction Pattern: PSPACE via quantifier encoding
*   Paradox-Avoidance Pattern: Robust programs with multiple valid branches

## Addendum 2: Optimization & SMT

### Performance Optimisation (§1–2)
The addendum develops a multi-layered optimisation framework:
*   **Temporal Core Reduction**: Static analysis identifies the "temporal core"—cells that participate in feedback loops between oracle reads and present writes. Cells outside this core are deterministic functions of core cells and need not participate in fixed-point iteration. For programs where |TC(Π)| << M, this reduces the search space exponentially.
*   **Stratified Evaluation**: When the temporal dependency graph admits a topological ordering (is mostly acyclic), we can evaluate strata sequentially. Lower strata stabilise before affecting higher strata, providing a convergence bound of O(k · W) where k is the number of strata.
*   **Incremental Epoch Evaluation**: Rather than re-executing the entire epoch when anamnesis changes, we track dependency cones and recompute only affected cells. Combined with Merkle tree hashing for efficient state comparison, this reduces per-epoch cost substantially.
*   **Contraction Detection**: Programs where the epoch function is contractive (distance-reducing) converge geometrically. We can estimate the contraction factor empirically and, when detected, provide tight iteration bounds via the Banach fixed-point theorem.
*   **Compilation to SAT/SMT**: The most powerful optimisation: rather than iterating, we compile the fixed-point constraint A = F(A) directly to a satisfiability problem and invoke industrial solvers (Z3, CVC5). For programs with moderate memory footprints, this transforms an exponential search into a single solver invocation. The SMT approach using bit-vector and array theories handles the full semantics elegantly.

The tiered execution architecture combines these techniques: fast iteration with all optimisations first, falling back to SMT for difficult cases, with full diagnosis as the final tier.

### Paradox Diagnosis (§3)
This was the harder problem. The fundamental insight is that paradoxes arise from unsatisfiable constraints, and explaining unsatisfiability is related to proof complexity. The addendum develops paradox witnesses—constructive proofs of inconsistency with explanatory power:
*   **Cycle Witnesses**: When the trajectory reveals a k-cycle (A₀ → A₁ → ... → A₀), we can present the concrete oscillating states. The programmer sees exactly which cells are changing and how they form a closed loop.
*   **Divergence Witnesses**: When a cell grows monotonically without bound, we detect this pattern and report that no fixed point is reachable because the value never stabilises.
*   **Negative Causal Loop Detection**: Static analysis identifies the "grandfather paradox" structure: cycles in the causal graph where an odd number of negations occur. The constraint A = ¬A is manifestly unsatisfiable, and we can point to the exact code path that creates it.
*   **Conflict Core Extraction**: The most powerful technique: when we compile to SMT and obtain UNSAT, we extract the minimal unsatisfiable core—the smallest set of constraints that conflict. This is translated back to source-level cells and presented as: "The fixed-point constraints for cells {a₁, a₂, a₃} are mutually unsatisfiable. Here's why."
*   **Hierarchical Diagnosis**: Different witness types have different computational costs. We apply them in order of expense: runtime cycle detection (cheap), static negative loop analysis (cheap), SMT conflict extraction (expensive). This provides quick answers for common cases whilst retaining full diagnostic power.
*   **Human-Readable Proof Translation**: The proof terms are rendered into natural language explanations with source locations. The output looks like the example in the document: "Found a negative causal loop... Cell 0 must equal its own negation. This is the classic grandfather paradox structure."
