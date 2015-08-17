% Patterns

Parsing and expansion patterns.

## Incremental TT munchers

```rust
macro_rules! mixed_rules {
    () => {};
    (trace $name:ident; $($tail:tt)*) => {
        {
            println!(concat!(stringify!($name), " = {:?}"), $name);
            mixed_rules!($($tail)*);
        }
    };
    (trace $name:ident = $init:expr; $($tail:tt)*) => {
        {
            let $name = $init;
            println!(concat!(stringify!($name), " = {:?}"), $name);
            mixed_rules!($($tail)*);
        }
    };
}
```

This pattern is perhaps the *most powerful* macro parsing technique available, allowing one to parse grammars of significant complexity.

A "TT muncher" is a recursive macro that works by incrementally processing its input one step at a time.  At each step, it matches and removes (munches) some sequence of tokens from the start of its input, generates some intermediate output, then recurses on the input tail.

The reason for "TT" in the name specifically is that the unprocessed part of the input is *always* captured as `$($tail:tt)*`.  This is done as a `tt` repetition is the only way to *losslessly* capture part of a macro's input.

The only hard restrictions on TT munchers are those imposed on the macro system as a whole:

* You can only match against literals and grammar constructs which can be captured by `macro_rules!`.
* You cannot match unbalanced groups.

It is important, however, to keep the macro recursion limit in mind.  `macro_rules!` does not have *any* form of tail recursion elimination or optimisation.  It is recommended that, when writing a TT muncher, you make reasonable efforts to keep recursion as limited as possible.  This can be done by adding additional rules to account for variation in the input (as opposed to recursion into an intermediate layer), or by making compromises on the input syntax to make using standard repetitions more tractable.

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

## Push-down Accumulation

```rust
macro_rules! init_array {
    (@accum (0, $_e:expr) -> ($($body:tt)*))
        => {init_array!(@as_expr [$($body)*])};
    (@accum (1, $e:expr) -> ($($body:tt)*))
        => {init_array!(@accum (0, $e) -> ($($body)* $e,))};
    (@accum (2, $e:expr) -> ($($body:tt)*))
        => {init_array!(@accum (1, $e) -> ($($body)* $e,))};
    (@accum (3, $e:expr) -> ($($body:tt)*))
        => {init_array!(@accum (2, $e) -> ($($body)* $e,))};
    (@as_expr $e:expr) => {$e};
    [$e:expr; $n:tt] => {
        {
            let e = $e;
            init_array!(@accum ($n, e.clone()) -> ())
        }
    };
}

let strings: [String; 3] = init_array![String::from("hi!"); 3];
```

All macros in Rust **must** result in a complete, supported syntax element (such as an expression, item, *etc.*).  This means that it is impossible to have a macro expand to a partial construct.

One might hope that the above example could be more directly expressed like so:

```rust
macro_rules! init_array {
    (@accum 0, $_e:expr) => {/* empty */};
    (@accum 1, $e:expr) => {$e};
    (@accum 2, $e:expr) => {$e, init_array!(@accum 1, $e)};
    (@accum 3, $e:expr) => {$e, init_array!(@accum 2, $e)};
    [$e:expr; $n:tt] => {
        {
            let e = $e;
            [init_array!(@accum $n, e)]
        }
    };
}
```

The expectation is that the expansion of the array literal would proceed as follows:

```ignore
            [init_array!(@accum 3, e)]
            [e, init_array!(@accum 2, e)]
            [e, e, init_array!(@accum 1, e)]
            [e, e, e]
```

However, this would require each intermediate step to expand to an incomplete expression.  Even though the intermediate results will never be used *outside* of a macro context, it is still forbidden.

Push-down, however, allows us to incrementally build up a sequence of tokens without needing to actually have a complete construct at any point prior to completion.  In the example given at the top, the sequence of macro invocations proceeds as follows:

```ignore
init_array! { String:: from ( "hi!" ) ; 3 }
init_array! { @ accum ( 3 , e . clone (  ) ) -> (  ) }
init_array! { @ accum ( 2 , e.clone() ) -> ( e.clone() , ) }
init_array! { @ accum ( 1 , e.clone() ) -> ( e.clone() , e.clone() , ) }
init_array! { @ accum ( 0 , e.clone() ) -> ( e.clone() , e.clone() , e.clone() , ) }
init_array! { @ as_expr [ e.clone() , e.clone() , e.clone() , ] }
```

As you can see, each layer adds to the accumulated output until the terminating rule finally emits it as a complete construct.

The only critical part of the above formulation is the use of `$($body:tt)*` to preserve the output without triggering parsing.  The use of `($input) -> ($output)` is simply a convention adopted to help clarify the behaviour of such macros.

Push-down accumulation is frequently used as part of [incremental TT munchers](#incremental-tt-munchers), as it allows arbitrarily complex intermediate results to be constructed.

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

## Trailing separators

```rust
macro_rules! match_exprs {
    ($($exprs:expr),* $(,)*) => {...};
}
```

There are various places in the Rust grammar where trailing commas are permitted.  The two common ways of matching (for example) a list of expressions (`$($exprs:expr),*` and `$($exprs:expr,)*`) can deal with *either* no trailing comma *or* a trailing comma, but *not both*.

Placing a `$(,)*` repetition *after* the main list, however, will capture any number (including zero or one) of trailing commas, or any other separator you may be using.

Note that this cannot be used in all contexts.  If the compiler rejects this, you will likely need to use multiple arms and/or incremental matching.
