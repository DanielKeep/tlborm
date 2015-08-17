% Patterns

Parsing and expansion patterns.

## Internal rules

```rust
#[macro_export]
macro_rules! foo {
    (@as_expr $e:expr) => {$e};

    // ...

    ($($tts:tt)*) => {
        foo!(@as_expr $($tts)*)
    };
}
```

Because macros do not interact with regular item privacy or lookup, any public macro *must* bring with it all other macros that it depends on.  This can lead to pollution of the global macro namespace, or even conflicts with macros from other crates.  It may also cause confusion to users who attempt to *selectively* import macros: they must transitively import *all* macros, including ones that may not be publicly documented.

A good solution is to conceal what would otherwise be other public macros *inside* the macro being exported.  The above example shows how the common `as_expr!` macro could be moved *into* the publicly exported macro that is using it.

The reason for using `@` is that, as of Rust 1.2, the `@` token is *not* used *anywhere* in the Rust grammar; as such, it cannot possibly conflict with anything.  Other symbols or unique prefixes may be used as desired, but use of `@` has started to become widespread, so using it may aid readers in understanding your code.

> **Note**: the `@` token is a hold-over from when Rust used sigils to denote the various built-in pointer types.  `@` in particular was for garbage-collected pointers.

Additionally, internal rules will often come *before* any "bare" rules, to avoid issues with `macro_rules!` incorrectly attempting to parse an internal invocation as something it cannot possibly be, such as an expression.

If exporting at least one internal macro is unavoidable (*e.g.* you have many macros that depend on a common set of utility rules), you can use this pattern to combine *all* internal macros into a single uber-macro.

```rust
macro_rules! crate_name_util {
    (@as_expr $e:expr) => {$e};
    (@as_item $i:item) => {$i};
    (@count_tts) => {0usize};
    // ...
}
```

## Repetition replacement

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
                    ($tup_tys)
                    Default::default()
                ),
            )*
        )
    };
}
```

> **JFTE**: we *could* have simply used `$tup_tys::default()`.

Here, we are not actually *using* the matched types.  Instead, we throw them away and instead replace them with a single, repeated expression.  To put it another way, we don't care *what* the types are, only *how many* there are.

## Trailing separators

```rust
macro_rules! match_exprs {
    ($($exprs:expr),* $(,)*) => {...};
}
```

There are various places in the Rust grammar where trailing commas are permitted.  The two common ways of matching (for example) a list of expressions (`$($exprs:expr),*` and `$($exprs:expr,)*`) can deal with *either* no trailing comma *or* a trailing comma, but *not both*.

Placing a `$(,)*` repetition *after* the main list, however, will capture any number (including zero or one) of trailing commas, or any other separator you may be using.

Note that this cannot be used in all contexts.  If the compiler rejects this, you will likely need to use multiple arms and/or incremental matching.
