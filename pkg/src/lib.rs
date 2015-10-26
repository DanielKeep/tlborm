/*!
This crate implements various macros detailed in [The Little Book of Rust Macros](https://danielkeep.github.io/tlborm/).

If you use selective macro importing, you should make sure to *always* use the `tlborm_util` macro, as most macros in this crate depend on it being present.
*/

/**
Forces the parser to interpret this macro's argument as an expression, even in the presence of `tt` substitutions.

See [TLBoRM: AST Coercion](https://danielkeep.github.io/tlborm/book/blk-ast-coercion.html).

## Examples

```rust
# #[macro_use] extern crate tlborm;
# fn main() {
assert_eq!(as_expr!(42), 42);

macro_rules! conceal_as_tts {
    // The `tt` substitution will break regular parsing.
    (passthru, $($tts:tt)*) => {$($tts)*};
    ($callback:ident, $($tts:tt)*) => {$callback!($($tts)*)};
}

assert_eq!(conceal_as_tts!(as_expr,  2 * (3 + 4)), 14);
# }
```

The following will *not* compile:

<!-- NO-FAILING-TESTS -->

```ignore
# #[macro_use(as_expr, tlborm_util)] extern crate tlborm;
# fn main() {
# macro_rules! conceal_as_tts {
#     (passthru, $($tts:tt)*) => {$($tts)*};
#     ($callback:ident, $($tts:tt)*) => {$callback!($($tts)*)};
# }
assert_eq!(conceal_as_tts!(passthru, 2 * (3 + 4)), 14);
# }
```
*/
#[macro_export]
macro_rules! as_expr { ($e:expr) => {$e} }

/**
Forces the parser to interpret this macro's argument as an item, even in the presence of `tt` substitutions.

See [TLBoRM: AST Coercion](https://danielkeep.github.io/tlborm/book/blk-ast-coercion.html).

## Examples

```rust
# #[macro_use(as_item, tlborm_util)] extern crate tlborm;
macro_rules! enoom {
    ($name:ident { $($body:tt)* }) => {
        as_item! {
            // The `tt` substitution breaks regular parsing.
            enum $name { $($body)* }
        }
    }
}

enoom! {
    Dash { Solid, Dash, Dot }
}
# fn main() {}
```
*/
#[macro_export]
macro_rules! as_item { ($i:item) => {$i} }

/**
Forces the parser to interpret this macro's argument as a pattern, even in the presence of `tt` substitutions.

See [TLBoRM: AST Coercion](https://danielkeep.github.io/tlborm/book/blk-ast-coercion.html).

## Examples

```rust
# #[macro_use(as_pat, tlborm_util)] extern crate tlborm;
# fn main() {
macro_rules! tuple_pat {
    ($($names:tt)*) => {
        // The `tt` substitution breaks regular parsing.
        as_pat!( ( $($names,)* ) )
    }
}

match (1, 2, 3) {
    tuple_pat!(a b c) => assert_eq!((a, b, c), (1, 2, 3))
}
# }
```
*/
#[macro_export]
macro_rules! as_pat  { ($p:pat) =>  {$p} }

/**
Forces the parser to interpret this macro's argument as a statement, even in the presence of `tt` substitutions.

See [TLBoRM: AST Coercion](https://danielkeep.github.io/tlborm/book/blk-ast-coercion.html).

## Examples

```rust
# #[macro_use(as_stmt, tlborm_util)] extern crate tlborm;
# fn main() {
macro_rules! let_stmt {
    ($name:tt = $($init:tt)*) => {
        // The `tt` substitution breaks regular parsing.
        as_stmt!(let $name = $($init)*);
    }
}

let_stmt!(x = 42);
assert_eq!(x, 42);
# }
```
*/
#[macro_export]
macro_rules! as_stmt { ($s:stmt) => {$s} }

/**
Expands to the number of identifiers provided.  The expansion is suitable for use in a constant expression, and is of type `u32`.

The identifiers provided **must** be mutually unique; *i.e.* there cannot be any repeated identifiers.  In addition, the identifier `__CountIdentsLast` **must not** be used in the invocation.  This macro should be usable for even very large numbers of identifiers.

See [TLBoRM: Counting (Enum counting)](https://danielkeep.github.io/tlborm/book/blk-counting.html#enum-counting).

## Examples

```rust
# #[macro_use(count_idents_enum, tlborm_util)] extern crate tlborm;
# fn main() {
const NUM: u32 = count_idents_enum!(Silly swingers get your feeling under spell);
assert_eq!(NUM, 7);
# }
*/
#[macro_export]
macro_rules! count_idents_enum {
    ($($idents:ident)*) => {tlborm_util!(@count_idents_enum $($idents)*)};
}

/**
Expands to the number of token trees provided.  The expansion is suitable for use in a constant expression, and is of type `usize`.

This macro is limited to input of approximately 500 tokens, but efficiently expands in a single pass.  This makes it useful in recursion-limited contexts, or when you want fast expansion of small inputs.

See [TLBoRM: Counting (Repetition with replacement)](https://danielkeep.github.io/tlborm/book/blk-counting.html#repetition-with-replacement).

## Examples

```rust
# #[macro_use(count_tts_flat, tlborm_util)] extern crate tlborm;
# fn main() {
const NUM: usize = count_tts_flat!(Everybody's rhythm mad (and I love that rhythm too!));
assert_eq!(NUM, 5);
# }
*/
#[macro_export]
macro_rules! count_tts_flat {
    ($($tts:tt)*) => {tlborm_util!(@count_tts_flat $($tts)*)};
}

/**
Expands to the number of token trees provided.  The expansion is suitable for use in a constant expression, and is of type `usize`.

This macro is limited to input of approximately 1,200 tokens, but requires multiple recursive expansion passes.  This macro is useful when you need to count a large number of things *and* you need the result to be a compile-time constant.

See [TLBoRM: Counting (Recursion)](https://danielkeep.github.io/tlborm/book/blk-counting.html#recursion).

## Examples

```rust
# #[macro_use(count_tts_recur, tlborm_util)] extern crate tlborm;
# fn main() {
const NUM: usize = count_tts_recur!(De l'enfer au paradis!);
assert_eq!(NUM, 6);
# }
*/
#[macro_export]
macro_rules! count_tts_recur {
    ($($tts:tt)*) => {tlborm_util!(@count_tts_recur $($tts)*)};
}

/**
Expands to the number of token trees provided.  The expansion is **not** suitable for use in a constant expression, though it should be optimised to a simple integer constant in release builds.

This macro is has no practical limit (and has been tested to over 10,000 tokens).

See [TLBoRM: Counting (Slice length)](https://danielkeep.github.io/tlborm/book/blk-counting.html#slice-length).

## Examples

```rust
# #[macro_use(count_tts_slice, tlborm_util)] extern crate tlborm;
# fn main() {
let num = count_tts_slice!(You have no idea how tedious this is! #examplesrhard);
assert_eq!(num, 11);
# }
*/
#[macro_export]
macro_rules! count_tts_slice {
    ($($tts:tt)*) => {tlborm_util!(@count_tts_slice $($tts)*)};
}

/**
Expands to an invocation of the `$callback` macro, with a list of the unitary variant names of the provided enum separated by commas.  The invocation's argument will be prefixed by the contents of `$arg`.

If `$arg` is of the form `{…}`, then the expansion will be parsed as one or more items.  If it is of the form `(…)`, the expansion will be parsed as an expression.

See [TLBoRM: Enum Parsing](https://danielkeep.github.io/tlborm/book/blk-enum-parsing.html).

## Examples

```rust
# #[macro_use(parse_unitary_variants, tlborm_util)] extern crate tlborm;
# fn main() {
macro_rules! variant_list {
    (sep: $sep:tt, ($($var:ident),*)) => {
        concat!($(stringify!($var), $sep,)*)
    }
}

const LIST: &'static str = parse_unitary_variants!(
    enum Currency { Trenni, Phiring, Ryut, FakeMarinne, Faram, SoManyCoins }
    => variant_list(sep: ", ", )
);
assert_eq!(LIST, "Trenni, Phiring, Ryut, FakeMarinne, Faram, SoManyCoins, ");
# }
*/
#[macro_export]
macro_rules! parse_unitary_variants {
    (
        enum $name:ident {$($body:tt)*} => $callback:ident $arg:tt
    ) => {
        tlborm_util! {
            @parse_unitary_variants
            enum $name {$($body)*} => $callback $arg
        }
    };
}

/**
Utility macro that takes a token tree and an expression, expanding to the expression.

This is typically used to replace elements of an arbitrary token sequence with some fixed expression.

See [TLBoRM: Repetition replacement](https://danielkeep.github.io/tlborm/book/pat-repetition-replacement.html).

## Examples

```rust
# #[macro_use(replace_expr, tlborm_util)] extern crate tlborm;
# fn main() {
macro_rules! tts_to_zeroes {
    ($($tts:tt)*) => {
        [$(replace_expr!($tts 0)),*]
    }
}

assert_eq!(tts_to_zeroes!(pub const unsafe impl), [0, 0, 0, 0]);
# }
```
*/
#[macro_export]
macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {tlborm_util!(@replace_expr $_t $sub)};
}

#[doc(hidden)]
#[macro_export]
macro_rules! tlborm_util {
    (@as_expr $e:expr) => {$e};

    (@as_item $($i:item)+) => {$($i)+};

    (@as_pat $p:pat) => {$p};

    (@as_stmt $s:stmt) => {$s};

    (@count_idents_enum $($idents:ident)*) => {
        {
            #[allow(dead_code, non_camel_case_types)]
            enum Idents { $($idents,)* __CountIdentsLast }
            const COUNT: u32 = Idents::__CountIdentsLast as u32;
            COUNT
        }
    };

    (@count_tts_flat $($tts:tt)*) => {0usize $(+ tlborm_util!(@replace_expr $tts 1usize))*};

    (@count_tts_recur
     $_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $_f:tt $_g:tt $_h:tt $_i:tt $_j:tt
     $_k:tt $_l:tt $_m:tt $_n:tt $_o:tt
     $_p:tt $_q:tt $_r:tt $_s:tt $_t:tt
     $($tail:tt)*)
        => {20usize + tlborm_util!(@count_tts_recur $($tail)*)};
    (@count_tts_recur
     $_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $_f:tt $_g:tt $_h:tt $_i:tt $_j:tt
     $($tail:tt)*)
        => {10usize + tlborm_util!(@count_tts_recur $($tail)*)};
    (@count_tts_recur
     $_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $($tail:tt)*)
        => {5usize + tlborm_util!(@count_tts_recur $($tail)*)};
    (@count_tts_recur
     $_a:tt
     $($tail:tt)*)
        => {1usize + tlborm_util!(@count_tts_recur $($tail)*)};
    (@count_tts_recur) => {0usize};

    (@count_tts_slice $($tts:tt)*)
        => {<[()]>::len(&[$(tlborm_util!(@replace_expr $tts ())),*])};

    (@replace_expr $_t:tt $sub:expr) => {$sub};

    // ========================================================================
    // @parse_unitary_variants
    (
        @parse_unitary_variants
        enum $name:ident {$($body:tt)*} => $callback:ident $arg:tt
    ) => {
        tlborm_util! {
            @collect_unitary_variants
            ($callback $arg), ($($body)*,) -> ()
        }
    };
    
    // ========================================================================
    // @collect_unitary_variants
    // Exit rules.
    (
        @collect_unitary_variants ($callback:ident ( $($args:tt)* )),
        ($(,)*) -> ($($var_names:ident,)*)
    ) => {
        tlborm_util! {
            @as_expr
            $callback!{ $($args)* ($($var_names),*) }
        }
    };

    (
        @collect_unitary_variants ($callback:ident { $($args:tt)* }),
        ($(,)*) -> ($($var_names:ident,)*)
    ) => {
        tlborm_util! {
            @as_item
            $callback!{ $($args)* ($($var_names),*) }
        }
    };

    // Consume an attribute.
    (
        @collect_unitary_variants $fixed:tt,
        (#[$_attr:meta] $($tail:tt)*) -> ($($var_names:tt)*)
    ) => {
        tlborm_util! {
            @collect_unitary_variants $fixed,
            ($($tail)*) -> ($($var_names)*)
        }
    };

    // Handle a variant, optionally with an with initialiser.
    (
        @collect_unitary_variants $fixed:tt,
        ($var:ident $(= $_val:expr)*, $($tail:tt)*) -> ($($var_names:tt)*)
    ) => {
        tlborm_util! {
            @collect_unitary_variants $fixed,
            ($($tail)*) -> ($($var_names)* $var,)
        }
    };

    // Abort on variant with a payload.
    (
        @collect_unitary_variants $fixed:tt,
        ($var:ident $_struct:tt, $($tail:tt)*) -> ($($var_names:tt)*)
    ) => {
        const _error: () = "cannot parse unitary variants from enum with non-unitary variants";
    };
}
