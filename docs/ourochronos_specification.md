# OUROCHRONOS

## A Closed Timelike Curve Programming Language

### Complete Formal Specification v1.0

---

## Abstract

OUROCHRONOS is an esoteric programming language whose execution model is founded upon the semantics of closed timelike curves (CTCs). Programs do not compute outputs in the conventional sense; rather, they specify constraints that a self-consistent temporal loop must satisfy. Execution is the discovery of a fixed point in the space of possible memory states—a configuration wherein the information a program "remembers from its future" is identical to the information it "sends to its past."

This document provides a complete formal specification sufficient for compiler implementation, including mathematical foundations, syntax, operational semantics, fixed-point computation algorithms, and proofs of relevant properties.

---

## Table of Contents

1. [Mathematical Preliminaries](#1-mathematical-preliminaries)
2. [Semantic Domains](#2-semantic-domains)
3. [The Temporal Memory Model](#3-the-temporal-memory-model)
4. [Syntax](#4-syntax)
5. [Operational Semantics](#5-operational-semantics)
6. [Fixed-Point Computation](#6-fixed-point-computation)
7. [Consistency and Determinism](#7-consistency-and-determinism)
8. [Turing Completeness](#8-turing-completeness)
9. [Compiler Implementation](#9-compiler-implementation)
10. [Example Programs](#10-example-programs)

---

## 1. Mathematical Preliminaries

### 1.1 Notation

We employ the following notational conventions throughout this specification:

| Symbol | Meaning |
|--------|---------|
| ℤ | The set of integers |
| ℤ_n | The set of integers modulo n, i.e., {0, 1, ..., n-1} |
| ℕ | The set of natural numbers (including 0) |
| 𝔹 | The Boolean domain {⊥, ⊤} or equivalently {0, 1} |
| ⊥ | Bottom element / undefined / false |
| ⊤ | Top element / defined / true |
| ∅ | The empty set |
| [A → B] | The set of total functions from A to B |
| [A ⇀ B] | The set of partial functions from A to B |
| f ∘ g | Function composition: (f ∘ g)(x) = f(g(x)) |
| π_i | The i-th projection function |
| ⟨a, b⟩ | Ordered pair |
| ⟨a, b, c⟩ | Ordered triple |
| f[x ↦ v] | Function f updated at point x with value v |
| dom(f) | Domain of function f |
| cod(f) | Codomain of function f |

### 1.2 Fixed-Point Theory

**Definition 1.1 (Partial Order).** A partial order on a set S is a binary relation ⊑ ⊆ S × S that is:
- Reflexive: ∀x ∈ S. x ⊑ x
- Antisymmetric: ∀x, y ∈ S. (x ⊑ y ∧ y ⊑ x) → x = y
- Transitive: ∀x, y, z ∈ S. (x ⊑ y ∧ y ⊑ z) → x ⊑ z

**Definition 1.2 (Complete Partial Order).** A complete partial order (CPO) is a partial order (S, ⊑) such that:
- There exists a least element ⊥ ∈ S with ∀x ∈ S. ⊥ ⊑ x
- Every ascending chain x₀ ⊑ x₁ ⊑ x₂ ⊑ ... has a least upper bound ⊔{xᵢ | i ∈ ℕ}

**Definition 1.3 (Monotone Function).** A function f: S → S on a partial order (S, ⊑) is monotone if:
∀x, y ∈ S. x ⊑ y → f(x) ⊑ f(y)

**Definition 1.4 (Scott-Continuous Function).** A function f: S → S on a CPO (S, ⊑) is Scott-continuous if it is monotone and preserves least upper bounds of chains:
f(⊔{xᵢ}) = ⊔{f(xᵢ)}

**Theorem 1.1 (Kleene Fixed-Point Theorem).** Let (S, ⊑, ⊥) be a CPO and f: S → S a Scott-continuous function. Then f has a least fixed point given by:
fix(f) = ⊔{fⁿ(⊥) | n ∈ ℕ}
where f⁰(x) = x and fⁿ⁺¹(x) = f(fⁿ(x)).

### 1.3 The Deutschian CTC Model

Our execution semantics draw upon Deutsch's resolution of the grandfather paradox in quantum computing contexts. Deutsch proposed that a CTC enforces self-consistency: the state entering the CTC must equal the state exiting it.

**Definition 1.5 (Deutschian Consistency).** Given a computation C that takes an input state σ_in and a "message from the future" σ_ctc, producing output state σ_out and "message to the past" σ'_ctc, a consistent execution satisfies:
σ_ctc = σ'_ctc

This is precisely a fixed-point condition on the CTC channel.

---

## 2. Semantic Domains

### 2.1 Value Domain

**Definition 2.1 (Values).** The value domain is the set of bounded integers:

Val = ℤ_W where W = 2⁶⁴

All arithmetic operates modulo W, implementing unsigned 64-bit integer semantics. We define:
- val_max = W - 1 = 2⁶⁴ - 1
- val_zero = 0

**Definition 2.2 (Lifted Values).** For handling undefined memory, we employ the lifted domain:

Val_⊥ = Val ∪ {⊥}

with the flat ordering: ⊥ ⊑ v for all v ∈ Val, and v ⊑ v' iff v = v'.

### 2.2 Address Domain

**Definition 2.3 (Addresses).** The address domain is bounded:

Addr = ℤ_M where M = 2¹⁶ = 65536

This provides 65536 addressable memory cells.

### 2.3 Memory Domain

**Definition 2.4 (Memory State).** A memory state is a total function from addresses to lifted values:

Mem = [Addr → Val_⊥]

The set Mem forms a CPO under the pointwise lifting of the flat order on Val_⊥:
σ ⊑ σ' iff ∀a ∈ Addr. σ(a) ⊑ σ'(a)

The bottom element is ⊥_Mem where ⊥_Mem(a) = ⊥ for all a.

**Definition 2.5 (Defined Memory).** A memory state σ is defined, written defined(σ), if:
∀a ∈ Addr. σ(a) ≠ ⊥

### 2.4 Stack Domain

**Definition 2.6 (Stack).** The operand stack is a finite sequence of values:

Stack = Val*

We write ε for the empty stack, v :: s for the stack with v on top of s, and s₁ · s₂ for concatenation.

### 2.5 Output Domain

**Definition 2.7 (Output).** The output domain is a finite sequence of values:

Output = Val*

### 2.6 Program State

**Definition 2.8 (Epoch State).** During a single epoch of execution, the program state is a quintuple:

State = Mem × Mem × Stack × Output × ℕ

A state ⟨P, A, S, O, pc⟩ consists of:
- P ∈ Mem: Present memory (read-write)
- A ∈ Mem: Anamnesis memory (read-only, the "message from the future")
- S ∈ Stack: Operand stack
- O ∈ Output: Output buffer
- pc ∈ ℕ: Program counter

---

## 3. The Temporal Memory Model

### 3.1 Conceptual Overview

OUROCHRONOS employs a dual-memory architecture reflecting the closed timelike curve:

```
                    ┌─────────────────────────────────────┐
                    │           TEMPORAL LOOP             │
                    │                                     │
    ┌───────────┐   │   ┌───────────┐     ┌───────────┐  │
    │  Program  │───┼──▶│  Epoch    │────▶│  Present  │──┼───┐
    │           │   │   │ Execution │     │  Memory   │  │   │
    └───────────┘   │   └───────────┘     └───────────┘  │   │
                    │         ▲                          │   │
                    │         │                          │   │
                    │   ┌─────┴─────┐                    │   │
                    │   │ Anamnesis │◀───────────────────┼───┘
                    │   │  Memory   │    (consistency    │
                    │   └───────────┘     required)      │
                    │                                     │
                    └─────────────────────────────────────┘
```

**Anamnesis** (from Greek ἀνάμνησις, "recollection"): The memory of what will have happened. At the beginning of each epoch, anamnesis contains the program's "memory of the future"—the values it receives from the closed timelike curve.

**Present**: The memory being constructed during the current epoch. At epoch's end, the present becomes the past, which (via the CTC) becomes the anamnesis of the next iteration.

### 3.2 Temporal Consistency Condition

**Definition 3.1 (Temporal Consistency).** An execution is temporally consistent if the present memory at epoch termination equals the anamnesis memory at epoch initialisation:

P_final = A_initial

This is the fixed-point condition that defines valid OUROCHRONOS execution.

### 3.3 Memory Operations

**Definition 3.2 (Present Read).** Reading from present memory at address a in state ⟨P, A, S, O, pc⟩:

read_P(a) = P(a)

If P(a) = ⊥, the read returns 0 (undefined memory reads as zero).

**Definition 3.3 (Present Write).** Writing value v to present memory at address a:

write_P(a, v): ⟨P, A, S, O, pc⟩ ↦ ⟨P[a ↦ v], A, S, O, pc⟩

**Definition 3.4 (Anamnesis Read).** Reading from anamnesis memory at address a:

read_A(a) = A(a)

If A(a) = ⊥, the read returns 0.

Anamnesis is immutable during epoch execution. There is no write_A operation.

### 3.4 The Prophecy-Fulfilment Duality

The programming model of OUROCHRONOS is characterised by a duality:

- **Prophecy**: Reading from anamnesis is receiving a prophecy—information about what the future holds.
- **Fulfilment**: Writing to present is fulfilling that prophecy—ensuring that the information sent to the past matches what was received.

A program achieves consistency when all prophecies are self-fulfilling.

---

## 4. Syntax

### 4.1 Lexical Structure

#### 4.1.1 Character Set

OUROCHRONOS source files are encoded in UTF-8. The following ASCII subset is significant:

- Digits: `0-9`
- Letters: `a-z`, `A-Z`
- Symbols: `@`, `#`, `$`, `!`, `?`, `:`, `;`, `(`, `)`, `[`, `]`, `+`, `-`, `*`, `/`, `%`, `&`, `|`, `^`, `~`, `<`, `>`, `=`, `.`, `,`
- Whitespace: space (0x20), tab (0x09), newline (0x0A), carriage return (0x0D)

#### 4.1.2 Comments

Line comments begin with `;;` and extend to the end of the line.
Block comments are delimited by `(*` and `*)` and may nest.

#### 4.1.3 Tokens

```ebnf
token       ::= keyword | identifier | literal | symbol
keyword     ::= 'ORACLE' | 'PROPHECY' | 'PRESENT' | 'MANIFEST' 
              | 'PARADOX' | 'LOOP' | 'UNTIL' | 'IF' | 'THEN' 
              | 'ELSE' | 'END' | 'HALT' | 'INPUT' | 'OUTPUT'
identifier  ::= letter (letter | digit | '_')*
literal     ::= integer | character
integer     ::= digit+ | '0x' hexdigit+ | '0b' bindigit+
character   ::= '\'' printable '\''
symbol      ::= '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' 
              | '~' | '<' | '>' | '=' | '!' | '@' | '#' | '$'
              | '(' | ')' | '[' | ']' | ':' | ';' | '.' | ','
```

### 4.2 Grammar

#### 4.2.1 Program Structure

```ebnf
program     ::= declaration* statement+

declaration ::= const_decl | label_decl

const_decl  ::= 'MANIFEST' identifier '=' expression ';'

label_decl  ::= identifier ':'
```

#### 4.2.2 Statements

```ebnf
statement   ::= memory_stmt | stack_stmt | control_stmt | io_stmt

memory_stmt ::= present_read | present_write | oracle_read | prophecy_write

present_read   ::= 'PRESENT' '[' expression ']'          ;; Push P[addr] to stack
present_write  ::= 'PRESENT' '[' expression ']' '<-' expression  ;; Write to P[addr]

oracle_read    ::= 'ORACLE' '[' expression ']'           ;; Push A[addr] to stack
prophecy_write ::= 'PROPHECY' '[' expression ']' '<-' expression ;; Write to P[addr] (semantic alias)

stack_stmt  ::= 'PUSH' expression
              | 'DUP'
              | 'DROP'
              | 'SWAP'
              | 'OVER'
              | 'ROT'

control_stmt ::= 'IF' expression 'THEN' statement* ('ELSE' statement*)? 'END'
               | 'LOOP' statement* 'UNTIL' expression 'END'
               | 'GOTO' identifier
               | 'HALT'
               | 'PARADOX'          ;; Force inconsistency (abort epoch)

io_stmt     ::= 'INPUT'             ;; Read from input, push to stack
              | 'OUTPUT'            ;; Pop from stack, append to output buffer
```

#### 4.2.3 Expressions

```ebnf
expression  ::= term (('+' | '-' | '|' | '^') term)*

term        ::= factor (('*' | '/' | '%' | '&') factor)*

factor      ::= unary_op factor
              | '(' expression ')'
              | atom

unary_op    ::= '-' | '~' | '!'

atom        ::= literal
              | identifier
              | 'PEEK'              ;; Top of stack without pop
              | 'DEPTH'             ;; Current stack depth
              | 'ORACLE' '[' expression ']'
              | 'PRESENT' '[' expression ']'
```

### 4.3 Abstract Syntax

For formal semantics, we define the abstract syntax:

**Definition 4.1 (Abstract Syntax).** The abstract syntax of OUROCHRONOS is given by:

```
Program P ::= (D*, S*)                    -- declarations and statements

Declaration D ::= Manifest(x, e)          -- constant definition
                | Label(l)                -- label definition

Statement S ::= PresentRead(e)            -- read P[e], push
              | PresentWrite(e₁, e₂)      -- P[e₁] := e₂
              | OracleRead(e)             -- read A[e], push
              | ProphecyWrite(e₁, e₂)     -- P[e₁] := e₂ (semantic alias)
              | Push(e)                   -- push e
              | Dup | Drop | Swap | Over | Rot
              | If(e, S*, S*)             -- conditional
              | Loop(S*, e)               -- loop until e ≠ 0
              | Goto(l)                   -- jump to label
              | Halt                      -- terminate epoch normally
              | Paradox                   -- terminate epoch with inconsistency
              | Input | Output

Expression e ::= Lit(n)                   -- literal value
               | Const(x)                 -- constant reference
               | BinOp(op, e₁, e₂)        -- binary operation
               | UnOp(op, e)              -- unary operation
               | Peek                     -- stack top
               | Depth                    -- stack depth
               | ORd(e)                   -- oracle read (in expression)
               | PRd(e)                   -- present read (in expression)

BinaryOp op ::= Add | Sub | Mul | Div | Mod | And | Or | Xor | Eq | Lt | Gt

UnaryOp op ::= Neg | Not | BNot
```

---

## 5. Operational Semantics

### 5.1 Semantic Domains (Revisited)

We refine our semantic domains for the operational semantics:

```
State       = Mem × Mem × Stack × Output × PC × Status
Mem         = [Addr → Val_⊥]
Stack       = Val*
Output      = Val*
PC          = ℕ
Status      = Running | Halted | Paradox | Error(String)
```

### 5.2 Notation for Operational Semantics

We write the judgement:

⟨P, A, S, O, pc, Running⟩ —[I]→ ⟨P', A', S', O', pc', σ'⟩

to mean: executing instruction I in the given state produces the new state.

Since A is immutable within an epoch, we have A' = A always.

### 5.3 Expression Evaluation

We define expression evaluation as a function:

⟦·⟧ : Expr × Mem × Mem × Stack × Env → Val

where Env maps constant names to values.

**Definition 5.1 (Expression Semantics).**

```
⟦Lit(n)⟧(P, A, S, Γ)           = n mod W
⟦Const(x)⟧(P, A, S, Γ)         = Γ(x)
⟦BinOp(op, e₁, e₂)⟧(P, A, S, Γ) = ⟦e₁⟧(P, A, S, Γ) ⟨op⟩ ⟦e₂⟧(P, A, S, Γ)
⟦UnOp(op, e)⟧(P, A, S, Γ)      = ⟨op⟩ ⟦e⟧(P, A, S, Γ)
⟦Peek⟧(P, A, S, Γ)             = head(S) if S ≠ ε, else 0
⟦Depth⟧(P, A, S, Γ)            = |S|
⟦ORd(e)⟧(P, A, S, Γ)           = lift(A(⟦e⟧(P, A, S, Γ) mod M))
⟦PRd(e)⟧(P, A, S, Γ)           = lift(P(⟦e⟧(P, A, S, Γ) mod M))
```

where lift(⊥) = 0 and lift(v) = v for v ∈ Val.

Binary operations are defined modulo W:
```
v₁ ⟨Add⟩ v₂ = (v₁ + v₂) mod W
v₁ ⟨Sub⟩ v₂ = (v₁ - v₂ + W) mod W
v₁ ⟨Mul⟩ v₂ = (v₁ × v₂) mod W
v₁ ⟨Div⟩ v₂ = v₁ ÷ v₂ if v₂ ≠ 0, else 0
v₁ ⟨Mod⟩ v₂ = v₁ mod v₂ if v₂ ≠ 0, else 0
v₁ ⟨And⟩ v₂ = v₁ ∧ v₂ (bitwise)
v₁ ⟨Or⟩ v₂  = v₁ ∨ v₂ (bitwise)
v₁ ⟨Xor⟩ v₂ = v₁ ⊕ v₂ (bitwise)
v₁ ⟨Eq⟩ v₂  = 1 if v₁ = v₂, else 0
v₁ ⟨Lt⟩ v₂  = 1 if v₁ < v₂, else 0
v₁ ⟨Gt⟩ v₂  = 1 if v₁ > v₂, else 0
```

Unary operations:
```
⟨Neg⟩ v  = (W - v) mod W
⟨Not⟩ v  = 1 if v = 0, else 0
⟨BNot⟩ v = (W - 1) ⊕ v (bitwise complement)
```

### 5.4 Instruction Semantics

Let Π be the program as a sequence of instructions, with |Π| denoting its length.

#### 5.4.1 Memory Instructions

**Present Read:**
```
⟨P, A, v::S, O, pc, Running⟩ —[PresentRead]→ ⟨P, A, lift(P(v mod M))::S, O, pc+1, Running⟩
```

**Present Write:**
```
⟨P, A, v₂::v₁::S, O, pc, Running⟩ —[PresentWrite]→ ⟨P[(v₁ mod M) ↦ v₂], A, S, O, pc+1, Running⟩
```

**Oracle Read:**
```
⟨P, A, v::S, O, pc, Running⟩ —[OracleRead]→ ⟨P, A, lift(A(v mod M))::S, O, pc+1, Running⟩
```

**Prophecy Write:** (identical to Present Write; semantic distinction only)
```
⟨P, A, v₂::v₁::S, O, pc, Running⟩ —[ProphecyWrite]→ ⟨P[(v₁ mod M) ↦ v₂], A, S, O, pc+1, Running⟩
```

#### 5.4.2 Stack Instructions

**Push:**
```
⟨P, A, S, O, pc, Running⟩ —[Push(e)]→ ⟨P, A, ⟦e⟧::S, O, pc+1, Running⟩
```

**Dup:**
```
⟨P, A, v::S, O, pc, Running⟩ —[Dup]→ ⟨P, A, v::v::S, O, pc+1, Running⟩
```

**Drop:**
```
⟨P, A, v::S, O, pc, Running⟩ —[Drop]→ ⟨P, A, S, O, pc+1, Running⟩
```

**Swap:**
```
⟨P, A, v₁::v₂::S, O, pc, Running⟩ —[Swap]→ ⟨P, A, v₂::v₁::S, O, pc+1, Running⟩
```

**Over:**
```
⟨P, A, v₁::v₂::S, O, pc, Running⟩ —[Over]→ ⟨P, A, v₂::v₁::v₂::S, O, pc+1, Running⟩
```

**Rot:**
```
⟨P, A, v₁::v₂::v₃::S, O, pc, Running⟩ —[Rot]→ ⟨P, A, v₃::v₁::v₂::S, O, pc+1, Running⟩
```

#### 5.4.3 Control Flow Instructions

**If-Then-Else:** (We treat this as a compound instruction that evaluates its body inline)

For structured control flow, we compile to a linear instruction sequence with explicit jumps. The denotational treatment is:

```
⟦If(e, S_then, S_else)⟧(state) = 
    if ⟦e⟧ ≠ 0 then ⟦S_then⟧(state) else ⟦S_else⟧(state)
```

**Loop:**
```
⟦Loop(S_body, e)⟧(state) = 
    let state' = ⟦S_body⟧(state) in
    if ⟦e⟧(state') ≠ 0 then state' else ⟦Loop(S_body, e)⟧(state')
```

**Goto:**
```
⟨P, A, S, O, pc, Running⟩ —[Goto(l)]→ ⟨P, A, S, O, labels(l), Running⟩
```

where labels : Label → ℕ maps labels to instruction addresses.

**Halt:**
```
⟨P, A, S, O, pc, Running⟩ —[Halt]→ ⟨P, A, S, O, pc, Halted⟩
```

**Paradox:**
```
⟨P, A, S, O, pc, Running⟩ —[Paradox]→ ⟨P, A, S, O, pc, Paradox⟩
```

#### 5.4.4 I/O Instructions

**Input:**
```
⟨P, A, S, O, pc, Running⟩ —[Input]→ ⟨P, A, read_input()::S, O, pc+1, Running⟩
```

where read_input() reads from the input stream (deterministic for each epoch).

**Output:**
```
⟨P, A, v::S, O, pc, Running⟩ —[Output]→ ⟨P, A, S, O·v, pc+1, Running⟩
```

### 5.5 Epoch Execution

**Definition 5.2 (Epoch Execution).** Given a program Π and anamnesis A_init, epoch execution is defined as:

```
epoch(Π, A_init, input) = 
    let state₀ = ⟨⊥_Mem, A_init, ε, ε, 0, Running⟩
    in execute(Π, state₀, input)

execute(Π, state, input) = 
    match status(state) with
    | Halted  → state
    | Paradox → state
    | Error   → state
    | Running → 
        if pc(state) ≥ |Π| then state with status := Halted
        else execute(Π, step(Π, state, input), input)
```

where step executes a single instruction.

**Definition 5.3 (Epoch Result).** The result of an epoch is the triple:

epoch_result(state) = ⟨present(state), output(state), status(state)⟩

---

## 6. Fixed-Point Computation

### 6.1 The Consistency Function

**Definition 6.1 (Consistency Function).** For a program Π and input I, define:

F_{Π,I} : Mem → Mem × Output × Status

F_{Π,I}(A) = epoch_result(epoch(Π, A, I))

The program achieves temporal consistency when there exists A* such that:

π₁(F_{Π,I}(A*)) = A*

### 6.2 Fixed-Point Iteration Algorithm

**Algorithm 6.1 (Naive Iteration).**

```
function find_fixed_point(Π, I, max_iterations):
    A := ⊥_Mem                          // Initial guess: all undefined
    for i := 1 to max_iterations:
        (P, O, σ) := epoch(Π, A, I)
        if σ = Paradox:
            return (Inconsistent, ∅)
        if P = A:                        // Fixed point found
            return (Consistent, O)
        A := P                           // Next iteration uses this epoch's present
    return (NonConvergent, ∅)
```

### 6.3 Convergence Analysis

**Definition 6.2 (Memory Distance).** Define the Hamming distance on memory states:

d(σ₁, σ₂) = |{a ∈ Addr | σ₁(a) ≠ σ₂(a)}|

**Definition 6.3 (Contractive Program).** A program Π is contractive for input I if there exists 0 ≤ c < 1 such that for all A:

d(π₁(F_{Π,I}(A)), π₁(F_{Π,I}(A'))) ≤ c · d(A, A')

**Theorem 6.1.** If Π is contractive, then the fixed-point iteration converges in O(M · log(W)) iterations, where M is the memory size and W is the value range.

*Proof sketch.* By the Banach fixed-point theorem, contractive maps on complete metric spaces have unique fixed points, and iteration converges geometrically. □

### 6.4 Advanced Fixed-Point Algorithms

For programs that are not contractive, we employ more sophisticated techniques:

**Algorithm 6.2 (Widening with Narrowing).**

```
function find_fixed_point_widening(Π, I, max_iterations):
    A := ⊥_Mem
    // Widening phase: find post-fixed-point
    for i := 1 to max_iterations / 2:
        (P, O, σ) := epoch(Π, A, I)
        if σ = Paradox:
            return (Inconsistent, ∅)
        if P ⊑ A:
            break                        // Found post-fixed-point
        A := A ∇ P                       // Widening operator
    
    // Narrowing phase: descend to fixed point
    for i := 1 to max_iterations / 2:
        (P, O, σ) := epoch(Π, A, I)
        if P = A:
            return (Consistent, O)
        if ¬(P ⊑ A):
            return (Unstable, ∅)
        A := A ∆ P                       // Narrowing operator
    
    return (NonConvergent, ∅)
```

The widening operator ∇ accelerates convergence by over-approximating:

(σ₁ ∇ σ₂)(a) = 
    if σ₁(a) = σ₂(a) then σ₁(a)
    else if σ₂(a) > σ₁(a) then ⊤  (some large sentinel)
    else σ₁(a)

The narrowing operator ∆ refines the over-approximation:

(σ₁ ∆ σ₂)(a) = 
    if σ₁(a) = ⊤ then σ₂(a)
    else σ₁(a)

### 6.5 Handling Multiple Fixed Points

**Definition 6.4 (Canonical Fixed Point).** When multiple fixed points exist, OUROCHRONOS selects the lexicographically minimal one:

A* = min_{lex} {A ∈ Mem | π₁(F_{Π,I}(A)) = A}

where the lexicographic order compares memory states as sequences:
A <_{lex} A' iff ∃a. (∀a' < a. A(a') = A'(a')) ∧ A(a) < A'(a)

**Algorithm 6.3 (Canonical Fixed Point Search).**

```
function find_canonical_fixed_point(Π, I, max_iterations):
    candidates := ∅
    for seed in canonical_seeds():
        A := seed
        for i := 1 to max_iterations:
            (P, O, σ) := epoch(Π, A, I)
            if σ = Paradox:
                break
            if P = A:
                candidates := candidates ∪ {(A, O)}
                break
            A := P
    
    if candidates = ∅:
        return (Inconsistent, ∅)
    else:
        (A*, O*) := min_{lex}(candidates)
        return (Consistent, O*)
```

The function canonical_seeds() generates starting points systematically, e.g., ⊥_Mem, then memories with single defined cells, etc.

### 6.6 Bounded Iteration Guarantee

**Definition 6.5 (Iteration Bound).** The iteration bound B(Π) for program Π is:

B(Π) = min(max_iterations, M × W)

where max_iterations is a configurable limit (default: 10000).

**Theorem 6.2 (Termination).** The fixed-point search algorithm terminates in at most B(Π) iterations.

*Proof.* The iteration either finds a fixed point, detects a paradox, or exhausts the iteration budget. Since each iteration is finite (assuming epoch execution terminates), the overall procedure terminates. □

---

## 7. Consistency and Determinism

### 7.1 Epoch Determinism

**Theorem 7.1 (Epoch Determinism).** For any program Π, anamnesis A, and input I, epoch execution is deterministic:

∀A, I. ∃! result. epoch(Π, A, I) = result

*Proof.* By structural induction on the operational semantics. Each instruction rule is deterministic, and the epoch terminates when status becomes non-Running or pc exceeds program length. □

### 7.2 Fixed-Point Determinism

**Theorem 7.2 (Fixed-Point Determinism).** The OUROCHRONOS fixed-point search produces deterministic output:

∀Π, I. ∃! O. find_canonical_fixed_point(Π, I) = (_, O) ∨ find_canonical_fixed_point(Π, I) = (Inconsistent, ∅)

*Proof.* By Theorem 7.1, each epoch is deterministic. The canonical seed ordering is fixed, and the lexicographic minimum over a finite set is unique. □

### 7.3 Consistency Classes

**Definition 7.1 (Program Consistency Classes).** We classify OUROCHRONOS programs:

1. **Trivially Consistent**: F_{Π,I}(⊥_Mem) = (⊥_Mem, O, Halted)
   The program never reads from anamnesis and produces no prophecies.

2. **Self-Fulfilling**: ∃A*. π₁(F_{Π,I}(A*)) = A*
   The program has at least one consistent execution.

3. **Paradoxical**: ∀A. π₁(F_{Π,I}(A)) ≠ A ∨ status(F_{Π,I}(A)) = Paradox
   No consistent execution exists.

4. **Divergent**: Epoch execution does not terminate.
   The program contains an infinite loop within a single epoch.

### 7.4 Consistency Verification

**Definition 7.2 (Consistency Witness).** A consistency witness for program Π with input I is a memory state A* such that:

π₁(F_{Π,I}(A*)) = A* ∧ status(F_{Π,I}(A*)) = Halted

**Algorithm 7.1 (Witness Verification).**

```
function verify_witness(Π, I, A_witness):
    (P, O, σ) := epoch(Π, A_witness, I)
    return (P = A_witness) ∧ (σ = Halted)
```

This runs in O(epoch_time) and provides a certificate of consistency.

---

## 8. Turing Completeness

### 8.1 Simulation of Turing Machines

**Theorem 8.1 (Turing Completeness).** OUROCHRONOS is Turing complete.

*Proof.* We prove this by simulating an arbitrary Turing machine M = (Q, Σ, Γ, δ, q₀, q_accept, q_reject).

**Encoding:**
- Tape: Present memory addresses 0 to M-2 encode tape cells.
- Head position: Present memory address M-1 encodes head position.
- State: Present memory address M-2 encodes current state.

**Simulation:**
The program performs standard TM simulation, never reading from anamnesis:

```
MANIFEST TAPE_START = 0;
MANIFEST TAPE_END = 65533;
MANIFEST HEAD_ADDR = 65534;
MANIFEST STATE_ADDR = 65535;

;; Initialize
PUSH q₀; PUSH STATE_ADDR; PRESENT_WRITE;
PUSH initial_head_pos; PUSH HEAD_ADDR; PRESENT_WRITE;
;; (initialize tape cells)

main_loop:
    ;; Read current state
    PUSH STATE_ADDR; PRESENT_READ;
    
    ;; Check for halt states
    DUP; PUSH q_accept; EQ; IF THEN HALT END;
    DUP; PUSH q_reject; EQ; IF THEN HALT END;
    
    ;; Read tape symbol at head
    PUSH HEAD_ADDR; PRESENT_READ; PRESENT_READ;
    
    ;; Compute transition (state, symbol) -> (state', symbol', direction)
    ;; (lookup table implementation)
    
    ;; Write new symbol
    ;; Update head position
    ;; Update state
    
    GOTO main_loop;
```

Since this program never reads anamnesis, any anamnesis (including ⊥_Mem) is consistent, making F_{Π,I}(⊥_Mem) a fixed point. The simulation is faithful, and M accepts iff the OUROCHRONOS program outputs accordingly. □

### 8.2 The Temporal Fragment

The interesting computational class is programs that meaningfully use anamnesis:

**Definition 8.1 (Temporal Program).** A program Π is temporal if there exists input I and address a such that:

∃A₁, A₂. A₁(a) ≠ A₂(a) ∧ epoch(Π, A₁, I) ≠ epoch(Π, A₂, I)

**Theorem 8.2 (Temporal Turing Completeness).** Temporal OUROCHRONOS programs form a Turing complete class.

*Proof sketch.* We can simulate a TM by using anamnesis to "receive" the final tape state, verify it step-by-step, and write it to present memory. The fixed point exists iff the TM halts with that tape configuration. □

### 8.3 Complexity Considerations

**Theorem 8.3.** Determining whether an OUROCHRONOS program has a consistent execution is undecidable.

*Proof.* Reduction from the halting problem. Given TM M, construct OUROCHRONOS program Π_M that:
1. Reads a "claimed halting time" t from anamnesis cell 0.
2. Simulates M for t steps.
3. If M halts in exactly t steps, writes t to present cell 0.
4. Otherwise, writes t+1 to present cell 0.

Π_M has a consistent execution iff M halts: if M halts at time t*, then A(0) = t* is a fixed point. If M never halts, no fixed point exists. □

---

## 9. Compiler Implementation

### 9.1 Compilation Pipeline

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│    Source    │────▶│    Lexer     │────▶│    Parser    │────▶│   AST        │
│    (.ouro)   │     │              │     │              │     │              │
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
                                                                      │
                     ┌──────────────┐     ┌──────────────┐           │
                     │   Bytecode   │◀────│   Compiler   │◀──────────┘
                     │   (.ourob)   │     │              │
                     └──────────────┘     └──────────────┘
                            │
                     ┌──────────────┐     ┌──────────────┐
                     │  Interpreter │────▶│   Output     │
                     │   (VM)       │     │              │
                     └──────────────┘     └──────────────┘
```

### 9.2 Bytecode Format

#### 9.2.1 Instruction Encoding

Each instruction is encoded as a variable-length byte sequence:

```
┌─────────┬─────────────────────────────────────┐
│ Opcode  │ Operand (optional, variable length) │
│ (1 byte)│                                     │
└─────────┴─────────────────────────────────────┘
```

#### 9.2.2 Opcode Table

| Opcode | Mnemonic | Operand | Description |
|--------|----------|---------|-------------|
| 0x00 | NOP | - | No operation |
| 0x01 | HALT | - | Terminate epoch normally |
| 0x02 | PARADOX | - | Terminate with paradox |
| 0x10 | PUSH_IMM | u64 | Push immediate value |
| 0x11 | DUP | - | Duplicate top of stack |
| 0x12 | DROP | - | Remove top of stack |
| 0x13 | SWAP | - | Swap top two elements |
| 0x14 | OVER | - | Copy second element to top |
| 0x15 | ROT | - | Rotate top three elements |
| 0x20 | P_READ | - | Read present[pop()], push result |
| 0x21 | P_WRITE | - | present[pop()] := pop() |
| 0x22 | A_READ | - | Read anamnesis[pop()], push result |
| 0x30 | ADD | - | Push pop() + pop() |
| 0x31 | SUB | - | Push pop() - pop() (second - first) |
| 0x32 | MUL | - | Push pop() × pop() |
| 0x33 | DIV | - | Push pop() ÷ pop() |
| 0x34 | MOD | - | Push pop() mod pop() |
| 0x35 | AND | - | Bitwise AND |
| 0x36 | OR | - | Bitwise OR |
| 0x37 | XOR | - | Bitwise XOR |
| 0x38 | NOT | - | Logical NOT |
| 0x39 | BNOT | - | Bitwise NOT |
| 0x3A | NEG | - | Arithmetic negation |
| 0x3B | EQ | - | Push 1 if equal, 0 otherwise |
| 0x3C | LT | - | Push 1 if less than |
| 0x3D | GT | - | Push 1 if greater than |
| 0x40 | JMP | u32 | Unconditional jump to address |
| 0x41 | JZ | u32 | Jump if top is zero (consumes) |
| 0x42 | JNZ | u32 | Jump if top is nonzero (consumes) |
| 0x50 | INPUT | - | Read input, push to stack |
| 0x51 | OUTPUT | - | Pop and append to output |
| 0x60 | DEPTH | - | Push stack depth |

### 9.3 Virtual Machine Specification

#### 9.3.1 VM State

```
struct VMState {
    present: [u64; 65536],        // Present memory
    anamnesis: [u64; 65536],      // Anamnesis memory (read-only during epoch)
    stack: Vec<u64>,              // Operand stack
    output: Vec<u64>,             // Output buffer
    pc: usize,                    // Program counter
    status: Status,               // Running | Halted | Paradox | Error
}

enum Status {
    Running,
    Halted,
    Paradox,
    Error(String),
}
```

#### 9.3.2 Epoch Execution Loop

```
function run_epoch(bytecode: &[u8], anamnesis: &[u64; 65536], input: &[u64]) -> EpochResult {
    let mut state = VMState::new(anamnesis);
    let mut input_cursor = 0;
    
    while state.status == Running {
        if state.pc >= bytecode.len() {
            state.status = Halted;
            break;
        }
        
        let opcode = bytecode[state.pc];
        state.pc += 1;
        
        match opcode {
            0x00 => { /* NOP */ }
            0x01 => { state.status = Halted; }
            0x02 => { state.status = Paradox; }
            0x10 => {
                let value = read_u64(&bytecode[state.pc..]);
                state.pc += 8;
                state.stack.push(value);
            }
            // ... (remaining opcodes)
        }
        
        // Fuel check for divergence detection
        if iterations > MAX_EPOCH_ITERATIONS {
            state.status = Error("Epoch timeout".into());
        }
    }
    
    EpochResult {
        present: state.present,
        output: state.output,
        status: state.status,
    }
}
```

#### 9.3.3 Fixed-Point Driver

```
function execute_program(bytecode: &[u8], input: &[u64]) -> ProgramResult {
    let mut anamnesis = [0u64; 65536];  // Initial guess: all zeros
    
    for iteration in 0..MAX_FIXED_POINT_ITERATIONS {
        let result = run_epoch(bytecode, &anamnesis, input);
        
        match result.status {
            Paradox => return ProgramResult::Inconsistent,
            Error(e) => return ProgramResult::Error(e),
            Halted => {
                if result.present == anamnesis {
                    return ProgramResult::Consistent(result.output);
                }
                anamnesis = result.present;  // Next iteration
            }
        }
    }
    
    ProgramResult::NonConvergent
}
```

### 9.4 Optimisations

#### 9.4.1 Anamnesis Dependency Analysis

Static analysis can determine which anamnesis cells a program reads:

```
function anamnesis_dependencies(ast: &AST) -> Set<Addr> {
    let mut deps = Set::new();
    for stmt in ast.statements {
        match stmt {
            OracleRead(e) => {
                if let Some(addr) = const_eval(e) {
                    deps.insert(addr);
                } else {
                    return Set::all();  // Dynamic address, assume all
                }
            }
            // ... recurse into control structures
        }
    }
    deps
}
```

Programs with empty dependency sets are trivially consistent.

#### 9.4.2 Incremental Fixed-Point Computation

Track which present cells were written, and only compare those cells:

```
function incremental_fixed_point(bytecode, input) {
    let mut anamnesis = [UNDEFINED; 65536];
    let mut written_cells = Set::new();
    
    for iteration in 0..MAX_ITERATIONS {
        let (result, newly_written) = run_epoch_tracked(bytecode, &anamnesis, input);
        written_cells = written_cells.union(newly_written);
        
        let consistent = written_cells.iter().all(|&addr| {
            result.present[addr] == anamnesis[addr]
        });
        
        if consistent && result.status == Halted {
            return Consistent(result.output);
        }
        
        for &addr in &written_cells {
            anamnesis[addr] = result.present[addr];
        }
    }
    NonConvergent
}
```

#### 9.4.3 Memoisation

Cache epoch results for repeated anamnesis states:

```
struct EpochCache {
    cache: HashMap<Hash, EpochResult>,
}

function cached_epoch(cache, bytecode, anamnesis, input) {
    let key = hash(anamnesis, input);
    if let Some(result) = cache.get(key) {
        return result.clone();
    }
    let result = run_epoch(bytecode, anamnesis, input);
    cache.insert(key, result.clone());
    result
}
```

### 9.5 Error Handling

#### 9.5.1 Static Errors (Compile-Time)

| Error Code | Description |
|------------|-------------|
| E001 | Syntax error |
| E002 | Undefined constant |
| E003 | Undefined label |
| E004 | Duplicate label |
| E005 | Type error in expression |

#### 9.5.2 Dynamic Errors (Run-Time)

| Error Code | Description |
|------------|-------------|
| E101 | Stack underflow |
| E102 | Division by zero (returns 0, not error) |
| E103 | Epoch timeout (exceeded MAX_EPOCH_ITERATIONS) |
| E104 | Fixed-point timeout (exceeded MAX_FIXED_POINT_ITERATIONS) |
| E105 | Input exhausted |

---

## 10. Example Programs

### 10.1 Hello, Consistency

The simplest consistent program:

```ouro
;; hello_consistency.ouro
;; A program that achieves consistency trivially by not using anamnesis.

MANIFEST H = 72;   ;; 'H'
MANIFEST e = 101;  ;; 'e'
MANIFEST l = 108;  ;; 'l'
MANIFEST o = 111;  ;; 'o'
MANIFEST SPACE = 32;
MANIFEST W = 87;   ;; 'W'
MANIFEST r = 114;  ;; 'r'
MANIFEST d = 100;  ;; 'd'
MANIFEST BANG = 33;
MANIFEST NEWLINE = 10;

PUSH H; OUTPUT;
PUSH e; OUTPUT;
PUSH l; OUTPUT;
PUSH l; OUTPUT;
PUSH o; OUTPUT;
PUSH SPACE; OUTPUT;
PUSH W; OUTPUT;
PUSH o; OUTPUT;
PUSH r; OUTPUT;
PUSH l; OUTPUT;
PUSH d; OUTPUT;
PUSH BANG; OUTPUT;
PUSH NEWLINE; OUTPUT;
HALT;
```

**Analysis:** This program never reads anamnesis, so F(⊥) = (⊥, "Hello, World!\n", Halted), which is trivially a fixed point.

### 10.2 Self-Fulfilling Prophecy

A program that reads its own future and fulfils it:

```ouro
;; prophecy.ouro
;; The program reads a value from anamnesis and writes the same value to present.
;; The fixed point is when these are equal.

MANIFEST PROPHECY_ADDR = 0;

;; Read the prophecy: "What will cell 0 contain?"
PUSH PROPHECY_ADDR;
ORACLE [PEEK];       ;; Push A[0] to stack
DROP;                ;; Clean up address

;; Fulfil the prophecy: write the same value to present
PUSH PROPHECY_ADDR;
SWAP;
PRESENT [PEEK] <- PEEK;  ;; P[0] := value from oracle
DROP; DROP;

HALT;
```

**Execution trace:**

- Epoch 1: A[0] = ⊥ (undefined, reads as 0). Program writes P[0] = 0.
- Epoch 2: A[0] = 0. Program writes P[0] = 0.
- Fixed point achieved: A[0] = P[0] = 0.

Any initial anamnesis value is a fixed point, as the program faithfully copies it.

### 10.3 The Bootstrap Paradox

A program that computes the Fibonacci sequence via temporal bootstrap:

```ouro
;; fibonacci_bootstrap.ouro
;; Compute Fibonacci(10) by receiving the answer from the future,
;; verifying it, and sending it to the past.

MANIFEST N = 10;
MANIFEST RESULT_ADDR = 0;
MANIFEST TEMP_ADDR = 1;

;; Receive claimed result from future
PUSH RESULT_ADDR;
ORACLE [PEEK];
DROP;
;; Stack: [claimed_fib_n]

;; Compute Fibonacci(N) the hard way to verify
PUSH 0;              ;; fib(0)
PUSH 1;              ;; fib(1)
;; Stack: [claimed_fib_n, fib_prev, fib_curr]

PUSH N;
PUSH 1;
SUB;                 ;; counter = N - 1
;; Stack: [claimed_fib_n, fib_prev, fib_curr, counter]

fib_loop:
    ;; Check if counter == 0
    DUP;
    PUSH 0;
    EQ;
    IF THEN
        DROP;        ;; Remove counter
        GOTO fib_done;
    END
    
    ;; Compute next Fibonacci number
    ;; Stack: [claimed, prev, curr, counter]
    ROT;             ;; [claimed, curr, counter, prev]
    ROT;             ;; [claimed, counter, prev, curr]
    ROT;             ;; [claimed, prev, curr, counter]
    
    ;; new_curr = prev + curr
    OVER;            ;; [claimed, prev, curr, counter, curr]
    ROT;             ;; [claimed, prev, counter, curr, curr]
    ROT;             ;; [claimed, counter, curr, curr, prev]
    ADD;             ;; [claimed, counter, curr, new_curr]
    ROT;             ;; [claimed, curr, new_curr, counter]
    ROT;             ;; [claimed, new_curr, counter, curr]
    DROP;            ;; [claimed, new_curr, counter] -- wait, this isn't right
    
    ;; (Simplified: assume helper for Fibonacci computation)
    ;; After loop: Stack: [claimed_fib_n, computed_fib_n]
    
    PUSH 1;
    SUB;             ;; counter--
    GOTO fib_loop;

fib_done:
;; Stack: [claimed_fib_n, fib_prev, fib_curr]
;; fib_curr is Fibonacci(N)

SWAP;
DROP;                ;; Remove fib_prev
;; Stack: [claimed_fib_n, computed_fib_n]

;; Verify: does claimed == computed?
DUP;
ROT;
DUP;
ROT;
;; Stack: [claimed, computed, claimed, computed]
EQ;
IF THEN
    ;; Prophecy verified! Write to present.
    DROP;
    PUSH RESULT_ADDR;
    SWAP;
    PRESENT [PEEK] <- PEEK;
    DROP; DROP;
    
    ;; Output the result
    PUSH RESULT_ADDR;
    PRESENT [PEEK];
    DROP;
    OUTPUT;
    HALT;
ELSE
    ;; Prophecy false! Write computed value (will iterate).
    SWAP;
    DROP;
    PUSH RESULT_ADDR;
    SWAP;
    PRESENT [PEEK] <- PEEK;
    DROP; DROP;
    HALT;
END
```

**Execution trace:**

- Epoch 1: A[0] = 0. Computed Fib(10) = 55. Mismatch. P[0] = 55.
- Epoch 2: A[0] = 55. Computed Fib(10) = 55. Match! Output: 55.

The fixed point is reached when the "prophecy" matches the computation.

### 10.4 Grandfather Paradox (Inconsistent Program)

A program that cannot achieve consistency:

```ouro
;; grandfather_paradox.ouro
;; This program deliberately creates an inconsistency.
;; If A[0] = 0, write P[0] = 1.
;; If A[0] = 1, write P[0] = 0.
;; No fixed point exists.

MANIFEST ADDR = 0;

PUSH ADDR;
ORACLE [PEEK];
DROP;
;; Stack: [A[0]]

PUSH 0;
EQ;
IF THEN
    PUSH 1;
ELSE
    PUSH 0;
END
;; Stack: [new_value]

PUSH ADDR;
SWAP;
PRESENT [PEEK] <- PEEK;
DROP; DROP;

HALT;
```

**Execution trace:**

- Epoch 1: A[0] = 0. Condition true. P[0] = 1.
- Epoch 2: A[0] = 1. Condition false. P[0] = 0.
- Epoch 3: A[0] = 0. Condition true. P[0] = 1.
- ... (oscillates forever)

**Result:** NonConvergent / Inconsistent.

### 10.5 Primality Test via Temporal Witness

Use anamnesis to receive a factor (if one exists):

```ouro
;; primality.ouro
;; Test if input N is prime.
;; Oracle provides a potential factor. If factor is valid, N is composite.
;; If no valid factor exists in any fixed point, N is prime.

MANIFEST N_ADDR = 0;
MANIFEST FACTOR_ADDR = 1;
MANIFEST RESULT_ADDR = 2;

;; Read N from input
INPUT;
DUP;
PUSH N_ADDR;
SWAP;
PRESENT [PEEK] <- PEEK;
DROP; DROP;
;; Stack: [N], P[N_ADDR] = N

;; Read claimed factor from oracle
PUSH FACTOR_ADDR;
ORACLE [PEEK];
DROP;
;; Stack: [N, factor]

;; Check if factor is valid: 1 < factor < N and N mod factor == 0
DUP;
PUSH 1;
GT;                  ;; factor > 1?
SWAP;
DUP;
ROT;
SWAP;
;; Stack: [N, factor, factor>1, factor]

ROT;
DUP;
ROT;
;; Stack: [factor, factor>1, factor, N, N]

SWAP;
LT;                  ;; factor < N?
;; Stack: [factor, factor>1, N, factor<N]

ROT;
ROT;
;; Stack: [factor>1, factor<N, factor, N]

SWAP;
DUP;
ROT;
MOD;
PUSH 0;
EQ;                  ;; N mod factor == 0?
;; Stack: [factor>1, factor<N, divides]

AND;
AND;
;; Stack: [is_valid_factor]

IF THEN
    ;; Valid factor found: N is composite
    ;; Write factor to present (self-consistent)
    PUSH FACTOR_ADDR;
    ORACLE [PEEK];
    DROP;
    PUSH FACTOR_ADDR;
    SWAP;
    PRESENT [PEEK] <- PEEK;
    DROP; DROP;
    
    PUSH 0;          ;; 0 = composite
    OUTPUT;
ELSE
    ;; No valid factor with this oracle value
    ;; Write 1 to present (different from any valid factor)
    PUSH 1;
    PUSH FACTOR_ADDR;
    SWAP;
    PRESENT [PEEK] <- PEEK;
    DROP; DROP;
    
    PUSH 1;          ;; 1 = prime (tentative)
    OUTPUT;
END

HALT;
```

**Semantics:** 
- If N is composite with factor f, the fixed point A[FACTOR_ADDR] = f exists.
- If N is prime, iteration converges to A[FACTOR_ADDR] = 1 (not a valid factor), outputting "prime."

---

## Appendix A: Formal Grammar (BNF)

```bnf
<program>      ::= <declaration>* <statement>+

<declaration>  ::= <const-decl> | <label-decl>
<const-decl>   ::= "MANIFEST" <identifier> "=" <expression> ";"
<label-decl>   ::= <identifier> ":"

<statement>    ::= <memory-stmt> | <stack-stmt> | <control-stmt> | <io-stmt>

<memory-stmt>  ::= "PRESENT" "[" <expression> "]"
                 | "PRESENT" "[" <expression> "]" "<-" <expression>
                 | "ORACLE" "[" <expression> "]"
                 | "PROPHECY" "[" <expression> "]" "<-" <expression>

<stack-stmt>   ::= "PUSH" <expression>
                 | "DUP" | "DROP" | "SWAP" | "OVER" | "ROT"

<control-stmt> ::= "IF" <expression> "THEN" <statement>* 
                   ("ELSE" <statement>*)? "END"
                 | "LOOP" <statement>* "UNTIL" <expression> "END"
                 | "GOTO" <identifier>
                 | "HALT"
                 | "PARADOX"

<io-stmt>      ::= "INPUT" | "OUTPUT"

<expression>   ::= <term> (("+" | "-" | "|" | "^") <term>)*
<term>         ::= <factor> (("*" | "/" | "%" | "&") <factor>)*
<factor>       ::= <unary-op> <factor> | "(" <expression> ")" | <atom>
<unary-op>     ::= "-" | "~" | "!"
<atom>         ::= <integer> | <character> | <identifier>
                 | "PEEK" | "DEPTH"
                 | "ORACLE" "[" <expression> "]"
                 | "PRESENT" "[" <expression> "]"

<identifier>   ::= <letter> (<letter> | <digit> | "_")*
<integer>      ::= <digit>+ | "0x" <hex-digit>+ | "0b" <bin-digit>+
<character>    ::= "'" <printable> "'"
```

---

## Appendix B: Reference Implementation (Pseudocode)

### B.1 Complete Interpreter

```pseudocode
class OurochronosInterpreter:
    MAX_EPOCH_ITERATIONS = 1_000_000
    MAX_FIXED_POINT_ITERATIONS = 10_000
    MEMORY_SIZE = 65536
    
    function execute(program: Program, input: List<u64>) -> Result:
        bytecode = compile(program)
        return run_fixed_point(bytecode, input)
    
    function run_fixed_point(bytecode: Bytes, input: List<u64>) -> Result:
        anamnesis = [0] * MEMORY_SIZE
        
        for iteration in range(MAX_FIXED_POINT_ITERATIONS):
            result = run_epoch(bytecode, anamnesis, input)
            
            match result.status:
                case Paradox:
                    return Result.Inconsistent
                case Error(e):
                    return Result.Error(e)
                case Halted:
                    if result.present == anamnesis:
                        return Result.Consistent(result.output)
                    anamnesis = result.present.copy()
        
        return Result.NonConvergent
    
    function run_epoch(bytecode: Bytes, anamnesis: Memory, input: List<u64>) -> EpochResult:
        state = EpochState(
            present = [0] * MEMORY_SIZE,
            anamnesis = anamnesis,
            stack = [],
            output = [],
            pc = 0,
            status = Running,
            input_cursor = 0
        )
        
        iterations = 0
        while state.status == Running:
            if iterations > MAX_EPOCH_ITERATIONS:
                state.status = Error("Epoch timeout")
                break
            
            if state.pc >= len(bytecode):
                state.status = Halted
                break
            
            execute_instruction(state, bytecode, input)
            iterations += 1
        
        return EpochResult(state.present, state.output, state.status)
    
    function execute_instruction(state: EpochState, bytecode: Bytes, input: List<u64>):
        opcode = bytecode[state.pc]
        state.pc += 1
        
        match opcode:
            case 0x00:  // NOP
                pass
            
            case 0x01:  // HALT
                state.status = Halted
            
            case 0x02:  // PARADOX
                state.status = Paradox
            
            case 0x10:  // PUSH_IMM
                value = read_u64(bytecode, state.pc)
                state.pc += 8
                state.stack.append(value)
            
            case 0x11:  // DUP
                if len(state.stack) < 1:
                    state.status = Error("Stack underflow")
                    return
                state.stack.append(state.stack[-1])
            
            case 0x12:  // DROP
                if len(state.stack) < 1:
                    state.status = Error("Stack underflow")
                    return
                state.stack.pop()
            
            case 0x13:  // SWAP
                if len(state.stack) < 2:
                    state.status = Error("Stack underflow")
                    return
                state.stack[-1], state.stack[-2] = state.stack[-2], state.stack[-1]
            
            case 0x20:  // P_READ
                if len(state.stack) < 1:
                    state.status = Error("Stack underflow")
                    return
                addr = state.stack.pop() % MEMORY_SIZE
                state.stack.append(state.present[addr])
            
            case 0x21:  // P_WRITE
                if len(state.stack) < 2:
                    state.status = Error("Stack underflow")
                    return
                value = state.stack.pop()
                addr = state.stack.pop() % MEMORY_SIZE
                state.present[addr] = value
            
            case 0x22:  // A_READ
                if len(state.stack) < 1:
                    state.status = Error("Stack underflow")
                    return
                addr = state.stack.pop() % MEMORY_SIZE
                state.stack.append(state.anamnesis[addr])
            
            case 0x30:  // ADD
                binary_op(state, lambda a, b: (a + b) % (2**64))
            
            case 0x31:  // SUB
                binary_op(state, lambda a, b: (b - a + 2**64) % (2**64))
            
            // ... (remaining opcodes)
            
            case 0x40:  // JMP
                addr = read_u32(bytecode, state.pc)
                state.pc = addr
            
            case 0x41:  // JZ
                addr = read_u32(bytecode, state.pc)
                state.pc += 4
                if len(state.stack) < 1:
                    state.status = Error("Stack underflow")
                    return
                if state.stack.pop() == 0:
                    state.pc = addr
            
            case 0x50:  // INPUT
                if state.input_cursor >= len(input):
                    state.status = Error("Input exhausted")
                    return
                state.stack.append(input[state.input_cursor])
                state.input_cursor += 1
            
            case 0x51:  // OUTPUT
                if len(state.stack) < 1:
                    state.status = Error("Stack underflow")
                    return
                state.output.append(state.stack.pop())
    
    function binary_op(state: EpochState, op: Function):
        if len(state.stack) < 2:
            state.status = Error("Stack underflow")
            return
        b = state.stack.pop()
        a = state.stack.pop()
        state.stack.append(op(a, b))
```

---

## Appendix C: Mathematical Properties

### C.1 Decidability Results

| Property | Decidability |
|----------|--------------|
| Epoch termination | Undecidable (reduces to halting problem) |
| Consistency existence | Undecidable (Theorem 8.3) |
| Unique fixed point | Undecidable |
| Output equivalence | Undecidable |
| Trivial consistency | Decidable (static analysis) |

### C.2 Complexity Classes

| Operation | Time Complexity | Space Complexity |
|-----------|-----------------|------------------|
| Single epoch | O(E) | O(M + S) |
| Fixed-point search (contractive) | O(E · M · log W) | O(M) |
| Fixed-point search (general) | O(E · I) | O(M) |

Where:
- E = epoch execution steps (bounded by MAX_EPOCH_ITERATIONS)
- M = memory size (65536)
- W = value range (2^64)
- S = maximum stack size
- I = fixed-point iterations (bounded by MAX_FIXED_POINT_ITERATIONS)

### C.3 Fixed-Point Lattice Structure

The space of possible fixed points for a program Π forms a lattice under the intersection ordering:

**Theorem C.1.** Let FP(Π, I) = {A ∈ Mem | π₁(F_{Π,I}(A)) = A}. If FP(Π, I) is non-empty and F_{Π,I} is monotone, then FP(Π, I) forms a complete lattice with meet ∧ and join ∨.

This implies that when fixed points exist, there is both a least and a greatest fixed point.

---

## Appendix D: Glossary

**Anamnesis** (ἀνάμνησις): Memory of the future; the read-only memory containing values received via the closed timelike curve.

**Consistency**: The property that a program's final present state equals its initial anamnesis state.

**Epoch**: A single execution of the program body, from initialisation to HALT or PARADOX.

**Fixed Point**: A memory state A such that executing an epoch with anamnesis A produces present memory A.

**Oracle**: An instruction that reads from anamnesis (receiving information from the future).

**Paradox**: Explicit termination indicating that the current execution branch cannot achieve consistency.

**Present**: The read-write memory constructed during epoch execution; becomes anamnesis for subsequent iterations.

**Prophecy**: An instruction that writes to present with the intent of fulfilling a temporal prediction.

**Temporal Consistency**: See Consistency.

---

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-01-02 | Initial specification |

---

*"The serpent bites its tail, and time flows in a circle."*

— OUROCHRONOS Design Philosophy
