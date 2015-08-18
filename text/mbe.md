% Macros

Introduction to MBE, the rules, limitations, etc.

# Syntax extensions

> **TODO**: Talk about the various classes of syntax extension, and how they are processed.  In particular: prior to names and types are resolved.  They *aren't* items and aren't in the same namespace as anything else.  List major limitations.

## Token trees

> **TODO**: What is a token, what is a token tree.  Mention NTs.

## Invocation syntax

> **TODO**: The `ident ! $tt` syntax, that it might be a syntax extension (not a macro).

## Expansion

> **TODO**: Macros are expanded from the outside in.  The expansion can contain macro invocations, and will be forced into one a limited set of syntax elements.  Note the recursion limit and that it can be raised.

# Macro-by-example

> **TODO**: Introduce `macro_rules!` and the basic syntax.  Perhaps also a reminder that recursion is handled *after* expansion, not *during*.

## Processing

> **TODO**: How MR handles an invocation.  First example should be literal token matching.

## Captures

> **TODO**: Second example: introduce straight captures and substitutions.  Full list of capture kinds.

## Repetitions

> **TODO**: Third example: add repetitions with and without captures.  Note that repetitions must have a consistent "depth", and can't be mixed.

# Details

## Captures and Expansion Redux

> **TODO**: Captures are unabortable.

> **TODO**: Captures restrict what can come after (TY_FOLLOW).

> **TODO**: Substitutions create NTs (except for TTs), which *cannot* be destructured afterwards.  Good example are meta items.

## Hygiene

> **TODO**: Show syntax context colouring.

## Debugging

> **TODO**: `trace_macros!`, `log_syntax!`, `--pretty expanded`, `--pretty expanded,hygiene`.

## Scoping

> **TODO**: scoping in modules, impls, functions.

## Import/Export

> **TODO**: `macro_use`, `macro_export`.

> **TODO**: `$crate`.
