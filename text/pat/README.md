% Patterns

Parsing and expansion patterns.

## Callbacks

```rust
macro_rules! call_with_larch {
    ($callback:ident) => { $callback!(larch) };
}

macro_rules! expand_to_larch {
    () => { larch };
}

macro_rules! recognise_tree {
    (larch) => { println!("#1, the Larch.") };
    (redwood) => { println!("#2, the Mighty Redwood.") };
    (fir) => { println!("#3, the Fir.") };
    (chestnut) => { println!("#4, the Horse Chestnut.") };
    (pine) => { println!("#5, the Scots Pine.") };
    ($($other:tt)*) => { println!("I don't know; some kind of birch maybe?") };
}

fn main() {
    recognise_tree!(expand_to_larch!());
    call_with_larch!(recognise_tree);
}
```

Due to the order that macros are expanded in, it is (as of Rust 1.2) impossible to pass information to a macro from the expansion of *another* macro.  This can make modularising macros very difficult.

An alternative is to use recursion and pass a callback.  Here is a trace of the above example to demonstrate how this takes place:

```rust
recognise_tree! { expand_to_larch ! (  ) }
println! { "I don't know; some kind of birch maybe?" }
// ...

call_with_larch! { recognise_tree }
recognise_tree! { larch }
println! { "#1, the Larch." }
// ...
```

Using a `tt` repetition, one can also forward arbitrary arguments to a callback.

```rust
macro_rules! callback {
    ($callback:ident($($args:tt)*)) => {
        $callback!($($args)*)
    };
}

fn main() {
    callback!(callback(println("Yes, this *was* unnecessary.")));
}
```

You can, of course, insert additional tokens in the arguments as needed.

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
                    ($tup_tys)
                    Default::default()
                ),
            )*
        )
    };
}
```

> **<abbr title="Just for this example">JFTE</abbr>**: we *could* have simply used `$tup_tys::default()`.

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

## TT Bundling

```rust
macro_rules! call_a_or_b_on_tail {
    ((a: $a:expr, b: $b:expr), call a: $($tail:tt)*) => {
        $a(stringify!($($tail)*))
    };

    ((a: $a:expr, b: $b:expr), call b: $($tail:tt)*) => {
        $b(stringify!($($tail)*))
    };

    ($ab:tt, $_skip:tt $($tail:tt)*) => {
        call_a_or_b_on_tail!($ab, $($tail)*)
    };
}

fn compute_len(s: &str) -> Option<usize> {
    Some(s.len())
}

fn show_tail(s: &str) -> Option<usize> {
    println!("tail: {:?}", s);
    None
}

fn main() {
    assert_eq!(
        call_a_or_b_on_tail!(
            (a: compute_len, b: show_tail),
            the recursive part that skips over all these
            tokens doesn't much care whether we will call a
            or call b: only the terminal rules care.
        ),
        None
    );
    assert_eq!(
        call_a_or_b_on_tail!(
            (a: compute_len, b: show_tail),
            and now, to justify the existence of two paths
            we will also call a: its input should somehow
            be self-referential, so let's make it return
            some ninety one!
        ),
        Some(91)
    );
}
```

In particularly complex recursive macros, a large number of arguments may be needed in order to carry identifiers and expressions to successive layers.  However, depending on the implementation there may be many intermediate layers which need to forward these arguments, but do not need to *use* them.

As such, it can be very useful to bundle all such arguments together into a single TT by placing them in a group.  This allows layers which do not need to use the arguments to simply capture and substitute a single `tt`, rather than having to exactly capture and substitute the entire argument group.

The example above bundles the `$a` and `$b` expressions into a group which can then be forwarded as a single `tt` by the recursive rule.  This group is then destructured by the terminal rules to access the expressions.

# Provisional

This section is for patterns or techniques which are of dubious value, or which might be *too* niche for inclusion.

## Abacus Counters

> **Provisional**: needs a more compelling example.  Although an important part of the `Ook!` macro, matching nested groups that are *not* denoted by Rust groups is sufficiently unusual that it may not merit inclusion.

> **Note**: this section assumes understanding of [push-down accumulation](#push-down-accumulation) and [incremental TT munchers](#incremental-tt-munchers).

```rust
macro_rules! abacus {
    ((- $($moves:tt)*) -> (+ $($count:tt)*)) => {
        abacus!(($($moves)*) -> ($($count)*))
    };
    ((- $($moves:tt)*) -> ($($count:tt)*)) => {
        abacus!(($($moves)*) -> (- $($count)*))
    };
    ((+ $($moves:tt)*) -> (- $($count:tt)*)) => {
        abacus!(($($moves)*) -> ($($count)*))
    };
    ((+ $($moves:tt)*) -> ($($count:tt)*)) => {
        abacus!(($($moves)*) -> (+ $($count)*))
    };

    // Check if the final result is zero.
    (() -> ()) => { true };
    (() -> ($($count:tt)+)) => { false };
}

fn main() {
    let equals_zero = abacus!((++-+-+++--++---++----+) -> ());
    assert_eq!(equals_zero, true);
}
```

This technique can be used in cases where you need to keep track of a varying counter that starts at or near zero, and must support the following operations:

* Increment by one.
* Decrement by one.
* Compare to zero (or any other fixed, finite value).

A value of *n* is represented by *n* instances of a specific token stored in a group.  Modifications are done using recursion and [push-down accumulation](#push-down-accumulation).  Assuming the token used is `x`, the operations above are implemented as follows:

* Increment by one: match `($($count:tt)*)`, substitute `(x $($count)*)`.
* Decrement by one: match `(x $($count:tt)*)`, substitute `($($count)*)`.
* Compare to zero: match `()`.
* Compare to one: match `(x)`.
* Compare to two: match `(x x)`.
* *(and so on...)*

In this way, operations on the counter are like flicking tokens back and forth like an abacus.[^abacus]

[^abacus]:
    This desperately thin reasoning conceals the *real* reason for this name: to avoid having *yet another* thing with "token" in the name.  Talk to your writer about avoiding [semantic satiation](https://en.wikipedia.org/wiki/Semantic_satiation) today!

    In fairness, it could *also* have been called ["unary counting"](https://en.wikipedia.org/wiki/Unary_numeral_system).

In cases where you want to represent negative values, *-n* can be represented as *n* instances of a *different* token.  In the example given above, *+n* is stored as *n* `+` tokens, and *-m* is stored as *m* `-` tokens.

In this case, the operations become slightly more complicated; increment and decrement effectively reverse their usual meanings when the counter is negative.  To whit given `+` and `-` for the positive and negative tokens respectively, the operations change to:

* Increment by one:
  * match `()`, substitute `(+)`.
  * match `(- $($count:tt)*)`, substitute `($($count)*)`.
  * match `($($count:tt)+)`, substitute `(+ $($count)+)`.
* Decrement by one:
  * match `()`, substitute `(-)`.
  * match `(+ $($count:tt)*)`, substitute `($($count)*)`.
  * match `($($count:tt)+)`, substitute `(- $($count)+)`.
* Compare to 0: match `()`.
* Compare to +1: match `(+)`.
* Compare to -1: match `(-)`.
* Compare to +2: match `(++)`.
* Compare to -2: match `(--)`.
* *(and so on...)*

Note that the example at the top combines some of the rules together (for example, it combines increment on `()` and `($($count:tt)+)` into an increment on `($($count:tt)*)`).

If you want to extract the actual *value* of the counter, this can be done using a regular [counter macro](../blk/README.html#counting).  For the example above, the terminal rules can be replaced with the following:

```rust
macro_rules! abacus {
    // ...

    // This extracts the counter as an integer expression.
    (() -> ()) => {0};
    (() -> (- $($count:tt)*)) => {
        {(-1i32) $(- replace_expr!($count 1i32))*}
    };
    (() -> (+ $($count:tt)*)) => {
        {(1i32) $(+ replace_expr!($count 1i32))*}
    };
}

macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {$sub};
}
```

> **<abbr title="Just for this example">JFTE</abbr>**: strictly speaking, the above formulation of `abacus!` is needlessly complex.  It can be implemented much more efficiently using repetition, provided you *do not* need to match against the counter's value in a macro:
>
> ```rust
> macro_rules! abacus {
>     (-) => {-1};
>     (+) => {1};
>     ($($moves:tt)*) => {
>         0 $(+ abacus!($moves))*
>     }
> }
> ```
