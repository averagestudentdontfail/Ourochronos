# Ourochronos Coding Standard
**Version**: 1.0
**Applicability**: All Critical Temporal Software

## 1. Introduction
Programming in Ourochronos is not merely instruction sequencing; it is reality engineering. A bug in Ourochronos does not just crash a program; it destabilizes a timeline. This standard enforces rigor to ensure Causal Integrity and Temporal Stability.

## 2. Rule Summary
### Causal Integrity (Safety)
1.  **Bounded Causality**: Every causal loop must have a verifiable convergence condition.
2.  **No Naked Paradoxes**: The `PARADOX` opcode shall only be used within a conditional block that detects an invalid state.
3.  **Grandfather Safety**: Negation loops (reading $X$ and writing $!X$) are prohibited unless gated by a convergence path.

### Temporal Efficiency (Performance)
4.  **Convergence Hygiene**: Programs should converge in $O(1)$ epochs where possible (Identity Loops).
5.  **Divergence Checks**: Loops that increment values based on Anamnesis must include an upper bound check to prevent infinite divergence.

### Structural Clarity (Maintainability)
6.  **Stack Discipline**: Every block (`{ ... }`) must have a net-zero stack effect unless explicitly documented as a transition.
7.  **Prophetic Comments**: Every `PROPHECY` instruction must be preceded by a comment explaining which `ORACLE` read triggers it.

## 3. Detailed Rules

### LOC-1: Causal Integrity

#### Rule 1 (Bounded Causality)
*   **Requirement**: All self-referential logic must admit at least one Fixed Point.
*   **Rationale**: Unbounded loops cause infinite re-simulation, consuming infinite computing resources (and potentially destabilizing the runtime).
*   **Example**:
    ```ourochronos
    0 ORACLE 1 ADD 0 PROPHECY  ; VIOLATION: Diverges (x = x + 1)
    ```

#### Rule 2 (Grandfather Safety)
*   **Requirement**: Do not invert a signal from the future without an escape clause.
*   **Correction**:
    ```ourochronos
    ; VIOLATION: Oscillation
    0 ORACLE NOT 0 PROPHECY

    ; COMPLIANT: Conditional stabilization
    0 ORACLE DUP 10 GT IF {
        DROP 10 0 PROPHECY  ; Stabilize at 10
    } ELSE {
        NOT 0 PROPHECY      ; Oscillate only below 10 (still risky, but bounded)
    }
    ```

### LOC-2: Temporal Efficiency

#### Rule 3 (Identity Loops)
*   **Guideline**: Default to Identity. If you read a value you don't intend to change, write it back exactly as received.
*   **Rationale**: This preserves the stability of memory cells you aren't actively computing on.
*   **Pattern**: `DUP 0 PROPHECY` (Read $X$, Write $X$).

### LOC-3: Structural Clarity

#### Rule 4 (Prophetic Responsibility)
*   **Requirement**: Comments must explicitly link Cause (Oracle) and Effect (Prophecy).
*   **Format**: `; Causal Loop: A[addr] -> P[addr]` prior to `PROPHECY`.
*   **Example**:
    ```ourochronos
    0 ORACLE
    1 ADD
    ; Causal Loop: A[0] -> P[0] (seeking x = x + 1)
    0 PROPHECY
    ```
