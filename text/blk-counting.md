% Counting

## Repetition with replacement

Counting things in a macro is a surprisingly tricky task.  The simplest way is to use replacement with a repetition match.

```rust
macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {$sub};
}

macro_rules! count_tts {
    ($($tts:tt)*) => {0usize $(+ replace_expr!($tts 1usize))*};
}
# 
# fn main() {
#     assert_eq!(count_tts!(0 1 2), 3);
# }
```

This is a fine approach for smallish numbers, but will likely *crash the compiler* with inputs of around 500 or so tokens.  Consider that the output will look something like this:

```ignore
0usize + 1usize + /* ~500 `+ 1usize`s */ + 1usize
```

The compiler must parse this into an AST, which will produce what is effectively a perfectly unbalanced binary tree 500+ levels deep.

## Recursion

An older approach is to use recursion.

```rust
macro_rules! count_tts {
    () => {0usize};
    ($_head:tt $($tail:tt)*) => {1usize + count_tts!($($tail)*)};
}
# 
# fn main() {
#     assert_eq!(count_tts!(0 1 2), 3);
# }
```

> **Note**: As of `rustc` 1.2, the compiler has *grevious* performance problems when large numbers of integer literals of unknown type must undergo inference.  We are using explicitly `usize`-typed literals here to avoid that.
>
> If this is not suitable (such as when the type must be substitutable), you can help matters by using `as` (*e.g.* `0 as $ty`, `1 as $ty`, *etc.*).

This *works*, but will trivially exceed the recursion limit.  Unlike the repetition approach, you can extend the input size by matching multiple tokens at once.

```rust
macro_rules! count_tts {
    ($_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $_f:tt $_g:tt $_h:tt $_i:tt $_j:tt
     $_k:tt $_l:tt $_m:tt $_n:tt $_o:tt
     $_p:tt $_q:tt $_r:tt $_s:tt $_t:tt
     $($tail:tt)*)
        => {20usize + count_tts!($($tail)*)};
    ($_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $_f:tt $_g:tt $_h:tt $_i:tt $_j:tt
     $($tail:tt)*)
        => {10usize + count_tts!($($tail)*)};
    ($_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $($tail:tt)*)
        => {5usize + count_tts!($($tail)*)};
    ($_a:tt
     $($tail:tt)*)
        => {1usize + count_tts!($($tail)*)};
    () => {0usize};
}

fn main() {
    assert_eq!(700, count_tts!(
        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
        
        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
        
        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
        
        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
        
        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
        
        // Repetition breaks somewhere after this
        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,

        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
    ));
}
```

This particular formulation will work up to ~1,200 tokens.

## Slice length

A third approach is to help the compiler construct a shallow AST that won't lead to a stack overflow.  This can be done by constructing an array literal and calling the `len` method.

```rust
macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {$sub};
}

macro_rules! count_tts {
    ($($tts:tt)*) => {<[()]>::len(&[$(replace_expr!($tts ())),*])};
}
# 
# fn main() {
#     assert_eq!(count_tts!(0 1 2), 3);
# }
```

This has been tested to work up to 10,000 tokens, and can probably go much higher.  The *downside* is that as of Rust 1.2, this *cannot* be used to produce a constant expression.  Although the result can be optimised to a simple constant (in debug builds it compiles down to a load from memory), it still cannot be used in constant positions (such as the value of `const`s, or a fixed array's size).

However, if a non-constant count is acceptable, this is very much the preferred method.

## Enum counting

This approach can be used where you need to count a set of mutually distinct identifiers.  Additionally, the result of this approach is usable as a constant.

```rust
macro_rules! count_idents {
    ($($idents:ident),* $(,)*) => {
        {
            #[allow(dead_code, non_camel_case_types)]
            enum Idents { $($idents,)* __CountIdentsLast }
            const COUNT: u32 = Idents::__CountIdentsLast as u32;
            COUNT
        }
    };
}
# 
# fn main() {
#     const COUNT: u32 = count_idents!(A, B, C);
#     assert_eq!(COUNT, 3);
# }
```

This method does have two drawbacks.  First, as implied above, it can *only* count valid identifiers (which are also not keywords), and it does not allow those identifiers to repeat.

Secondly, this approach is *not* hygienic, meaning that if whatever identifier you use in place of `__CountIdentsLast` is provided as input, the macro will fail due to the duplicate variants in the `enum`.
