# Ourochronos Theory & Design

> "The future causes the past just as much as the past causes the future."

This document details the theoretical foundations, mathematical semantics, and advanced implementation strategies of the Ourochronos programming language.

## 1. The Computational Model

### 1.1 Closed Timelike Curves (CTCs)
Ourochronos incorporates the physics of **Closed Timelike Curves** directly into its execution model. Specifically, it adheres to the **Deutschian CTC Model** (David Deutsch, 1991), which resolves temporal paradoxes through a self-consistency requirement.

In this model, a computation is not a linear transformation from input to output, but a search for a **Fixed Point** in the state space. A valid execution history is one where the state entering the time loop (from the future) is identical to the state exiting the time loop (to the past).

### 1.2 The Fixed-Point Condition
Let $\Sigma$ be the set of possible memory states. The execution of a program $P$ can be modeled as a function $F_P: \Sigma \to \Sigma$.
*   Input State ($A$): The "Anamnesis" (memory of the future).
*   Output State ($Present$): The result of executing $P$ given $A$.

A valid timeline is a state $S \in \Sigma$ such that:
$$ S = F_P(S) $$

Execution in Ourochronos is the process of finding such an $S$.

## 2. Mathematical Semantics

### 2.1 Domain Theory
The memory state space $\Sigma$ forms a **Complete Partial Order (CPO)** under the information ordering $\sqsubseteq$.
*   $\bot$ represents "undefined" or "unknown" memory.
*   $x \sqsubseteq y$ if $x$ is less defined than or equal to $y$.

By the **Kleene Fixed-Point Theorem**, if $F_P$ is Scott-continuous, the least fixed point can be found by interating:
$$ S = \bigsqcup_{n=0}^{\infty} F_P^n(\bot) $$
This provides the theoretical basis for the "Constructive Iteration" execution mode.

### 2.2 Banach Fixed-Point Theorem
If the program $F_P$ is **Contractive** (i.e., information differences diminish over time) on the metric space of memory, the **Banach Fixed-Point Theorem** guarantees a *unique* fixed point that can be found via iteration from *any* starting state.
*   Convergence Rate: $O(\log(\epsilon^{-1}))$.
*   Implication: "Stable" time loops converge exponentially fast.

## 3. Complexity Class
Ourochronos is not merely Turing Complete; its temporal features place it in a distinct complexity class.

### 3.1 Theorem: OURO = PSPACE
**Theorem (Aaronson-Watrous, 2009)**: A P-computer with access to a CTC cannot compute anything beyond PSPACE, but can solve any problem in PSPACE in polynomial time.

*   **Proof Sketch**:
    *   **OURO $\subseteq$ PSPACE**: A PSPACE machine can iterate through all temporal states (exponential size, but polynomial depth) to verify a fixed point.
    *   **PSPACE $\subseteq$ OURO**: A logic formula (QBF) can be encoded as a time loop where the "future" guesses the existential witnesses and the "past" verifies the universal quantifiers.

This implies Ourochronos programs can efficiently solve NP-complete problems (like SAT or Traveling Salesman) by "guessing" the answer via `ORACLE` and verifying it.

## 4. Paradox Theory

When $S = F_P(S)$ has no solution, we encounter a **Paradox**.

### 4.1 Taxonomy of Paradoxes
1.  **Grandfather Paradox (Oscillation)**:
    *   Structure: $x_{next} = \neg x_{prev}$
    *   Result: The state flips periodically (e.g., $0 \to 1 \to 0$).
    *   Witness: A cycle $C = (S_1, S_2, \dots, S_k)$ where $F(S_i) = S_{i+1}$.

2.  **Divergence (Infinite Energy)**:
    *   Structure: $x_{next} = x_{prev} + 1$
    *   Result: The value grows indefinitely (modulo $2^{64}$).
    *   Witness: A monotonic trend where $S_{t+1} > S_t$.

3.  **Trivial Consistency (Identity)**:
    *   Structure: $x_{next} = x_{prev}$
    *   Result: $S = S$ is always true. Any value is valid.
    *   Note: While consistent, this represents "Unconstrained" timelines. Ourochronos selects the lexicographically minimal fixed point in these cases.

### 4.2 SMT Diagnosis
To diagnose paradoxes, we compile the program into **SMT-LIB2** constraints (Satisfiability Modulo Theories).
*   **Logic**: `QF_ABV` (Quantifier-Free Arrays and Bit-Vectors).
*   **Encoding**:
    *   $A$: Array variables for Anamnesis.
    *   $P$: Array variables for Present.
    *   Constraint: $P = F_{symbolic}(A) \land A = P$.

If the solver returns `UNSAT`, the **Unsatisfiable Core** identifies the exact memory cells and instructions responsible for the contradiction (e.g., "Cell 0 depends on NOT(Cell 0)").

## 5. Optimization Strategies

### 5.1 Temporal Core Analysis
Not all memory cells participate in time travel. We define the **Temporal Core** ($TC$) as the set of addresses $a$ such that $a$'s value at $t$ depends on $a$'s value at $t+1$.
*   **Optimization**: Restrict fixed-point iteration to $TC$.
*   **Benefit**: Reduces state space from $2^{64 \times M}$ to $2^{64 \times |TC|}$.

### 5.2 Stratification
If the dependency graph of $TC$ is acyclic (except for self-loops), the program can be **Stratified**.
*   Algorithm: Solve fixed points for Stratum 0, then Stratum 1, etc.
*   Benefit: Linear convergence time $O(k \cdot W)$ instead of exponential.

## 6. Implementation Details

### 6.1 The Epoch
The atomic unit of execution is the **Epoch**.
1.  **Load**: $A \leftarrow$ result of previous epoch.
2.  **Execute**: Run program from $PC=0$ to $HALT$.
3.  **Compare**: Check if $P == A$.
4.  **Loop**: If different, update $A \leftarrow P$ and repeat.

### 6.2 Diagnostics Infrastructure
The interpreter maintains a **Causal Graph** tracking dependencies:
*   Nodes: Memory addresses.
*   Edges: "Cell A was used to compute Cell B".
*   Cycle Detection: Tarjan's SCS algorithm on the Causal Graph identifies Negative Loops staticly.
