% Patterns

Parsing and expansion patterns.

# Repetition replacement

```rust
macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {$sub};
}
```

This pattern is where a matched repetition sequence is simply discarded, with the variable being used to instead drive some repeated pattern that is related to the input only in terms of length.

For example, consider constructing a default instance of a tuple with more than 12 elements (the limit as of Rust 1.2).

```rust
macro_rules! tuple_default {
    ($($tup_tys:ty),*) => {
        (
            $(
                replace_expr!(
                    ($tup_ty)
                    Default::default()
                ),
            )*
        )
    };
}
```

> **JFTE**: we *could* have simply used `$tup_tys::default()`.

Here, we are not actually *using* the matched types.  Instead, we throw them away and instead replace them with a single, repeated expression.  To put it another way, we don't care *what* the types are, only *how many* there are.

# Trailing separators

```rust
macro_rules! match_exprs {
    ($($exprs:expr),* $(,)*) => {...};
}
```

There are various places in the Rust grammar where trailing commas are permitted.  The two common ways of matching (for example) a list of expressions (`$($exprs:expr),*` and `$($exprs:expr,)*`) can deal with *either* no trailing comma *or* a trailing comma, but *not both*.

Placing a `$(,)*` repetition *after* the main list, however, will capture any number (including zero or one) of trailing commas, or any other separator you may be using.

Note that this cannot be used in all contexts.  If the compiler rejects this, you will likely need to use multiple arms and/or incremental matching.
