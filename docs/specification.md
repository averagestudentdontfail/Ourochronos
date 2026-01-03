# Ourochronos Formal Specification

## 1. Abstract Machine

The Ourochronos Abstract Machine (OAM) consists of the following state components:

*   **Stack ($S$)**: A LIFO structure of 64-bit unsigned integers ($u64$).
*   **Anamnesis ($A$)**: A read-only memory of $2^{16}$ cells ($u16 \to u64$), representing the state of the world at the *start* of the timeline (received from the future).
*   **Present ($P$)**: A mutable memory of $2^{16}$ cells, representing the state of the world being constructed in the current timeline. $P$ is initialized to $0$ at the start of execution.
*   **Program Counter ($PC$)**: Index of the current instruction.

### 1.1 Execution Model
The machine operates in **Epochs**.
1.  **Initialization**: $S$ is empty. $P$ is zeroed. $A$ is populated with the result of the *previous* epoch (or Seed/Zero for the first epoch).
2.  **Execution**: The program runs sequentially.
    *   `ORACLE` reads from $A$.
    *   `PROPHECY` writes to $P$.
3.  **Termination**: Execution ends when $PC$ reaches the end of the program or a `HALT` instruction is executed.
4.  **Convergence Check**:
    *   If $P = A$, the timeline is **Consistent**. Execution halts successfully.
    *   If $P \neq A$, a **Time Loop** occurs. $A_{next} \leftarrow P$, and a new Epoch begins.

## 2. Instruction Set Architecture (ISA)

### 2.1 Stack Manipulation
| Opcode | Stack Effect | Description |
| :--- | :--- | :--- |
| `Bi` | `( -- n )` | Push integer literal $n$. |
| `POP` | `( a -- )` | Discard top element. |
| `DUP` | `( a -- a a )` | Duplicate top element. |
| `SWAP` | `( a b -- b a )` | Swap top two elements. |
| `OVER` | `( a b -- a b a )` | Copy second element to top. |
| `ROT` | `( a b c -- b c a )` | Rotate top three elements. |
| `DEPTH` | `( -- n )` | Push current stack depth. |

### 2.2 Arithmetic & Logic
All operations are modulo $2^{64}$.
| Opcode | Stack Effect | Description |
| :--- | :--- | :--- |
| `ADD` | `( a b -- a+b )` | |
| `SUB` | `( a b -- a-b )` | |
| `MUL` | `( a b -- a*b )` | |
| `DIV` | `( a b -- a/b )` | Integer division. If $b=0$, result is $0$. |
| `MOD` | `( a b -- a%b )` | Modulo. If $b=0$, result is $0$. |
| `NOT` | `( a -- ~a )` | Bitwise NOT. |
| `AND` | `( a b -- a&b )` | Bitwise AND. |
| `OR` | `( a b -- a\|b )` | Bitwise OR. |
| `XOR` | `( a b -- a^b )` | Bitwise XOR. |

### 2.3 Comparison
Returns $1$ for true, $0$ for false.
| Opcode | Stack Effect | Description |
| :--- | :--- | :--- |
| `EQ` | `( a b -- ? )` | $a = b$ |
| `NEQ` | `( a b -- ? )` | $a \neq b$ |
| `LT` | `( a b -- ? )` | $a < b$ |
| `GT` | `( a b -- ? )` | $a > b$ |
| `LTE` | `( a b -- ? )` | $a \le b$ |
| `GTE` | `( a b -- ? )` | $a \ge b$ |

### 2.4 Control Flow
Ourochronos uses structured control flow.
*   `IF { ... } ELSE { ... }`: Pops condition. If $\neq 0$, execute `then` block, else execute `else` block.
*   `WHILE { cond } { body }`: Executes `cond`. Pops top. If $\neq 0$, execute `body` and repeat.

### 2.5 Temporal Operations
| Opcode | Stack Effect | Description |
| :--- | :--- | :--- |
| `ORACLE` | `( addr -- val )` | Read $val = A[addr]$. |
| `PROPHECY` | `( val addr -- )` | Write $P[addr] \leftarrow val$. |
| `PRESENT` | `( addr -- val )` | Read $val = P[addr]$ (current epoch state). |
| `PARADOX` | `( -- )` | Abort epoch. Declare state **Paradoxical**. |
| `HALT` | `( -- )` | Terminate epoch immediately. |

## 3. Semantics

### 3.1 Fixed-Point Semantics
Let $F(S)$ be the state transformation function defined by the program, where input state is $A$ and output state is $P$.
The program is a search for a fixed point $S$ such that:
$$ S = F(S) $$
where equality is defined over the memory contents.

### 3.2 Convergence Status
*   **Consistent**: $P = A$.
*   **Oscillation**: Sequence of states $S_1, S_2, \dots$ such that $S_{t+k} = S_t$ for $k > 1$. (e.g. Grandfather Paradox).
*   **Divergence**: Sequence of states where values grow monotonically or chaotically without repetition within the epoch limit.

## 4. SMT Encoding
Programs can be compiled to SMT-LIB2 logic `QF_ABV` (Quantifier-Free Arrays and Bit-Vectors).
*   **Logic**: `(assert (= Present Anamnesis))`
*   **Control Flow**: Encoded via `ite` (If-Then-Else) terms and bounded unrolling for loops.
