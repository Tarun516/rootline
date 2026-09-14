# Rust engineering rules

This document defines the Rust implementation rules for Rootline's analysis engine and related Rust crates. It exists primarily to prevent subtle ownership, correctness, portability, and performance mistakes from becoming architectural conventions, especially when code is produced or modified by coding agents.

These rules are binding for Rust changes unless an accepted ADR explicitly changes them. They complement the repository-wide rules in `AGENTS.md` and `docs/12-engineering-operating-model.md`.

The objective is not to make Rootline maximally "clever Rust." The objective is to produce a trustworthy, inspectable, portable, high-performance analysis engine whose behavior remains understandable as the repository grows.

## Core principle

Treat the Rust type system, ownership model, and compiler diagnostics as design feedback.

Do not resolve compiler friction by mechanically adding ownership, indirection, concurrency, or visibility. A change that compiles but weakens the data model, hides an error state, or spreads shared mutable state is not a successful fix.

When the compiler rejects a design, ask in this order:

1. Who should own this value?
2. Does the callee actually need ownership, or only a borrow?
3. Is state stored at the correct layer?
4. Is the lifetime boundary exposing an implementation detail?
5. Is an API asking for more than it needs?
6. Is an abstraction premature?
7. Only then: is cloning, boxing, shared ownership, or interior mutability semantically correct?

## Ownership and borrowing

Prefer borrowing when a function does not retain ownership.

Prefer the general borrowed forms:

```rust
&str
&[T]
&Path
Option<&T>
```

over representation-specific borrowed forms:

```rust
&String
&Vec<T>
&PathBuf
&Option<T>
```

Take owned values when ownership is intentionally transferred or retained.

### Cloning

Do not add `.clone()` merely to satisfy the borrow checker.

Before cloning a non-trivial value, determine why another owner is required. A clone is appropriate when duplication is part of the domain semantics or when measurement shows that a deliberate copy is the best tradeoff. It is not appropriate as a default escape hatch from ownership design.

Agents must not fix move or lifetime errors by sprinkling clones across call sites.

For cheap scalar and small `Copy` values, normal copying is expected.

### Shared ownership

Do not introduce `Rc`, `Arc`, `Mutex`, `RwLock`, `RefCell`, or similar primitives merely because multiple components need to access data.

First determine whether:

- ownership can be moved upward;
- callers can borrow from a longer-lived owner;
- immutable data can be passed explicitly;
- a narrower context object can own the shared state;
- data can be transformed into an owned Rootline IR at a boundary.

`Arc<Mutex<T>>` is not Rootline's default state-management architecture.

## Types should encode meaning

Use the type system to prevent invalid states and accidental mixing of identifiers.

Avoid passing semantically different identifiers as bare `String`, `u64`, or `usize` when a small newtype provides a meaningful boundary.

Likely Rootline-specific types include, as the implementation requires them:

```rust
FileId
SymbolId
RevisionId
SnapshotId
AnalyzerId
FlowId
SubsystemId
ContentHash
```

Do not create all possible newtypes up front. Introduce them when they protect a real invariant or prevent confusion at a boundary.

### Valid construction

A constructor should establish the invariants promised by its type.

Do not derive or implement `Default` for a type unless the default value is semantically valid. Do not create dummy paths, zero IDs, empty revisions, or placeholder states merely to satisfy `Default`.

Prefer explicit constructors when creation requires validation or context.

### Enums over string states

Use enums for closed semantic states.

Prefer:

```rust
enum AnalysisStatus {
    Succeeded,
    Partial,
    Ambiguous,
    Unknown,
    Unsupported,
    Failed,
    Cancelled,
    Stale,
}
```

over strings such as `"failed"` or integer flags.

Do not collapse distinct Rootline outcomes into one generic error or boolean when the distinction matters to users or incremental correctness.

## Paths are not strings

Rootline analyzes arbitrary repositories across platforms. Filesystem-path semantics must remain explicit.

Use:

```rust
&Path
PathBuf
```

at filesystem boundaries.

Do not use `String` as the primary filesystem-path abstraction.

### Lossy conversion

`Path::to_string_lossy()` is permitted for display and diagnostics only.

Never use lossy path text for:

- file identity;
- hashing;
- cache keys;
- module resolution;
- persistence identity;
- graph IDs;
- equality decisions.

Lossy conversion can destroy distinctions present in the original path representation.

### Repository-relative identity

Do not spread ad-hoc path normalization throughout the engine.

When repository-relative path identity becomes part of the implementation, centralize its semantics in a dedicated abstraction such as `RepoPath` or an equivalent accepted type. That abstraction should define:

- whether it is lexical or filesystem-canonicalized;
- whether symlinks are followed;
- separator handling across platforms;
- behavior for `.` and `..`;
- repository-root escape prevention;
- serialization format;
- case-sensitivity assumptions;
- conversion to display strings.

Do not use filesystem canonicalization as a casual normalization primitive: it performs I/O, follows symlinks, may fail, and can alter the meaning of the repository view.

## Source coordinates must be typed

Tree-sitter, compilers, LSPs, text editors, databases, and humans can use different coordinate systems.

Do not represent all source positions as ambiguous `usize`, `u32`, or `(start, end)` tuples.

Use distinct types where coordinate systems cross component boundaries, for example:

```rust
struct ByteOffset(u64);

struct ByteRange {
    start: ByteOffset,
    end: ByteOffset,
}

struct LineIndex(u32);
struct Utf16Column(u32);

struct LspPosition {
    line: LineIndex,
    column: Utf16Column,
}
```

Unless a type explicitly documents otherwise, Rootline ranges should use **start-inclusive, end-exclusive** semantics.

Use checked conversions when narrowing offsets, counts, indexes, file sizes, or persisted integer values. Do not silently truncate repository-scale values with unchecked `as` conversions.

## Parser and third-party boundaries

Parser-specific values must remain inside parser or language-adapter boundaries.

Tree-sitter nodes, trees, cursors, parser lifetimes, and grammar-specific implementation types must not leak into Rootline's stable Code Intelligence IR or graph APIs.

The intended boundary is:

```text
parser-specific representation
        ->
language adapter
        ->
owned Rootline IR
```

This keeps parser lifetimes local, prevents the IR from inheriting third-party implementation constraints, and leaves room for future compiler, LSP, or alternate parser inputs.

Apply the same rule to database libraries, HTTP frameworks, model-provider SDKs, and other third-party systems: do not leak dependency-specific types through stable Rootline boundaries unless the dependency is intentionally part of the public contract.

## Error handling and panics

Repository content is untrusted input.

The following are fallible by default:

- filesystem operations;
- source decoding;
- Git metadata;
- manifests and compiler configuration;
- parser output;
- import and symbol resolution;
- database contents;
- cached artifacts;
- network or semantic-provider responses.

Production engine code must not use `unwrap()` or `expect()` on values derived from these sources.

Malformed or unexpected repository input must produce an explicit error or analysis status. It must not panic the Rootline process.

### Panics

`panic!`, `unreachable!`, and assertion failures are reserved for genuine internal invariants where violation represents a Rootline defect rather than bad user input.

Do not write:

```rust
panic!("unexpected syntax");
```

for syntax controlled by a repository.

Use debug assertions for inexpensive internal consistency checks when appropriate, but do not depend on debug-only checks for user-visible correctness.

### Errors versus analysis outcomes

Not every unresolved condition is a Rust `Err`.

`unsupported`, `ambiguous`, `unknown`, and `partial` can be successful execution outcomes that describe analysis coverage. Preserve that distinction instead of collapsing all uncertainty into generic errors.

Conversely, do not convert an actual analyzer failure into `Ok(empty_result)`.

### Error types

Prefer typed, component-local errors for reusable engine crates. Add contextual wrapping at executable, CLI, or orchestration boundaries.

Do not introduce an error-handling dependency until the relevant crate benefits from it. When dependencies are introduced, keep generic dynamic errors away from stable semantic contracts.

## API design and visibility

Use the narrowest visibility that satisfies the intended architecture:

```text
private -> pub(crate) -> pub
```

A new public item, public re-export, crate dependency, or dependency direction is an architectural change, not a routine implementation detail.

Keep fields private when they participate in an invariant. Expose behavior rather than writable internals.

Avoid setters that allow callers to place a type into invalid intermediate states. Prefer operations that preserve invariants.

### Function signatures

Functions should request only what they need.

Avoid long boolean-heavy signatures such as:

```rust
analyze(path, true, false, true)
```

Use an explicit configuration type when a group of related options forms a coherent input, or use separate operations when the behaviors are semantically distinct.

Do not use `Option<T>` or boolean flags merely to hide multiple substantially different operations behind one function.

### Public APIs and architectural pressure

Do not widen visibility, add a public abstraction, or change an accepted API merely to resolve a local ownership or lifetime problem.

A local compiler error is not sufficient justification for changing a component boundary.

## Start concrete; abstract from evidence

Do not create traits, generic abstraction layers, factories, builders, or `Box<dyn Trait>` solely for hypothetical extensibility.

Start with a concrete implementation when there is only one known implementation and no demonstrated boundary requiring abstraction.

Introduce a trait when at least one of these is true:

- multiple real implementations exist or are immediately required;
- the architecture explicitly defines a provider/adapter boundary;
- the abstraction protects dependency direction;
- testing requires substitution at a meaningful boundary and a simpler approach is insufficient.

Even at valid boundaries, keep traits small and behavior-oriented.

Avoid "doer" types that exist only to call one method unless an internal context object materially simplifies ownership or implementation.

## Safe Rust by default

Rootline application code is safe Rust by default.

Do not introduce `unsafe` to solve compilation errors, borrowing friction, lifetime problems, or speculative performance concerns.

When the first Rust workspace is created, the workspace lint policy should deny unsafe code by default.

If profiling or FFI later establishes a real need for unsafe code, require all of the following:

1. explicit architectural justification;
2. the smallest practical unsafe boundary;
3. documented safety invariants;
4. a `SAFETY:` comment for every unsafe block;
5. focused tests around the boundary;
6. Miri or equivalent validation where applicable;
7. review of whether a safe library/API can provide the same result;
8. an ADR if unsafe becomes part of a stable subsystem design.

Prefer `deny` over an irreversible repository-wide `forbid` so an explicitly reviewed future low-level crate remains possible without weakening unrelated code.

## Async and concurrency

Do not make core analysis APIs async merely because orchestration may use an async runtime.

Parsing, IR normalization, resolution algorithms, graph algorithms, and deterministic transformations should remain runtime-agnostic unless asynchronous behavior is fundamental to their contract.

Tokio or another runtime may later be appropriate for local serving, network providers, file-I/O orchestration, or cancellation infrastructure. Runtime-specific types must not infect core IR or parser contracts without a deliberate architecture decision.

### Bounded concurrency

Never spawn one task or thread per repository file for an unbounded repository.

Every concurrent repository-scale design must state:

- worker/concurrency limit;
- queue capacity or backpressure behavior;
- cancellation behavior;
- memory implications;
- deterministic merge/order behavior;
- failure propagation semantics.

Unbounded channels are not acceptable for repository-scale work without explicit justification.

Never hold a mutex or write lock across an `.await`.

Prefer message passing, scoped ownership, immutable shared data, and partitioned work before introducing coarse shared mutable state.

## Determinism is a correctness property

Do not allow `HashMap`, `HashSet`, filesystem traversal, thread scheduling, or task-completion order to define persisted or user-visible ordering.

Canonicalize data before:

- snapshot creation;
- persistence where ordering is observable;
- hashing/digests;
- protocol serialization when stable output matters;
- golden test comparison;
- benchmark output.

Parallel execution should produce semantically equivalent deterministic results.

Stable IDs must derive from documented identity rules, not incidental iteration order.

Use `BTreeMap` or sorted collections when ordering is itself part of the semantic contract; otherwise use appropriate efficient structures and sort at deterministic boundaries.

## Performance rules

Rootline is performance-sensitive, but performance is measured rather than assumed.

Do not introduce the following only because they "sound faster":

- custom allocators;
- specialized hashers;
- arenas;
- widespread string interning;
- memory mapping;
- unsafe optimizations;
- custom compact graph encodings;
- hand-written SIMD;
- elaborate worker pools;
- lock-free data structures.

Prefer clear correct code until profiling identifies a bottleneck.

Avoid unnecessary allocations and clones in known hot paths, but do not sacrifice invariants or readability for speculative micro-optimizations.

A meaningful optimization should preserve:

- the benchmark or profile that motivated it;
- before/after measurements;
- correctness tests;
- explanation of the tradeoff when the code becomes less obvious.

Performance claims in PRs should include reproducible evidence.

## Dependency policy

Every new crate increases compile time, supply-chain surface, maintenance work, and possible transitive complexity.

Before adding a dependency, answer:

1. What problem does it solve?
2. Why is `std` insufficient?
3. Why are existing workspace dependencies insufficient?
4. Is the crate actively maintained?
5. What transitive dependencies does it add?
6. Does it contain or enable unsafe code?
7. Does it use a build script or procedural macro?
8. Which default features are enabled, and are all of them needed?
9. Is its license compatible with Rootline?
10. Is it on a hot path or stable architectural boundary?

Do not enable dependency default features without checking what they activate.

Do not add unpinned Git dependencies for ordinary production use.

Do not perform broad lockfile updates as part of an unrelated change.

Once the Rust workspace exists, commit `Cargo.lock` because Rootline is an application/tool, and add automated dependency/advisory/license checks appropriate to the workspace.

## Formatting and linting

Use `rustfmt` rather than repository-specific manual formatting conventions.

Do not spend review time debating formatting that the formatter can decide.

Use Clippy as an engineering tool, not as a substitute for judgment.

Do not enable every pedantic, nursery, or restriction lint at `deny` merely to appear strict. Curate lints around actual Rootline failure modes.

Expected early lint policy once the workspace exists includes enforcement for:

- unsafe code;
- ignored `must_use` results;
- `unwrap()` / `expect()` in production paths;
- debug macros;
- `todo!()` / `unimplemented!()` in committed implementation;
- undocumented unsafe blocks if unsafe is ever permitted.

### Lint suppression

Do not add broad `#[allow(...)]` attributes merely to make CI pass.

Prefer changing the implementation when the lint identifies a real issue.

When suppression is genuinely correct, scope it as narrowly as possible and prefer an expectation with a reason, for example:

```rust
#[expect(
    clippy::some_lint,
    reason = "The allocation is intentional because ..."
)]
```

A lint suppression without an engineering reason is incomplete work.

Do not weaken workspace lint policy as part of an unrelated task.

## Testing rules

Bug fixes require a minimal regression test whenever the failure can be reproduced.

The expected workflow is:

```text
minimal failing fixture
    -> test demonstrates failure
    -> implementation fix
    -> test passes
```

Prefer tests through the relevant component boundary rather than binding tests to private implementation details.

Do not use `#[should_panic]` to test malformed repository input. Assert the explicit error or analysis state.

Keep fixtures minimal, deterministic, and focused on one behavior where practical.

When ordering is not semantically meaningful, canonicalize it before snapshot comparison instead of encoding accidental collection iteration order.

For incremental capabilities, include transition tests and clean-build-versus-incremental convergence tests.

For parsers and resolvers, maintain adversarial fixtures for malformed syntax, Unicode, unusual paths, configuration changes, ambiguous resolution, and unsupported constructs as the relevant capability is introduced.

### Property tests and fuzzing

Property-based and fuzz testing are appropriate when Rootline begins processing sufficiently broad untrusted inputs, especially for:

- repository-path normalization;
- manifest/config parsing;
- import specifiers;
- resolver state transitions;
- graph serialization;
- incremental change sequences.

Do not add fuzz infrastructure before there is a meaningful target and oracle.

## Diagnostics, logging, and privacy

Core engine code should emit structured diagnostics rather than arbitrary user-facing text.

Avoid `println!`, `eprintln!`, and `dbg!` in reusable engine code. The executable, CLI, or client boundary owns presentation.

Do not log repository source contents by default.

Treat the following as potentially sensitive:

- source code;
- environment variables;
- tokens and credentials;
- absolute private filesystem paths;
- repository remote URLs;
- provider request/response contents.

Diagnostics should reveal enough context to debug behavior without casually exfiltrating proprietary repository data.

## Comments and documentation

Comments should explain:

- why an implementation exists;
- which invariant is being preserved;
- a non-obvious algorithmic or ownership constraint;
- why a less obvious tradeoff was selected;
- the safety argument around unsafe code.

Do not narrate obvious syntax.

Public fallible APIs should document meaningful `# Errors` conditions. APIs that intentionally may panic should document `# Panics`. Unsafe APIs must document their `# Safety` contract.

If correctness relies on a subtle invariant, write it near the code that enforces it instead of leaving it only in an architecture document.

## Agent-specific prohibitions

When facing a compiler, lifetime, or Clippy error, coding agents must not reflexively solve it by:

- cloning data;
- wrapping state in `Arc`, `Rc`, `Mutex`, `RwLock`, or `RefCell`;
- adding `'static` bounds;
- boxing values or trait objects;
- adding `unsafe`;
- converting synchronous core logic to async;
- spawning background tasks;
- widening visibility;
- changing a public API;
- adding a trait or generic abstraction;
- adding a lint suppression;
- discarding an error;
- converting a path to a lossy string;
- replacing a typed state with `String`;
- adding a dependency.

First identify the underlying ownership, lifetime, API, state-model, or data-flow issue and make the smallest correct change.

Agents must explain the semantic reason when introducing any of the following in a non-trivial path:

```text
clone
Arc / Rc
Mutex / RwLock / RefCell
Box<dyn Trait>
'static bound
async boundary
spawned task/thread
unsafe
new public API
new dependency
lint suppression
```

The explanation should describe why the construct matches the ownership or architectural model, not merely that it makes the compiler or tests pass.

## Intended machine enforcement

Do not add fake Rust configuration before a Rust workspace exists. When the first Rust vertical slice is introduced, create the machine-enforcement layer with the implementation rather than leaving these rules as prose only.

The initial workspace should deliberately define:

- Rust edition and pinned development toolchain;
- workspace lint inheritance;
- rustfmt configuration kept close to standard formatting;
- Clippy policy focused on Rootline failure modes;
- safe-Rust-by-default policy;
- committed lockfile;
- formatting, check, Clippy, and test CI;
- dependency/advisory/license policy when external crates become meaningful.

A reasonable CI baseline is conceptually:

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
```

Exact commands may evolve with the workspace. Do not cargo-cult flags unsupported by the actual crate graph.

## Rust review checklist

Before considering a Rust change complete, review:

### Correctness

- Are malformed repository inputs handled without panic?
- Are `unknown`, `unsupported`, `ambiguous`, and `failed` still distinct where relevant?
- Are source ranges and IDs using the correct coordinate/identity types?
- Are parser-specific types contained at adapter boundaries?
- Does incremental behavior preserve the last valid published state where applicable?

### Ownership

- Does each long-lived value have a clear owner?
- Were clones introduced for semantic reasons rather than compiler convenience?
- Is shared mutable state genuinely necessary?
- Are borrows narrower than the lifetime of the owning component?

### API

- Is visibility as narrow as possible?
- Did the change accidentally expose a third-party type?
- Is a new trait actually justified?
- Are invalid states difficult to construct?

### Concurrency

- Is parallelism bounded?
- Are queues/backpressure explicit?
- Is cancellation defined?
- Is result merging deterministic?
- Is no lock held across `.await`?

### Performance

- Did performance-motivated complexity come from measurement?
- Are benchmark inputs reproducible?
- Were correctness and memory behavior preserved?

### Hygiene

- Does formatting pass?
- Does Clippy pass without unexplained suppression?
- Do focused and workspace tests pass?
- Did dependency changes remain narrow and intentional?
- Are comments documenting invariants rather than syntax?

## How these rules evolve

Do not expand this document with every stylistic preference encountered in review.

Add or change a rule when it prevents a recurring class of bugs, protects an architectural boundary, captures an important Rust-specific invariant, or is supported by repeated implementation experience.

If a Rust rule materially changes Rootline architecture or a stable contract, record the decision in an ADR as well.

The goal is a small number of strong rules backed by compiler checks, lints, tests, benchmarks, and review—not a large style manual that agents learn to mechanically satisfy.