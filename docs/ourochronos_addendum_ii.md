# OUROCHRONOS

## Specification Addendum II: Performance Optimisation and Paradox Diagnosis

### Version 1.2

---

## Abstract

This addendum addresses two critical concerns in the OUROCHRONOS specification: (1) the inherent computational expense of fixed-point search, and (2) the difficulty of explaining *why* paradoxical programs have no consistent execution. We develop formal frameworks for both problems, including static analysis techniques for performance optimisation, compilation strategies to leverage industrial solvers, and a theory of *paradox witnesses* that provide constructive proofs of inconsistency.

---

## Table of Contents

1. [Performance Optimisation Framework](#1-performance-optimisation-framework)
2. [Compilation to Constraint Solvers](#2-compilation-to-constraint-solvers)
3. [Paradox Diagnosis Theory](#3-paradox-diagnosis-theory)
4. [Implementation Strategies](#4-implementation-strategies)
5. [Formal Proofs](#5-formal-proofs)

---

## 1. Performance Optimisation Framework

### 1.1 The Cost Model

We begin by characterising where computational expense arises:

**Definition 1.1 (Execution Cost).** The total cost of executing program Π on input I is:

$$C(\Pi, I) = \sum_{i=1}^{N} C_{\text{epoch}}(\Pi, A_i, I)$$

where N is the number of epochs until convergence (or timeout) and $C_{\text{epoch}}$ is the cost of a single epoch.

The fixed-point search is expensive because:
1. **Epoch count N** may be large (exponential in memory size for adversarial programs)
2. **Epoch cost** scales with program size and memory accesses
3. **Memory comparison** at each epoch boundary costs O(M) where M is memory size

Our optimisation strategy attacks all three factors.

### 1.2 Static Dependency Analysis

**Definition 1.2 (Temporal Dependency Graph).** The temporal dependency graph TDG(Π) = (V, E) is a directed graph where:
- V = Addr (memory addresses)
- E contains edge (a₁, a₂) iff the value written to a₂ may depend on the oracle read of a₁

**Algorithm 1.1 (TDG Construction).**

```
function build_TDG(Π: Program) -> Graph:
    G := empty_graph(vertices = Addr)
    
    // Abstract interpretation: track symbolic provenance
    for instruction in abstract_execute(Π):
        match instruction:
            OracleRead(addr_set) →
                current_provenance := addr_set
            
            PresentWrite(target_set, provenance) →
                for target in target_set:
                    for source in provenance:
                        G.add_edge(source, target)
            
            BinaryOp →
                // Merge provenances
                current_provenance := union of operand provenances
    
    return G
```

**Definition 1.3 (Temporal Core).** The temporal core TC(Π) is the set of addresses participating in temporal feedback:

$$TC(\Pi) = \{a \in \text{Addr} \mid a \text{ is in a cycle in } TDG(\Pi)\}$$

**Theorem 1.1 (Temporal Core Sufficiency).** Fixed-point convergence depends only on cells in the temporal core. Cells outside TC(Π) are determined functions of the core cells.

*Proof.* If address a is not in any cycle, then either:
1. a is never written (constant, irrelevant to fixed point)
2. a is written but depends only on non-temporal values (determined in first epoch)
3. a depends on temporal values but is not itself read by oracle (sink, cannot affect convergence)

In all cases, a's value at the fixed point is determined by the values of TC(Π). □

**Optimisation 1.1 (Core Reduction).** Restrict fixed-point iteration to the temporal core:

```
function optimised_fixed_point(Π, I):
    core := compute_temporal_core(Π)
    
    if core = ∅:
        // Program is trivially consistent
        return epoch(Π, zero_memory(), I)
    
    // Project memory to core
    A_core := zero_memory_over(core)
    
    for iteration in 0..MAX:
        result := epoch(Π, expand(A_core), I)
        P_core := project(result.present, core)
        
        if P_core = A_core:
            return result  // Fixed point on core implies global fixed point
        
        A_core := P_core
    
    return NonConvergent
```

**Complexity Improvement:** If |TC(Π)| << |Addr|, the comparison cost drops from O(M) to O(|TC(Π)|), and the search space shrinks from $W^M$ to $W^{|TC(Π)|}$.

### 1.3 Stratified Evaluation

**Definition 1.4 (Temporal Stratification).** A program Π admits stratification if TDG(Π) is acyclic except for self-loops. The stratification is a topological ordering of the strongly connected components.

**Theorem 1.2 (Stratified Convergence).** If Π admits stratification into k strata, the fixed-point search converges in at most k · W iterations, where W is the value range.

*Proof.* Each stratum's values stabilise before affecting higher strata. Within a stratum, values can cycle but the cycle length is bounded by W. □

**Algorithm 1.2 (Stratified Evaluation).**

```
function stratified_fixed_point(Π, I):
    sccs := tarjan_scc(TDG(Π))
    strata := topological_sort(sccs)
    
    A := zero_memory()
    
    for stratum in strata:
        // Evaluate this stratum to fixed point
        // Lower strata are already stable
        A := stabilise_stratum(Π, I, A, stratum)
    
    return epoch(Π, A, I)

function stabilise_stratum(Π, I, A, stratum):
    for iteration in 0..MAX_STRATUM:
        result := epoch(Π, A, I)
        A_new := A.copy()
        
        changed := false
        for addr in stratum:
            if result.present[addr] ≠ A[addr]:
                A_new[addr] := result.present[addr]
                changed := true
        
        if not changed:
            return A_new
        
        A := A_new
    
    return NonConvergent (within stratum)
```

### 1.4 Incremental Epoch Evaluation

**Definition 1.5 (Change Set).** The change set Δ(A, A') = {a ∈ Addr | A(a) ≠ A'(a)}.

**Definition 1.6 (Dependency Cone).** The forward dependency cone of address set S is:

$$\text{cone}(S) = \{a \in \text{Addr} \mid \exists s \in S. \text{ there is a path from } s \text{ to } a \text{ in } TDG(\Pi)\}$$

**Theorem 1.3 (Incremental Sufficiency).** If anamnesis changes only on Δ, then present can only change on cone(Δ).

**Optimisation 1.2 (Incremental Evaluation).**

```
function incremental_epoch(Π, A_old, A_new, cached_result):
    Δ := change_set(A_old, A_new)
    affected := dependency_cone(Δ)
    
    if affected = ∅:
        return cached_result  // No change possible
    
    // Re-execute only instructions affecting 'affected' cells
    partial_result := partial_epoch(Π, A_new, affected)
    
    return merge(cached_result, partial_result, affected)
```

### 1.5 Memoisation and State Hashing

**Definition 1.7 (Epoch Memoisation).** Cache the mapping A ↦ F(A) for previously computed epochs.

**Challenge:** Memory states are large (M × 64 bits). Direct hashing is expensive.

**Solution:** Hierarchical hashing with Merkle trees.

**Definition 1.8 (Memory Merkle Tree).** Partition memory into blocks of size B. Each block has a hash. The root hash is computed from block hashes.

```
function merkle_hash(memory: Memory, block_size: int) -> Hash:
    blocks := partition(memory, block_size)
    block_hashes := [hash(block) for block in blocks]
    return merkle_root(block_hashes)

function memory_changed(old_hash, new_memory, old_merkle_tree) -> (bool, Set<Block>):
    new_tree := compute_merkle_tree(new_memory)
    if old_hash = new_tree.root:
        return (false, ∅)
    
    // Find differing blocks by comparing tree nodes
    changed_blocks := diff_merkle_trees(old_merkle_tree, new_tree)
    return (true, changed_blocks)
```

**Complexity:** Merkle comparison is O(k log M) where k is the number of changed blocks, rather than O(M) for naive comparison.

### 1.6 Parallel Fixed-Point Search

The nondeterministic semantics permit parallel exploration:

**Algorithm 1.3 (Parallel Multi-Seed Search).**

```
function parallel_fixed_point(Π, I, num_workers):
    seeds := generate_diverse_seeds(num_workers)
    
    results := parallel_map(seeds, λ seed:
        return search_from_seed(Π, I, seed)
    )
    
    // Return first successful result
    for result in results:
        if result.status = Consistent:
            return result
    
    // All failed: aggregate diagnostics
    return aggregate_failures(results)
```

**Work Stealing:** When one worker finds a cycle, it can share information with others to avoid redundant exploration.

```
function cooperative_search(Π, I, num_workers):
    shared_visited := ConcurrentHashSet()
    shared_cycles := ConcurrentQueue()
    
    workers := spawn(num_workers, λ worker_id:
        A := seed_for_worker(worker_id)
        
        while true:
            if shared_visited.contains(hash(A)):
                A := perturb_avoiding(A, shared_visited)
                continue
            
            shared_visited.add(hash(A))
            result := epoch(Π, A, I)
            
            if result.present = A:
                return Consistent(result)
            
            if shared_visited.contains(hash(result.present)):
                shared_cycles.push(extract_cycle(...))
                A := perturb_avoiding(A, shared_visited)
            else:
                A := result.present
    )
    
    return first_success_or_aggregate(workers)
```

### 1.7 Acceleration Techniques

**Definition 1.9 (Widening Operator).** For memory states A₁, A₂, the widening A₁ ∇ A₂ is:

$$(A_1 \triangledown A_2)(a) = \begin{cases} A_1(a) & \text{if } A_1(a) = A_2(a) \\ \top & \text{if } A_2(a) > A_1(a) \text{ (ascending)} \\ \bot & \text{if } A_2(a) < A_1(a) \text{ (descending)} \end{cases}$$

Widening accelerates convergence by jumping to extremal values.

**Definition 1.10 (Extrapolation).** Given trajectory A₀, A₁, A₂, ..., predict the fixed point:

$$\hat{A} = \text{extrapolate}(A_0, A_1, ..., A_k)$$

For linear recurrences, the fixed point can be computed exactly.

**Algorithm 1.4 (Accelerated Iteration with Extrapolation).**

```
function accelerated_fixed_point(Π, I):
    trajectory := []
    A := zero_memory()
    
    for iteration in 0..MAX:
        result := epoch(Π, A, I)
        trajectory.append((A, result.present))
        
        if result.present = A:
            return Consistent(result)
        
        // Attempt extrapolation every k iterations
        if iteration % EXTRAPOLATION_INTERVAL = 0 and len(trajectory) >= 3:
            predicted := extrapolate_fixed_point(trajectory)
            if predicted is not None:
                // Verify prediction
                verify_result := epoch(Π, predicted, I)
                if verify_result.present = predicted:
                    return Consistent(verify_result)
        
        // Detect and accelerate monotonic cells
        A := accelerate_monotonic(trajectory, result.present)
    
    return NonConvergent

function extrapolate_fixed_point(trajectory):
    // Analyse per-cell behaviour
    predictions := []
    
    for addr in Addr:
        values := [A[addr] for (A, _) in trajectory]
        
        if is_constant(values):
            predictions[addr] := values[0]
        elif is_arithmetic_progression(values):
            // Solve a + n*d = a + (n+1)*d for fixed point
            // Only possible if d = 0, already handled
            predictions[addr] := None
        elif is_geometric(values):
            // Fixed point is limit of geometric series
            predictions[addr] := compute_geometric_limit(values)
        elif is_periodic(values):
            // No fixed point possible for this cell with this period
            return None
        else:
            predictions[addr] := None
    
    if any(p is None for p in predictions):
        return None
    
    return predictions
```

### 1.8 Contraction Detection

**Definition 1.11 (Contractive Program).** Π is contractive for input I with factor c < 1 if:

$$\forall A_1, A_2. \quad d(F_{\Pi,I}(A_1), F_{\Pi,I}(A_2)) \leq c \cdot d(A_1, A_2)$$

where d is the Hamming distance.

**Theorem 1.4 (Banach Fixed-Point).** Contractive programs converge in O(log(M·W) / log(1/c)) iterations.

**Algorithm 1.5 (Contraction Detection).**

```
function estimate_contraction_factor(Π, I, samples=100):
    max_ratio := 0
    
    for _ in 0..samples:
        A1 := random_memory()
        A2 := random_memory()
        
        d_in := hamming_distance(A1, A2)
        if d_in = 0:
            continue
        
        P1 := epoch(Π, A1, I).present
        P2 := epoch(Π, A2, I).present
        d_out := hamming_distance(P1, P2)
        
        ratio := d_out / d_in
        max_ratio := max(max_ratio, ratio)
    
    if max_ratio < 1.0:
        return Contractive(max_ratio)
    else:
        return NotContractive(max_ratio)
```

**Optimisation 1.3 (Contractive Fast Path).**

```
function fixed_point_with_contraction_check(Π, I):
    contraction := estimate_contraction_factor(Π, I)
    
    if contraction.is_contractive:
        c := contraction.factor
        max_iterations := ceil(log(M * W) / log(1/c)) + SAFETY_MARGIN
        return bounded_iteration(Π, I, max_iterations)
    else:
        return general_fixed_point(Π, I)
```

---

## 2. Compilation to Constraint Solvers

### 2.1 Motivation

Rather than iteratively searching for fixed points, we can *compile* the fixed-point condition to a constraint satisfaction problem and leverage industrial SAT/SMT solvers.

**Key Insight:** The fixed-point condition P = A, combined with the epoch semantics P = F(A), yields the constraint:

$$A = F(A)$$

This is a system of equations over the memory cells, which can be encoded in propositional or first-order logic.

### 2.2 Compilation to SAT

**Definition 2.1 (Bit-Blasted Memory).** Represent each memory cell A[a] as 64 Boolean variables: $A_a = (b_{a,63}, b_{a,62}, ..., b_{a,0})$.

**Definition 2.2 (Epoch Relation).** The epoch relation R_Π(A, P) holds iff executing Π with anamnesis A produces present P:

$$R_\Pi(A, P) \iff P = F_{\Pi}(A)$$

**Theorem 2.1.** For programs without unbounded loops, R_Π is expressible as a Boolean formula of size polynomial in program length and memory size.

**Algorithm 2.1 (Compilation to SAT).**

```
function compile_to_sat(Π: Program) -> CNF:
    // Create variables for anamnesis (A) and present (P)
    A_vars := create_bitvector_vars("A", num_cells=M, bits=64)
    P_vars := create_bitvector_vars("P", num_cells=M, bits=64)
    
    // Symbolically execute program
    constraints := []
    symbolic_state := initial_symbolic_state(A_vars)
    
    for instruction in Π.instructions:
        new_constraints := symbolic_execute(instruction, symbolic_state, A_vars, P_vars)
        constraints.extend(new_constraints)
    
    // Add fixed-point constraint: A = P for all cells
    for addr in 0..M:
        constraints.append(A_vars[addr] = P_vars[addr])
    
    // Convert to CNF via Tseitin transformation
    return tseitin_transform(And(constraints))

function symbolic_execute(instruction, state, A_vars, P_vars):
    match instruction:
        OracleRead(addr_expr):
            addr := evaluate_symbolic(addr_expr, state)
            value := A_vars[addr]  // Symbolic read from anamnesis
            state.stack.push(value)
            return []
        
        PresentWrite(addr_expr, val_expr):
            addr := evaluate_symbolic(addr_expr, state)
            value := evaluate_symbolic(val_expr, state)
            // P[addr] = value
            return [P_vars[addr] == value]
        
        Add:
            b := state.stack.pop()
            a := state.stack.pop()
            state.stack.push(bitvector_add(a, b))
            return []
        
        // ... other instructions
```

**Handling Loops:** For bounded loops, unroll to maximum iteration count. For unbounded loops, we require programmer annotation of loop bounds or use abstract interpretation to infer them.

### 2.3 Compilation to SMT

SMT (Satisfiability Modulo Theories) allows richer constraints:

**Definition 2.3 (SMT Encoding).** Use the theory of fixed-width bit-vectors (QF_BV) for memory operations, and the theory of arrays (QF_ABV) for memory itself.

```smt2
; Declare memory arrays
(declare-fun A () (Array (_ BitVec 16) (_ BitVec 64)))
(declare-fun P () (Array (_ BitVec 16) (_ BitVec 64)))

; Epoch semantics: P = F(A)
; For each present write in the program:
(assert (= (select P #x0000) 
           (bvadd (select A #x0000) (select A #x0001))))

; Fixed-point constraint: A = P
(assert (= A P))

; Solve
(check-sat)
(get-model)
```

**Algorithm 2.2 (Compilation to SMT).**

```
function compile_to_smt(Π: Program) -> SMTFormula:
    solver := create_smt_solver(logic="QF_ABV")
    
    A := solver.declare_array("A", index_sort=BitVec(16), elem_sort=BitVec(64))
    P := solver.declare_array("P", index_sort=BitVec(16), elem_sort=BitVec(64))
    
    // Symbolic execution with array operations
    writes := symbolic_execute_for_smt(Π, A)
    
    // Each write becomes a constraint on P
    for (addr, value) in writes:
        solver.assert(Select(P, addr) == value)
    
    // Cells not written retain initial value (0)
    written_addrs := {addr for (addr, _) in writes}
    for addr in 0..M:
        if addr not in written_addrs:
            solver.assert(Select(P, addr) == 0)
    
    // Fixed-point constraint
    solver.assert(A == P)
    
    return solver

function solve_fixed_point_smt(Π, I):
    solver := compile_to_smt(Π)
    
    if solver.check() == SAT:
        model := solver.get_model()
        A_solution := extract_memory(model, "A")
        return Consistent(epoch(Π, A_solution, I))
    else:
        // UNSAT - extract proof of unsatisfiability
        proof := solver.get_proof()
        diagnosis := translate_proof_to_paradox(proof)
        return Paradoxical(diagnosis)
```

### 2.4 Incremental SMT for Multiple Queries

**Optimisation 2.1 (Incremental Solving).** For exploring multiple fixed points or different inputs, use incremental SMT:

```
function enumerate_fixed_points_smt(Π, max_count):
    solver := compile_to_smt(Π)
    fixed_points := []
    
    while len(fixed_points) < max_count:
        if solver.check() == SAT:
            model := solver.get_model()
            A_solution := extract_memory(model, "A")
            fixed_points.append(A_solution)
            
            // Block this solution
            solver.assert(Not(A == A_solution))
        else:
            break  // No more solutions
    
    return fixed_points
```

### 2.5 Performance Comparison

| Approach | Time Complexity | Space Complexity | Best For |
|----------|-----------------|------------------|----------|
| Naive Iteration | O(W^M · E) | O(M) | Small programs |
| Core Reduction | O(W^|TC| · E) | O(|TC|) | Programs with small core |
| Stratified | O(k · W · E) | O(M) | Acyclic dependency |
| SAT Compilation | O(SAT solver) | O(M · 64 · P) | Moderate programs |
| SMT Compilation | O(SMT solver) | O(M · P) | Complex arithmetic |
| Parallel Search | O(W^M · E / workers) | O(M · workers) | Large search space |

Where E = epoch cost, P = program size, M = memory size, W = value range.

---

## 3. Paradox Diagnosis Theory

### 3.1 The Diagnosis Problem

**Definition 3.1 (Paradox).** Program Π is paradoxical for input I if FP(Π, I) = ∅, i.e., no consistent execution exists.

**Problem:** Given a paradoxical program, explain *why* no fixed point exists in terms meaningful to the programmer.

**Insight:** A paradox arises when the constraints imposed by the program are mutually unsatisfiable. The diagnosis should identify the *minimal set of conflicting constraints*.

### 3.2 Paradox Witnesses

**Definition 3.2 (Paradox Witness).** A paradox witness for program Π is a structure W that certifies FP(Π, I) = ∅.

We develop several forms of witnesses, each providing different explanatory power.

#### 3.2.1 Oscillation Witnesses

**Definition 3.3 (k-Cycle Witness).** A k-cycle witness is a sequence of memory states (A₀, A₁, ..., A_{k-1}) such that:
- F(Aᵢ) = A_{(i+1) mod k} for all i
- All Aᵢ are distinct
- k > 1

**Theorem 3.1.** If a k-cycle witness exists with k > 1, no fixed point exists (within the cycle's basin of attraction).

**Explanation Power:** "These k memory states form a cycle: each one leads to the next, with no escape. The program oscillates forever between them."

**Visualisation:**
```
A₀ ──epoch──▶ A₁ ──epoch──▶ A₂ ──epoch──▶ A₀ (cycle!)

Cell 0: 0 → 1 → 0 → 1 → ... (oscillates)
Cell 1: 5 → 5 → 5 → 5 → ... (stable but trapped in cycle)
```

#### 3.2.2 Divergence Witnesses

**Definition 3.4 (Divergence Witness).** A divergence witness is a sequence (A₀, A₁, A₂, ...) such that:
- F(Aᵢ) = A_{i+1}
- The sequence is unbounded in some cell: ∃a. lim_{i→∞} Aᵢ(a) = ∞ (mod W, meaning cyclic traversal of entire value range)

**Theorem 3.2.** If a divergence witness exists, no fixed point is reachable from A₀.

**Explanation Power:** "Cell a grows (or shrinks) without bound. It can never stabilise because each epoch pushes it further."

#### 3.2.3 Constraint Conflict Witnesses

**Definition 3.5 (Conflict Core).** A conflict core is a minimal set of cells C ⊆ Addr such that:
- The projection of the fixed-point constraint to C is unsatisfiable
- No proper subset of C is unsatisfiable

**Definition 3.6 (Conflict Witness).** A conflict witness is a pair (C, Proof) where:
- C is a conflict core
- Proof is a derivation showing unsatisfiability

**Algorithm 3.1 (Conflict Core Extraction via SMT).**

```
function extract_conflict_core(Π, I) -> ConflictCore:
    solver := compile_to_smt_with_tracking(Π)
    
    // Use assumption-based UNSAT core extraction
    assumptions := []
    for addr in 0..M:
        // Each fixed-point constraint A[addr] = P[addr] is a separate assumption
        assumption := solver.create_assumption(f"fp_{addr}", A[addr] == P[addr])
        assumptions.append(assumption)
    
    result := solver.check(assumptions)
    
    if result == SAT:
        return None  // Not paradoxical
    
    // Get minimal UNSAT core
    core_assumptions := solver.get_unsat_core()
    core_addrs := {parse_addr(a.name) for a in core_assumptions}
    
    return ConflictCore(core_addrs, solver.get_proof())
```

**Explanation Power:** "The fixed-point constraints for cells {a₁, a₂, a₃} are mutually unsatisfiable. Here's why: [human-readable proof]."

#### 3.2.4 Causal Loop Witnesses

**Definition 3.7 (Negative Causal Loop).** A negative causal loop is a cycle in the causal graph where the parity of negations along the cycle is odd.

**Example:** Cell 0's new value is the negation of cell 0's old value.
```
A[0] ──read──▶ NOT ──write──▶ P[0]
  ▲                              │
  └──────── fixed point ─────────┘

Constraint: A[0] = P[0] = NOT(A[0])
This is unsatisfiable.
```

**Algorithm 3.2 (Negative Loop Detection).**

```
function find_negative_loops(Π) -> List<NegativeLoop>:
    // Build causal graph with edge polarities
    G := build_polarised_causal_graph(Π)
    
    // Find all cycles
    cycles := find_all_cycles(G)
    
    // Filter to negative (odd parity) cycles
    negative_loops := []
    for cycle in cycles:
        parity := 0
        for edge in cycle.edges:
            if edge.is_negating:
                parity := 1 - parity
        
        if parity == 1:  // Odd number of negations
            negative_loops.append(cycle)
    
    return negative_loops
```

**Explanation Power:** "Cell 0 must equal its own negation. This is the classic grandfather paradox structure."

### 3.3 Hierarchical Diagnosis

Different witness types have different computational costs and explanatory power:

| Witness Type | Detection Cost | Explanation Quality | Use Case |
|--------------|---------------|---------------------|----------|
| k-Cycle | O(k · E) | Good (concrete states) | Runtime detection |
| Divergence | O(sampling) | Moderate (trend) | Monotonic failures |
| Conflict Core | O(SMT) | Excellent (minimal) | Deep analysis |
| Negative Loop | O(|E|) static | Excellent (structural) | Common patterns |

**Algorithm 3.3 (Hierarchical Diagnosis).**

```
function diagnose_paradox(Π, I, trajectory) -> Diagnosis:
    // Level 1: Check for cycles in observed trajectory
    cycle := detect_cycle_in_trajectory(trajectory)
    if cycle is not None:
        return CycleDiagnosis(cycle)
    
    // Level 2: Check for monotonic divergence
    divergence := detect_divergence(trajectory)
    if divergence is not None:
        return DivergenceDiagnosis(divergence)
    
    // Level 3: Static negative loop analysis
    negative_loops := find_negative_loops(Π)
    if negative_loops:
        return NegativeLoopDiagnosis(negative_loops)
    
    // Level 4: Full SMT conflict analysis
    conflict_core := extract_conflict_core(Π, I)
    if conflict_core is not None:
        return ConflictCoreDiagnosis(conflict_core)
    
    // Level 5: Unknown (rare, indicates analysis limitation)
    return UnknownDiagnosis(trajectory)
```

### 3.4 Human-Readable Proof Translation

**Definition 3.8 (Proof Term).** We define a proof language for paradox explanations:

```
Proof p ::= Cycle(A₀, ..., A_{k-1})                    -- Cyclic witness
          | Diverge(a, direction, rate)               -- Unbounded cell
          | Conflict(C, reason)                       -- Constraint conflict
          | NegLoop(path)                             -- Negative causal loop
          | Compose(p₁, p₂)                           -- Combined reasons
          | Because(cell, value, depends_on)          -- Causal explanation
```

**Algorithm 3.4 (Proof Rendering).**

```
function render_proof(proof: Proof) -> String:
    match proof:
        Cycle(states):
            lines := ["The program enters a cycle of length {len(states)}:"]
            for i, state in enumerate(states):
                next_i := (i + 1) % len(states)
                changed := diff_cells(state, states[next_i])
                lines.append(f"  Epoch {i}: {summarise(state)}")
                lines.append(f"    Changes: {changed}")
            lines.append("  → Returns to initial state. No fixed point possible.")
            return join(lines, "\n")
        
        NegLoop(path):
            lines := ["Found a negative causal loop:"]
            for i, (src, dst, op) in enumerate(path.edges):
                lines.append(f"  {i+1}. Cell {src} → {op} → Cell {dst}")
            lines.append(f"  This creates constraint: A[{path[0]}] = NOT(A[{path[0]}])")
            lines.append("  This is unsatisfiable (grandfather paradox).")
            return join(lines, "\n")
        
        Conflict(core, reason):
            lines := [f"Conflict among cells {core}:"]
            lines.append(render_smt_proof(reason))
            lines.append("Suggestion: Review the logic connecting these cells.")
            return join(lines, "\n")
        
        // ... other cases
```

**Example Output:**
```
PARADOX DIAGNOSIS
═════════════════

Found a negative causal loop:

  1. Cell 0 (CHOICE) → read via ORACLE → temp₁
  2. temp₁ → EQUALS 0 → temp₂  
  3. temp₂ → IF branch selection → temp₃
  4. temp₃ → XOR 1 → new value
  5. new value → write to Cell 0 (CHOICE)

This creates the constraint: CHOICE = 1 - CHOICE

In other words: if CHOICE is 0, the program writes 1.
                if CHOICE is 1, the program writes 0.

No value of CHOICE can satisfy CHOICE = F(CHOICE).

This is the classic "grandfather paradox" structure: the program's
action on receiving a value necessarily produces a different value.

SUGGESTION: Ensure that at least one branch writes the same value
it received from ORACLE, creating a self-consistent path.

Location: lines 15-28 in paradox_example.ouro
```

### 3.5 Paradox Classification

**Definition 3.9 (Paradox Taxonomy).**

| Class | Structure | Example | Fix Strategy |
|-------|-----------|---------|--------------|
| Type I: Negation | A = ¬A | Grandfather paradox | Add identity branch |
| Type II: Offset | A = A + k (k ≠ 0) | Counter without reset | Add termination condition |
| Type III: Permutation | A = π(A), π has no fixed point | Rotation paradox | Use identity permutation |
| Type IV: Conditional | A = f(A) where f has no fixed point for any branch | Complex branching | Ensure one branch is consistent |
| Type V: Emergent | Fixed-point absence emerges from interaction of multiple cells | Distributed paradox | Analyse cell dependencies |

**Algorithm 3.5 (Paradox Classification).**

```
function classify_paradox(diagnosis: Diagnosis) -> ParadoxClass:
    match diagnosis:
        NegativeLoopDiagnosis(loop) if loop.is_pure_negation:
            return TypeI_Negation(loop)
        
        DivergenceDiagnosis(cell, Monotonic(direction)):
            offset := estimate_offset(cell)
            return TypeII_Offset(cell, offset)
        
        CycleDiagnosis(cycle) if is_permutation_cycle(cycle):
            return TypeIII_Permutation(cycle)
        
        ConflictCoreDiagnosis(core) if |core| == 1:
            return TypeIV_Conditional(core[0])
        
        ConflictCoreDiagnosis(core) if |core| > 1:
            return TypeV_Emergent(core)
        
        _:
            return Unknown
```

### 3.6 Automatic Repair Suggestions

**Definition 3.10 (Repair).** A repair for paradoxical program Π is a minimal modification Π' such that FP(Π', I) ≠ ∅.

**Algorithm 3.6 (Repair Suggestion).**

```
function suggest_repairs(Π, diagnosis) -> List<Repair>:
    repairs := []
    
    match diagnosis:
        TypeI_Negation(loop):
            // Add a branch that preserves the value
            cell := loop.cells[0]
            repairs.append(Repair(
                description = f"Add identity branch for cell {cell}",
                code = f"""
IF ORACLE[{cell}] = TARGET_VALUE THEN
    PROPHECY[{cell}] <- TARGET_VALUE  ;; Self-consistent
ELSE
    ;; ... existing logic
END
"""
            ))
        
        TypeII_Offset(cell, offset):
            // Add termination condition
            repairs.append(Repair(
                description = f"Add termination when cell {cell} reaches target",
                code = f"""
IF ORACLE[{cell}] >= MAX_VALUE THEN
    PROPHECY[{cell}] <- MAX_VALUE  ;; Clamp to fixed point
ELSE
    PROPHECY[{cell}] <- ORACLE[{cell}] + {offset}
END
"""
            ))
        
        // ... other repair strategies
    
    return repairs
```

---

## 4. Implementation Strategies

### 4.1 Tiered Execution Architecture

```
                              ┌─────────────────┐
                              │  OUROCHRONOS    │
                              │    Compiler     │
                              └────────┬────────┘
                                       │
                    ┌──────────────────┼──────────────────┐
                    │                  │                  │
                    ▼                  ▼                  ▼
           ┌───────────────┐  ┌───────────────┐  ┌───────────────┐
           │ Tier 1: Fast  │  │ Tier 2: SMT   │  │ Tier 3: Full  │
           │  Iteration    │  │  Compilation  │  │  Diagnosis    │
           └───────────────┘  └───────────────┘  └───────────────┘
                    │                  │                  │
                    │ Quick check      │ Exact solve      │ Deep analysis
                    │ (< 1000 epochs)  │ (SAT/SMT)        │ (all witnesses)
                    ▼                  ▼                  ▼
              ┌──────────┐       ┌──────────┐       ┌──────────┐
              │ Success? │──no──▶│ Success? │──no──▶│ Diagnose │
              └──────────┘       └──────────┘       └──────────┘
                   │ yes              │ yes              │
                   ▼                  ▼                  ▼
              ┌──────────┐       ┌──────────┐       ┌──────────┐
              │  Output  │       │  Output  │       │  Report  │
              └──────────┘       └──────────┘       └──────────┘
```

**Tier 1 (Fast Iteration):**
- Apply all static optimisations (core reduction, stratification)
- Run up to 1000 epochs with incremental evaluation
- If converges: return result
- If cycle detected: proceed to Tier 2

**Tier 2 (SMT Compilation):**
- Compile to SMT (or SAT for simpler programs)
- Use industrial solver (Z3, CVC5)
- If SAT: extract fixed point, verify with epoch execution
- If UNSAT: extract proof, proceed to Tier 3

**Tier 3 (Full Diagnosis):**
- Extract UNSAT core
- Classify paradox
- Generate human-readable explanation
- Suggest repairs

### 4.2 Runtime Metrics and Profiling

```
struct ExecutionMetrics {
    // Timing
    total_time: Duration,
    epoch_times: Vec<Duration>,
    solver_time: Duration,
    
    // Iteration
    epochs_executed: u64,
    instructions_executed: u64,
    
    // Memory
    cells_read: u64,
    cells_written: u64,
    temporal_core_size: usize,
    
    // Optimisation impact
    iterations_saved_by_core_reduction: u64,
    cache_hits: u64,
    cache_misses: u64,
    
    // Convergence
    convergence_rate: Vec<f64>,  // Hamming distance over epochs
    estimated_remaining_epochs: Option<u64>,
}

function profile_execution(Π, I) -> (Result, ExecutionMetrics):
    metrics := ExecutionMetrics::new()
    
    with timer(metrics.total_time):
        // ... execution with instrumentation
    
    return (result, metrics)
```

### 4.3 Caching Strategy

```
struct CompilerCache {
    // Static analysis results
    temporal_cores: Map<ProgramHash, Set<Addr>>,
    stratifications: Map<ProgramHash, Vec<Set<Addr>>>,
    negative_loops: Map<ProgramHash, Vec<NegativeLoop>>,
    
    // SMT encodings (expensive to rebuild)
    smt_encodings: Map<ProgramHash, SMTFormula>,
    
    // Epoch memoisation (per-program, per-input)
    epoch_results: Map<(ProgramHash, InputHash, MemoryHash), EpochResult>,
}

impl CompilerCache {
    function get_or_compute_core(&mut self, Π) -> Set<Addr>:
        hash := hash(Π)
        if hash not in self.temporal_cores:
            self.temporal_cores[hash] := compute_temporal_core(Π)
        return self.temporal_cores[hash]
    
    function get_cached_epoch(&self, Π, I, A) -> Option<EpochResult>:
        key := (hash(Π), hash(I), hash(A))
        return self.epoch_results.get(key)
    
    function cache_epoch(&mut self, Π, I, A, result):
        key := (hash(Π), hash(I), hash(A))
        self.epoch_results[key] := result
}
```

### 4.4 Integration with Diagnostic Mode

```
function execute_with_full_diagnostics(Π, I):
    cache := CompilerCache::new()
    metrics := ExecutionMetrics::new()
    
    // Phase 1: Static analysis
    core := cache.get_or_compute_core(Π)
    strata := cache.get_or_compute_stratification(Π)
    neg_loops := cache.get_or_compute_negative_loops(Π)
    
    // Early paradox detection
    if neg_loops:
        return DiagnosticResult(
            outcome = EarlyParadoxDetection,
            diagnosis = NegativeLoopDiagnosis(neg_loops),
            metrics = metrics,
        )
    
    // Phase 2: Optimised iteration
    result := optimised_fixed_point(Π, I, core, strata, &metrics, &cache)
    
    match result:
        Consistent(output, fixed_point):
            return DiagnosticResult(
                outcome = Consistent,
                output = output,
                fixed_point = fixed_point,
                metrics = metrics,
            )
        
        Cyclic(cycle):
            diagnosis := diagnose_cycle(cycle)
            return DiagnosticResult(
                outcome = Paradoxical,
                diagnosis = diagnosis,
                suggestions = suggest_repairs(Π, diagnosis),
                metrics = metrics,
            )
        
        Timeout(trajectory):
            // Fall back to SMT
            smt_result := solve_with_smt(cache.get_or_compute_smt(Π), I)
            
            match smt_result:
                SAT(model):
                    // SMT found solution iteration missed
                    fixed_point := extract_memory(model)
                    return DiagnosticResult(
                        outcome = Consistent,
                        output = epoch(Π, fixed_point, I).output,
                        fixed_point = fixed_point,
                        metrics = metrics,
                        note = "Found by SMT after iteration timeout",
                    )
                
                UNSAT(proof):
                    diagnosis := extract_and_classify_paradox(proof)
                    return DiagnosticResult(
                        outcome = Paradoxical,
                        diagnosis = diagnosis,
                        suggestions = suggest_repairs(Π, diagnosis),
                        metrics = metrics,
                    )
```

---

## 5. Formal Proofs

### 5.1 Proof of Optimisation Correctness

**Theorem 5.1 (Core Reduction Correctness).** If A* is a fixed point of F restricted to TC(Π), then the extension of A* to all addresses (with non-core cells computed from one epoch) is a fixed point of F.

*Proof.*

Let A*_{core} be a fixed point on TC(Π). Define A* as:
$$A^*(a) = \begin{cases} A^*_{core}(a) & \text{if } a \in TC(\Pi) \\ F(A^*_{core})(a) & \text{if } a \notin TC(\Pi) \end{cases}$$

We show A* = F(A*).

For a ∈ TC(Π): By assumption, A*_{core}(a) = F(A*_{core})(a), and the value at a depends only on core cells (otherwise a would not be in a cycle), so F(A*)(a) = F(A*_{core})(a) = A*(a). ✓

For a ∉ TC(Π): By definition, A*(a) = F(A*_{core})(a). Since a's value depends only on cells reachable from a in TDG(Π), and these are either in TC(Π) (unchanged) or also outside (also computed from F(A*_{core})), we have F(A*)(a) = F(A*_{core})(a) = A*(a). ✓

Thus A* = F(A*). □

### 5.2 Proof of SMT Compilation Correctness

**Theorem 5.2.** The SMT encoding is equisatisfiable with the fixed-point existence problem.

*Proof.*

(⟹) Suppose A* is a fixed point. Then F(A*) = A*. The SMT constraints encode:
1. P = F(A) (epoch semantics)
2. A = P (fixed-point constraint)

Setting A := A* and P := A* satisfies both: P = F(A*) = A* = A. ✓

(⟸) Suppose the SMT formula is satisfiable with model (A, P). Then P = F(A) (constraint 1) and A = P (constraint 2). Thus A = P = F(A), so A is a fixed point. ✓

□

### 5.3 Proof of Paradox Witness Soundness

**Theorem 5.3.** If a k-cycle witness exists for k > 1, no fixed point exists within the cycle's basin of attraction.

*Proof.*

Let (A₀, A₁, ..., A_{k-1}) be a k-cycle. Suppose for contradiction that A* is a fixed point reachable from A₀ via iteration.

Since F(Aᵢ) = A_{(i+1) mod k}, the iteration sequence from A₀ is:
A₀ → A₁ → A₂ → ... → A_{k-1} → A₀ → A₁ → ...

This sequence never reaches A* (unless A* = Aᵢ for some i).

If A* = Aᵢ, then F(A*) = F(Aᵢ) = A_{(i+1) mod k} ≠ Aᵢ = A* (since all Aⱼ are distinct and k > 1).

This contradicts A* being a fixed point. □

**Theorem 5.4.** If a negative causal loop exists for a single cell, no fixed point exists.

*Proof.*

A negative causal loop on cell a means:
$$P(a) = \neg A(a)$$ (or more generally, $P(a) = f(A(a))$ where f has no fixed point)

The fixed-point constraint requires A(a) = P(a) = ¬A(a).

For any value v ∈ Val, v ≠ ¬v (since ¬0 = W-1 ≠ 0 and ¬v ≠ v for all v).

Thus no assignment to A(a) satisfies the constraint. □

---

## Appendix A: Optimisation Impact Benchmarks

| Program Type | Naive Epochs | With Core Reduction | With Stratification | With SMT |
|--------------|--------------|---------------------|---------------------|----------|
| Trivially consistent | 1 | 0 (detected statically) | 0 | - |
| Self-fulfilling (1 cell) | 2 | 2 | 2 | 1 (direct) |
| Fibonacci bootstrap | 2 | 2 | 2 | 1 |
| SAT (10 vars, SAT) | ~100-1000 | ~50-500 | - | 1 |
| SAT (10 vars, UNSAT) | timeout | timeout | - | 1 + proof |
| TQBF (5 vars) | ~10000 | ~5000 | - | 1 |
| Grandfather paradox | ∞ (oscillates) | ∞ | detected statically | UNSAT |

---

## Appendix B: Integration Points

### B.1 External Solver Integration

```
trait ConstraintSolver {
    fn assert(&mut self, constraint: Constraint);
    fn check(&mut self) -> SolverResult;
    fn get_model(&self) -> Option<Model>;
    fn get_unsat_core(&self) -> Option<Vec<Constraint>>;
    fn get_proof(&self) -> Option<Proof>;
}

impl ConstraintSolver for Z3Solver { ... }
impl ConstraintSolver for CVC5Solver { ... }
impl ConstraintSolver for BoolectorSolver { ... }

fn get_default_solver() -> Box<dyn ConstraintSolver> {
    // Prefer Z3 for proof production, CVC5 for incremental solving
    if cfg!(feature = "z3") {
        Box::new(Z3Solver::new())
    } else if cfg!(feature = "cvc5") {
        Box::new(CVC5Solver::new())
    } else {
        Box::new(FallbackIterativeSolver::new())
    }
}
```

### B.2 Proof Format Interoperability

Support for standard proof formats:
- **LFSC** (Logical Framework with Side Conditions) - CVC5
- **Alethe** - standardised SMT proof format
- **DRAT** - for SAT proofs

```
enum ProofFormat {
    LFSC(LFSCProof),
    Alethe(AletheProof),
    DRAT(DRATProof),
    Native(OurochronosProof),
}

fn translate_proof(external: ProofFormat) -> OurochronosProof {
    match external {
        LFSC(p) => translate_lfsc(p),
        Alethe(p) => translate_alethe(p),
        DRAT(p) => translate_drat(p),
        Native(p) => p,
    }
}
```

---

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-01-02 | Initial specification |
| 1.1 | 2026-01-02 | Addendum I: temporal theory, nondeterminism, types, diagnostics |
| 1.2 | 2026-01-02 | Addendum II: performance optimisation, paradox diagnosis |

---

*"To find the fixed point is to solve the equation of time. To prove it absent is to demonstrate the impossibility of the timeline itself."*

— OUROCHRONOS Design Philosophy, Final
