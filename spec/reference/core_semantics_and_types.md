# Omni Core Semantics and Type System — Exhaustive Reference

**Status:** Normative Candidate Reference for Edition 1.  
**Consolidation Base:** Modules `OMNI-STD-ROOT`, `OMNI-TERMS`, `OMNI-NAMES`, `OMNI-TYPES`, `OMNI-OWN`, `OMNI-EFFECTS`, and `OMNI-MACHINE`.

---

## 1. Architectural Authority and Normative Taxonomy

### 1.1 Normative Modules
Omni Edition 1 is governed by modular specifications with defined authority boundaries:
- `OMNI-STD-ROOT`: Suite root, authority precedence, product definitions, and edition boundaries.
- `OMNI-TERMS`: Precise behavioral classifications (required, implementation-defined, unspecified, erroneous).
- `OMNI-NAMES`: Lexical scope, rib structures, shadowing rules, and forward function resolution.
- `OMNI-TYPES`: Interned type representations, structural equality, constraint solving, and monomorphization.
- `OMNI-OWN`: Affine ownership, move semantics, non-lexical borrows, and drop elaboration.
- `OMNI-EFFECTS`: Tracked algebraic effect rows and ambient capability authority.
- `OMNI-MACHINE`: Abstract machine store model, byte-level allocation provenance, and trap states.

### 1.2 Non-Negotiable Conformance Rules
- `ROOT-0001`: The normative Edition 1 specification consists exclusively of modules listed as required in the signed release manifest.
- `ROOT-0002`: No compiler implementation or reference interpreter may override or invent language semantics.
- `ROOT-0005`: Safe, well-typed Edition 1 programs SHALL NOT exhibit undefined behavior.
- `TERM-0005`: An erroneous program SHALL NOT produce a conforming release artifact.

---

## 2. Type System Architecture

The type system is implemented across arena interning (`omni-types::intern`), constraint solving (`omni-types::solver`), and type-directed monomorphization (`omni-types::monomorph`).

### 2.1 Type Hierarchy (`TyKind`)

The fundamental categories of types in Omni comprise:

1. **Primitive Scalar Types:**
   - `Int` / `i64`: 64-bit two's complement signed integer.
   - `Float` / `f64`: IEEE 754 double-precision floating-point number.
   - `Bool`: Boolean truth values (`true`, `false`).
   - `Char`: Unicode scalar value (32-bit scalar value).
   - `Byte` / `u8`: 8-bit unsigned octet.
   - `String`: UTF-8 encoded text buffer.
   - `Unit`: 0-tuple `()`, representing empty value.

2. **Generic Parameters & Variables:**
   - `GenericParam(String)`: Syntactic generic type variable (e.g., `T`, `U`) declared on items.
   - `Infer(u32)`: Fresh unification inference variable (`TyVar`) during solver resolution.

3. **Compound & Algebraic Types:**
   - `Tuple(Vec<Ty>)`: Heterogeneous fixed-size product type.
   - `Array(Ty, usize)`: Homogeneous contiguous array with compile-time known length.
   - `Range(Ty)`: Bounded half-open or closed sequence interval.
   - `Fn(Vec<Ty>, Ty)`: Function pointer or closure signature.
   - `Struct(String, Vec<Ty>)`: Nominal product type with named fields and generic type arguments.

### 2.2 Interning Arena (`TyCtxt`)
To achieve $O(1)$ type equivalence checking, `TyCtxt` maintains a deduplicated arena:
- Every unique `TyKind` maps to a single canonical `Ty(u32)` index handle.
- Types are compared by equality of their lightweight handles without deep recursive traversal.

---

## 3. Type Inference and Constraint Solving

Type inference employs bidirectional type checking combined with Union-Find unification:

```text
Abstract Syntax Tree Node
           │
           ▼
Fresh Type Variables Minted (`Solver::new_ty_var`)
           │
           ▼
Constraint Generation & Equivalence Solving
  • `Solver::equate_vars(TyVar, TyVar)`
  • `Solver::equate_var_ty(TyVar, Ty)`
           │
           ▼
Substitution Environment Extraction (`SubstEnv`)
  • Probes solver table for concrete assignments.
  • Rejects unconstrained or ambiguous parameters fail-closed.
```

### 3.1 Unification Invariants
- Two type variables `?A` and `?B` unify by merging their union-find sets.
- A type variable `?A` unifies with a concrete type `Ty` if no conflicting binding exists.
- Incompatible concrete types (e.g., equating `Int` with `String`) produce an unrecoverable `TypeError::MismatchedTypes`.

---

## 4. Type-Directed Multi-Instantiation Monomorphization

Omni replaces legacy single-specialization approximations with fully general, type-directed monomorphization.

### 4.1 Target Architectural Handoff

```text
Generic Source Code
       │
       ▼
Type Inference & Constraint Solving (`TypeChecker::infer_call`)
       │
       ▼
Concrete Substitution Environment (`SubstEnv`: T -> Ty)
       │
       ▼
Deterministic Specialization Key (`SpecializationKey`)
       │
       ▼
Monomorphized Specialized AST (`Monomorphizer::monomorphize_fn`)
       │
       ▼
Mid-Level Intermediate Representation (`omni-mir`)
```

### 4.2 Deterministic, Collision-Free Mangling
Specialized function instances are assigned unique symbol names by mangling their concrete type arguments:
```text
<function_name>_spec_<mangled_type_arguments>
```
Mangling rules:
- `Int` $\to$ `i64`
- `String` $\to$ `String`
- `Bool` $\to$ `bool`
- `Tuple([T1, T2])` $\to$ `tuple_<T1>_<T2>_end`
- `Array(T, N)` $\to$ `arr_<N>_<T>_end`
- `Struct(Name, [T])` $\to$ `struct_<Name>_<T>_end`

**Examples:**
- `identity[i64]` $\to$ `identity_spec_i64`
- `identity[String]` $\to$ `identity_spec_String`
- `complex_fn[bool]` $\to$ `complex_fn_spec_bool`

### 4.3 Active Specialization Environments for Nested Calls
When specializing a generic function `outer[T]` under a substitution environment $\{T \mapsto \text{String}\}$:
1. Every nested call `inner[T](arg)` evaluates `T` within the active `SubstEnv`.
2. `inner` is recognized as an instantiation request for `inner[String]`.
3. The specialization engine recursively monomorphizes `inner_spec_String` and substitutes the call target name in the specialized body.

### 4.4 Recursive Specialization Loop Termination
To prevent infinite monomorphization loops during generic recursion (e.g. `count[T](n)` calling `count[T](n - 1)`):
1. The engine tracks an `in_progress: HashSet<SpecializationKey>` set.
2. When a recursive call targets a key currently present in `in_progress` or the completed cache, the specialized symbol name is emitted immediately without recursive re-entry.

### 4.5 AST Node Substitutions
During monomorphization, generic types are substituted across all expression forms:
- Function parameters and return type annotations.
- Local variable bindings in `let` statements.
- Tuple, array, and range constructions.
- Field projection expressions (`Field`).
- Array and slice indexing expressions (`Index`).
- Pattern matching bindings in match arms (`Match`).
- Anonymous closure parameter types (`Lambda`).
- String interpolation components (`Interpolation`).
- Assignment targets and rvalues (`Assign`).

---

## 5. Affine Ownership and Memory Model

### 5.1 Affine Linear Semantics
- Every heap resource or linear value has exactly one owner place at any point in execution.
- Passing a place by value transfers ownership (move semantics), invalidating the source place.
- Re-use of a moved place triggers a static compile error.

### 5.2 Borrowing & Regions
- Shared borrows (`&T`) permit concurrent read access while forbidding writes or moves.
- Exclusive borrows (`&mut T`) permit read and write access while guaranteeing no alias exists.
- Non-lexical lifetimes (Polonius borrow checking) track liveness based on the control-flow graph rather than syntactic lexical scopes.

### 5.3 Deterministic Destruction
- When an owned value's lifetime terminates, the compiler synthesizes a `Drop` terminator in the MIR basic block.
- Resources are released deterministically in reverse declaration order.
