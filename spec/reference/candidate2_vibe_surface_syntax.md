# Candidate 2 Amendment: Vibe-First Surface Syntax — Exhaustive Specification

**Status:** Candidate amendment for Omni Edition 1; not yet ratified or implementation-certified.  
**Supersession Scope:** Explicit surface-syntax contracts only. All semantic guarantees, affine ownership rules, effect/capability models, dynamic rules, and ABI specifications remain unchanged.

---

## 1. Architectural Philosophy and Scope

Omni Candidate 2 introduces a **vibe-first surface syntax** designed to minimize syntax ceremony, eliminate visual clutter, and make code composition feel fluid and intent-driven for both human authors and LLM code generators.

### 1.1 Non-Negotiable Invariants
1. **No Natural-Language Ambiguity:** Vibe-first syntax is strictly deterministic. The compiler never employs heuristic natural-language processing or non-deterministic English grammar parsing.
2. **One Deterministic Parse:** Every valid source token sequence yields exactly one concrete syntax tree (CST).
3. **One Deterministic Desugaring:** Every surface convenience node lowers deterministically to a canonical core AST/HIR node.
4. **Zero Semantic Bypass:** Surface syntax never conceals or weakens:
   - Affine ownership transfers and linear resource consumption.
   - Borrow lifetimes and region constraints.
   - Capability escalation, requirements, or dynamic checks.
   - Effect handler boundaries and unhandled effect tracking.
   - Unsafe memory operations or foreign call boundaries.
   - Task cancellation semantics and join points in concurrency.

---

## 2. Compilation and Lowering Pipeline Architecture

The translation pipeline enforces strict phase separation:

```text
Source Text (UTF-8 Bytes)
    │
    ▼
Lossless Lexer (`omni-lex`)
  • Produces concrete tokens and trivia (comments, whitespace).
  • Implements context-sensitive split (`>>` in type arguments becomes `>` `>`).
  • Tracks layout state for newline-continuation analysis.
    │
    ▼
Parser Events (`omni-parse`)
  • Emits opening/closing/element events.
  • Performs precedence climbing and newline-boundary resolution.
    │
    ▼
Lossless Concrete Syntax Tree (CST - `omni-syntax`)
  • 100% round-trip fidelity to source text.
  • Rowan-based green/red tree architecture.
    │
    ▼
Surface Abstract Syntax Tree (Surface AST)
  • Explicit surface nodes: `PipelineExpr`, `ProjectionShorthand`, `CommandCall`, `ResourceScope`.
    │
    ▼
Deterministic Vibe Desugaring (`omni-parse::desugar`)
  • Transforms surface constructs into canonical core AST/HIR representations.
  • Attaches source provenance and diagnostics.
    │
    ▼
Core Semantic AST / High-Level IR (HIR - `omni-hir`)
  • Name Resolution (`omni-names`).
  • Bidirectional Type Checking & Monomorphization (`omni-types`).
  • Affine Ownership & Borrow Checker (`omni-own`).
  • Effect & Capability Verification (`omni-effects`).
    │
    ▼
Mid-Level IR (MIR - `omni-mir`)
  • Control-flow graph, basic blocks, terminators, drop elaborators.
    │
    ▼
Mechanical Verifier (`omni-verify`) & Code Generation (`omni-codegen`)
  • Polonius borrow verification, translation validation.
  • Cranelift or LLVM native object emission.
```

---

## 3. Layout, Statement Termination, and Newline Continuations

Earlier Edition 1 rules `LEX-0002`, `GRAM-0002`, and `GRAM-0003` required strict explicit semicolons or rigid lexical layout. Under the Candidate 2 surface amendment, they are superseded by `VIBE-GRAM-0001` through `VIBE-GRAM-0007`.

### 3.1 Formal Rules

| Rule Identifier | Normative Specification |
|---|---|
| `VIBE-GRAM-0001` | A newline (`\n` or `\r\n`) MAY terminate a statement when the parser is at a complete statement boundary and the next token cannot continue the current expression or declaration. |
| `VIBE-GRAM-0002` | A newline MUST NOT terminate a statement when the current construct is syntactically incomplete or the next token is a designated continuation token. |
| `VIBE-GRAM-0003` | Designated continuation tokens include: binary operators (`+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `&&`, `\|\|`, `^`, `&`, `\|`, `\|>`), navigation operators (`.`, `?.`, `::`), question operators (`?`, `??`), opening delimiters (`(`, `[`, `{`), closing delimiters followed by chaining, and comma (`,`). |
| `VIBE-GRAM-0004` | Semicolons (`;`) MAY be used explicitly and remain standard statement separators; they are not required at statement boundaries where newline termination is unambiguous. |
| `VIBE-GRAM-0005` | Formatter output MUST be canonical and MUST NOT rely on indentation alone to alter the semantic interpretation of an already valid expression. |
| `VIBE-GRAM-0006` | Braced blocks (`{ ... }`) remain valid in every context where the grammar admits blocks. Indentation-first formatting is preferred for presentation but does not create an alternate semantic language. |
| `VIBE-GRAM-0007` | Release translation MUST reject source for which newline interpretation remains ambiguous after the continuation rules are applied. IDE recovery MAY construct recovery nodes, but those nodes remain non-translatable. |

### 3.2 Parsing Examples

#### Trailing Binary Operator Continuation
```omni
let total =
    base_price +
    sales_tax +
    shipping_fee
```
*Parser state:* After `base_price +`, the expression is incomplete. The newline is skipped as trivia; the expression continues across all three lines into a single `LetStmt`.

#### Leading Binary Operator Continuation
```omni
let total =
    base_price
    + sales_tax
    + shipping_fee
```
*Parser state:* `+` is a designated continuation token. Even though `base_price` could form a complete expression, the following non-trivia token is `+`, which continues the binary expression.

#### Statement Boundary
```omni
let x = 10
let y = 20
```
*Parser state:* `10` completes the let binding. The next non-trivia token is `let`, which cannot legally follow an expression as a continuation. The newline is recognized as a statement terminator.

---

## 4. Pipeline Operator (`|>`)

Candidate 2 promotes `|>` from reserved punctuation into a first-class left-associative pipeline operator.

### 4.1 Syntax and Precedence
- **Precedence:** Lower than function calls, unary operators, and multiplicative/additive operators, but higher than assignment and statement termination.
- **Associativity:** Left-to-right:
  ```omni
  x |> f |> g
  ```
  is parsed as:
  ```omni
  ((x |> f) |> g)
  ```

### 4.2 Desugaring Semantics
For any expression `e` and function or callable target `f`:
1. **Unary Call:**
   ```omni
   value |> f
   ```
   desugars to:
   ```omni
   f(value)
   ```
2. **N-ary Call with Pre-Supplied Arguments:**
   ```omni
   value |> f(a, b)
   ```
   desugars to inserting the piped value into the primary (first) parameter slot:
   ```omni
   f(value, a, b)
   ```

### 4.3 Preservation Invariants
Pipeline desugaring guarantees:
- Exact evaluation order: `value` is evaluated strictly before `f`, `a`, and `b`.
- Ownership transfer: If `f` takes `value` by value (`T`), ownership moves linearly without implicit copies.
- Borrowing: If `f` takes `&T` or `&mut T`, the borrow begins and ends under standard region semantics.
- Effects & Capabilities: Pipeline chains propagate tracked effect rows without alteration.

---

## 5. Projection Shorthand (`.field`)

In projection contexts (such as functional iteration methods or map/filter lambdas), a leading dot followed by an identifier represents an implicit field projection closure.

### 5.1 Syntax and Examples
```omni
let user_names = users |> map .name
let active_users = users |> filter .is_active
```

### 5.2 Desugaring Contract
In a designated projection context expecting a callable argument `Fn(T) -> R`:
```omni
.name
```
desugars into a synthetic anonymous closure:
```omni
|#param| #param.name
```
where `#param` is a hygiene-isolated identifier bound to the element type `T`.

### 5.3 Ambiguity Boundaries
- `.field` is accepted only where the grammar permits a projection expression.
- It cannot appear as an isolated statement or unbound value.
- Chained projections (e.g. `.address.city`) desugar into nested field accesses: `|#param| #param.address.city`.

---

## 6. Command-Style Calls

To eliminate superfluous parentheses in script-like and procedural commands, Candidate 2 permits command calls:

### 6.1 Syntax
```omni
print "Hello, World!"
save user
sleep 5.seconds
log info, "Job completed"
```

### 6.2 Disambiguation Rules
A command-style call `callee arg1, arg2` is valid if and only if:
1. `callee` is a syntactically valid callee path.
2. The argument list begins immediately without an intervening operator that could form an infix expression.
3. The call boundary is uniquely determinable without lookahead beyond the statement boundary.
4. If an expression is syntactically ambiguous (such as nested command calls), parenthesized form `callee(arg1, arg2)` is required.

---

## 7. Optional Chaining and Null-Coalescing

Candidate 2 standardizes concise safe navigation and fallback operators:

### 7.1 Optional Chaining (`?.`)
```omni
let city = user?.address?.city
```
- Operates on types inhabiting `Option[T]` or nullable references.
- Short-circuits: if `user` evaluates to `None`, the remainder of the chain is skipped, and the entire expression yields `None`.

### 7.2 Null Coalescing (`??`)
```omni
let display_city = user?.address?.city ?? "Default City"
```
- Right-associative operator.
- Unwraps `Some(val)` into `val`.
- If the left-hand operand is `None`, evaluates and returns the right-hand operand.

---

## 8. Resource Scopes (`with`)

To guarantee deterministic resource cleanup without cumbersome try/finally blocks, Candidate 2 introduces the `with` statement.

### 8.1 Syntax
```omni
with file = open_file(path)? {
    let data = file.read_all()
    process(data)
}
```

### 8.2 Desugaring Contract
`with expr = init { body }` desugars into:
1. Linear binding of `expr`.
2. Execution of `body` within a protected drop scope.
3. Deterministic invocation of the type's `Drop` or `Close` destructor at scope exit (whether by normal return, early break, or error propagation).

---

## 9. Structured Concurrency Surface (`parallel`)

Candidate 2 introduces structured task scopes where all child tasks are bound to the lexical lifetime of the block.

### 9.1 Syntax
```omni
parallel {
    profile = fetch_profile(user_id)
    posts = fetch_posts(user_id)
}
```
and concurrent iterations:
```omni
parallel for item in items {
    process(item)
}
```

### 9.2 Invariants and Guarantees
- **No Orphan Tasks:** The enclosing `parallel` block does not exit until all spawned tasks complete or handle errors.
- **Fail-Fast Cancellation:** If any task encounters an unhandled fault, sibling tasks in the same scope receive cancellation tokens.
- **Deterministic Happens-Before:** All stores executed inside the scope happen-before execution resumes after the block.

---

## 10. Explicit Authority Declarations (`requires`)

In Omni, capabilities (authority) are strictly separated from effects (behavior). Candidate 2 introduces a dedicated clause for declaring authority requirements on function signatures:

### 10.1 Syntax
```omni
fn send_audit_log(entry: AuditEntry)
    requires Network.Connect, Storage.Append
    -> Result<(), LogError> / io
{
    ...
}
```

### 10.2 Semantic Invariants
- `requires` explicitly lists the unforgeable capability tokens required to invoke the function.
- Capabilities cannot be implicitly inferred merely from the presence of an effect row (`/ io`).
- The type checker verifies that the caller possesses dynamic authority or enclosing capability tokens before permitting the call.

---

## 11. AI-Generation and Tooling Invariants

To ensure that machine-generated and assisted code remains high-integrity and parse-stable, Candidate 2 enforces:
1. **Canonical Formatting:** All accepted vibe constructs have exactly one canonical representation in `omni-fmt`.
2. **Minimal Repair Regions:** Syntax error diagnostics identify precise local token regions rather than failing cascadingly across the file.
3. **No Indentation Ambiguity:** Indentation assists readability but never changes the AST tree topology of an already unambiguous token sequence.
