# Contributing to Kastel

Thank you for contributing to Kastel.

Kastel is a dynamic, interpreted programming language written in Rust, with a bytecode-based virtual machine. Contributions are welcome, but changes to the compiler, VM, runtime, garbage collector, parser, and core language semantics must preserve the stability and consistency of the project.

## Development Workflow

Kastel uses two main branches:

- `master` — stable and release-ready code.
- `develop` — integration branch for the next version.

Do not push directly to `master`.

For normal development, create a dedicated branch from `develop`:

```bash
git switch develop
git pull origin develop

git switch -c feature/my-feature
```

Use a branch name matching the type of change:

```text
feature/*      New functionality
fix/*          Bug fixes
refactor/*     Internal refactoring
test/*         Tests
docs/*         Documentation
perf/*         Performance improvements
build/*        Build or dependency changes
ci/*           CI/CD changes
```

Examples:

```text
feature/dict
feature/range-iterator
fix/module-import
fix/gc-upvalue
refactor/value
test/parser
perf/vm-dispatch
docs/language-reference
```

When the work is complete, push the branch:

```bash
git push -u origin feature/my-feature
```

Then open a Pull Request targeting:

```text
feature/my-feature → develop
```

Pull Requests targeting `master` should normally be used only for releases or explicitly approved maintenance.

## Before Opening a Pull Request

Make sure the project builds and tests successfully:

```bash
cargo check --all-targets
cargo test --all-targets
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

Do not submit a Pull Request with failing tests, formatting errors, or avoidable compiler warnings.

When adding a new feature, add tests whenever practical.

For language features, prefer tests that verify both successful execution and invalid input/error handling.

## Commit Messages

Use clear and consistent commit messages.

Recommended format:

```text
type: description
```

Common types:

```text
feat:     New functionality
fix:      Bug fix
refactor: Internal code restructuring
test:     Tests
docs:     Documentation
perf:     Performance improvement
build:    Build changes
ci:       CI changes
```

Examples:

```text
feat: add dictionary literals
fix: resolve module import error
refactor: simplify value representation
test: add closure execution tests
perf: optimize bytecode dispatch
docs: update range syntax
```

Keep commits focused. Avoid mixing unrelated changes in the same commit.

Bad:

```text
fix everything
```

Better:

```text
fix: preserve object field insertion order
```

## Pull Requests

A Pull Request should contain a focused and understandable change.

The description should explain:

- What was changed.
- Why the change was necessary.
- How the change was tested.
- Whether the change modifies Kastel language semantics, VM behavior, compiler behavior, or public APIs.

For example:

```text
## Summary

Add lazy range iteration for `for ... in`.

## Changes

- Add range iterator runtime object.
- Add iterator bytecode support.
- Update `for ... in` compilation.
- Add range iteration tests.

## Testing

- cargo check --all-targets
- cargo test --all-targets
- cargo clippy --all-targets --all-features -- -D warnings
```

Keep Pull Requests reasonably small when possible. Large architectural changes should be divided into logical steps.

## Code Style

Kastel is written in Rust.

Follow standard Rust conventions and let `rustfmt` format the code:

```bash
cargo fmt --all
```

Prefer code that is:

- Explicit.
- Maintainable.
- Easy to test.
- Consistent with the existing architecture.
- Free of unnecessary dependencies.

Do not introduce a third-party crate when the functionality can reasonably be implemented using the Rust standard library or existing Kastel infrastructure.

Avoid unnecessary rewrites of unrelated parts of the codebase.

## Language and Runtime Changes

Changes affecting Kastel's language semantics require additional care.

Examples include:

```text
Parser
Compiler
Bytecode
Opcode definitions
Value representation
Virtual Machine
Garbage Collector
Closures / Upvalues
Modules
Objects / Classes
Iterators
Error handling
Formatting
```

Before modifying one of these systems, inspect the existing implementation and tests.

A change should not silently alter existing Kastel behavior unless that behavior is intentionally being changed.

When language behavior changes, update the relevant tests and documentation.

## Breaking Changes

Breaking changes must be clearly identified in the Pull Request.

Examples:

```text
Syntax changes
Opcode changes
Runtime representation changes
Module system changes
Public API changes
Behavior changes
```

Do not make a breaking language change silently.

Explain the previous behavior and the new behavior.

Example:

```text
Before:

for (x in range(10)) { ... }

After:

for x in range(10) { ... }
```

## Tests

Tests are an important part of Kastel development.

When possible, test:

```text
Normal behavior
Edge cases
Invalid input
Runtime errors
Compiler errors
Parser errors
Regression cases
```

A bug fix should ideally include a regression test demonstrating the previous failure.

Do not remove an existing test simply because it fails after a code change. Determine whether the implementation or the test is incorrect first.

## Examples

Language examples are kept in the `examples/` directory.

When a language feature changes user-visible syntax or behavior, update affected examples when appropriate.

Example programs should remain understandable and should use valid current Kastel syntax.

## Issues

Before starting a large change, check existing GitHub Issues and Pull Requests to avoid duplicating work.

For bug reports, include:

```text
Kastel version / commit
Operating system
Minimal reproduction
Expected behavior
Actual behavior
Relevant error message
```

A minimal example is strongly preferred.

Example:

```text
Kastel commit: <commit>

Code:

let x = ...

Expected:
...

Actual:
...
```

## Security

Do not publicly disclose security-sensitive information in a GitHub Issue.

For a potentially serious vulnerability, contact the project maintainer privately before publishing technical details.

## Review Process

Pull Requests are reviewed for:

```text
Correctness
Tests
Architecture
Maintainability
Performance
Compatibility
Code quality
```

A Pull Request may be requested to change before it is merged.

Approval does not mean that all future behavior of the implementation is guaranteed. The project may continue to evolve as the architecture develops.

## Branch Integration

The normal integration path is:

```text
feature/*
fix/*
refactor/*
test/*
docs/*
      │
      ▼
   develop
      │
      ▼
   testing
      │
      ▼
   master
      │
      ▼
   release
```

The `master` branch must remain stable.

## Getting Started

Clone the repository:

```bash
git clone https://github.com/MJBruno/Kastel.git
cd Kastel
```

Switch to the development branch:

```bash
git switch develop
git pull origin develop
```

Create a working branch:

```bash
git switch -c feature/my-feature
```

Build and test:

```bash
cargo check
cargo test
```

Format:

```bash
cargo fmt
```

Run Kastel:

```bash
cargo run -- examples/main.ks
```

## Project Principles

Contributions should help Kastel remain:

- Simple to understand.
- Predictable.
- Correct.
- Maintainable.
- Testable.
- Efficient.
- Consistent with its language design.

The goal is not only to add functionality, but to keep the language and runtime coherent as the project grows.

## Maintainer

The project maintainer has final authority over:

- Language design.
- Runtime architecture.
- Compiler architecture.
- Public APIs.
- Breaking changes.
- Release decisions.
- Branch integration.

Contributors are encouraged to propose alternatives and improvements, especially when supported by tests, benchmarks, or concrete technical reasoning.

---

## License

By contributing to Kastel, you agree that your contributions may be distributed under the project's license.