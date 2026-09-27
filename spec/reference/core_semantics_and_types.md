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

---

## 6. Authoritative Trait System and Coherence

Omni Edition 1 enforces a unified, type-directed trait system (`omni-traits`) that acts as the sole semantic authority for trait declarations, implementations, obligations, and coherence.

### 6.1 Trait and Implementation Declarations
- **Trait Definition (`TraitDef`):** Declares an interface containing required and provided methods, along with optional supertrait dependencies.
- **Implementation Definition (`ImplDef`):** Implements a trait for a specific target type (`target_ty`), optionally constrained by generic predicates.
- **Completeness Invariant:** An `impl` must supply definitions for all required methods lacking default implementations in the trait definition. Failure triggers deterministic diagnostic `MissingRequiredMethod`.

### 6.2 Deterministic Registration Errors
Registration of traits and implementations is strictly validated at compile time. The compiler halts with deterministic diagnostics upon encountering:
1. `DuplicateTrait(name)`: Multiple trait declarations with the same nominal identifier.
2. `UnknownTrait(name)`: Implementation targeting an undeclared trait.
3. `DuplicateImpl(trait, target)`: Multiple identical implementation blocks for the same type.
4. `MissingRequiredMethod(trait, method)`: Omitting required methods in an impl.
5. `InvalidSupertrait(trait, supertrait)`: Undeclared supertrait dependency.
6. `ConflictingImplementation(trait, target, reason)`: Overlapping implementation without disambiguating specialization.
7. `CoherenceOrphanViolation(trait, target)`: Attempting to implement a foreign trait for a foreign type.
8. `NegativeBoundConflict(trait, target)`: Violating an explicit negative bound (`!Trait`).

### 6.3 Strict Coherence (Orphan Rules)
To guarantee coherence across independently compiled modules, an `impl Trait for Type` is valid if and only if at least one of the following conditions holds:
1. **Local Trait:** The trait is defined within the local compilation unit.
2. **Local Type:** The target type is defined within the local compilation unit.

An implementation of a foreign trait for a foreign type (`!is_local_trait && !is_local_type`) is strictly rejected with `CoherenceOrphanViolation`.

### 6.4 SLG Resolution and Bound Satisfaction
Trait obligations are resolved via Selective Linear Definite Clause (SLG) resolution with tabling and cycle detection:
- **Positive Bounds (`T: Trait`):** Evaluated against registered facts, supertraits, and conditional implementation rules.
- **Negative Bounds (`T: !Trait`):** Evaluated to ensure the target type does not satisfy `Trait` and matches explicit negative facts.
- **SubstEnv Integration:** When a generic function `foo[T: Trait](x: T)` is instantiated, type inference derives the concrete `SubstEnv`. Trait obligations are resolved immediately against the concrete substituted types before specialization proceeds. If any obligation fails, `TypeError::TraitObligationUnsatisfied` is returned.

### 6.5 Concrete Semantic Gate Before MIR
Before AST/HIR is lowered to Mid-Level IR (`omni-mir`), the compilation unit must pass `assert_concrete_for_mir`:
1. Every function must have zero unresolved generic type parameters (`type_params.is_empty()`).
2. Every trait obligation must be fully satisfied and discharged (`bounds.is_empty()`).
3. Every call expression must be specialized to a concrete function symbol without leftover `generic_args`.
4. Inference variables must be fully resolved to concrete types.
Any violation halts compilation before code generation.

---

## 7. Pattern Typing, Usefulness, and Exhaustive Matching Engine

Omni Edition 1 enforces compile-time pattern usefulness and exhaustiveness analysis using a formalized Maranget pattern matrix reduction engine (`omni-types::pattern::PatternChecker`).

### 7.1 Scrutinee-Directed Pattern Typing
- Every pattern matching expression `match scrutinee { arm_1, ..., arm_n }` requires a well-typed scrutinee expression yielding a concrete type `S`.
- Pattern forms are type-checked directly against `S`:
  1. **Wildcard (`_`) and Binding (`x`):** Assigns type `S` to the bound pattern variable in the arm's scope.
  2. **Literal (`Lit`):** Enforces that the literal pattern type equals `S` (e.g. `Int`, `Bool`, `String`).
  3. **Tuple (`Tuple(pats)`):** Requires `S` to be `TyKind::Tuple(elem_tys)` with matching arity; element subpatterns are type-checked against `elem_tys`.
  4. **Struct (`Struct { fields, .. }`):** Requires `S` to be `TyKind::Struct(name, field_tys)`; field subpatterns are type-checked against corresponding struct field types.
  5. **Enum Variant (`Variant { enum_name, variant, subpatterns }`):** Requires `S` to be `TyKind::Enum(enum_name, type_args)`. Subpatterns are typed against generic-substituted variant payload types derived via `SubstEnv`.
  6. **Bounded Range (`Range { start, end }`):** Valid for scalar types (`Int`, `Byte`, `Char`).
  7. **Or-Pattern (`P1 | P2`):** Requires both `P1` and `P2` to type-check against `S` and introduce identical variable binding signatures.
  8. **Never (`!`):** Valid for scrutinee type `S = TyKind::Never` (uninhabited type).

### 7.2 Maranget Matrix Usefulness Algorithm
A pattern row `v` is **useful** with respect to a matrix `M` of previously accepted patterns if there exists a value of type `S` matched by `v` but not matched by any row in `M`.
- **Matrix Reduction (Specialization):**
  - When the top constructor `c` of `v` is known, `M` and `v` are specialized against `c` (`specialize_matrix` / `specialize_vector`), reducing subpattern columns.
  - When `v` starts with a Wildcard/Binding:
    - If the head constructor set of `S` is finite and complete in `M`, usefulness is evaluated recursively across all constructors `c ∈ constructors(S)`.
    - If the constructor set is incomplete or infinite (`Int`, `String`), reduction falls back to the default matrix (`default_matrix`).

### 7.3 Guard Semantics & Unconditional Coverage
- **Guarded Arms (`MatchArm { pattern, guard: Some(expr), .. }`):**
  - A guarded arm is checked for usefulness against `M`.
  - **Critical Invariant:** If useful, a guarded arm is executed when its guard evaluates to `true`, but it **does not** contribute to matrix reduction for subsequent arms. Unguarded matrix `M` does not retain guarded rows during exhaustiveness determination.
- **Unconditional Arms (`guard: None`):**
  - Contributes to `M` upon passing usefulness analysis.

### 7.4 Exhaustiveness and Witnesses
- A match expression is **exhaustive** if every possible inhabitant of `S` is covered by at least one unguarded arm in `M`.
- **Exhaustiveness Test:** The engine tests if a synthetic wildcard row `[_]` is useful against `M`. If useful, the match is non-exhaustive and a witness string is generated (`missing_witness`).
- **Deterministic Diagnostics:**
  - `TypeError::NonExhaustiveMatch { scrutinee_ty, missing }`: Emitted when `[_]` is useful.
  - `TypeError::UnreachablePattern { arm_index, detail }`: Emitted when an arm row `v_i` is not useful relative to `M_{i-1}`.

### 7.5 Uninhabited Type (`Never`) Matching
- The `Never` type (`!`) has 0 inhabitants.
- An empty match expression `match never_val {}` or an arm `! => ...` on `scrutinee_ty = TyKind::Never` is **inherently exhaustive**.
- Wildcard rows against `TyKind::Never` yield `is_exhaustive = true` regardless of arm count.

### 7.6 Lowering & Verification Invariants
Every `Match` expression passing type-checking and lowering to MIR satisfies:
1. Scrutinee expression is well-typed with a concrete substituted type.
2. Every arm is useful (no `UnreachablePattern`).
3. Pattern arms unconditionally cover 100% of scrutinee type inhabitants (no `NonExhaustiveMatch`).
4. All pattern bindings are registered in the arm local scope with concrete types.


