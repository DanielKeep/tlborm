% Internal rules

```rust
#[macro_export]
macro_rules! foo {
    (@as_expr $e:expr) => {$e};

    ($($tts:tt)*) => {
        foo!(@as_expr $($tts)*)
    };
}
# 
# fn main() {
#     assert_eq!(foo!(42), 42);
# }
```

Because macros do not interact with regular item privacy or lookup, any public macro *must* bring with it all other macros that it depends on.  This can lead to pollution of the global macro namespace, or even conflicts with macros from other crates.  It may also cause confusion to users who attempt to *selectively* import macros: they must transitively import *all* macros, including ones that may not be publicly documented.

A good solution is to conceal what would otherwise be other public macros *inside* the macro being exported.  The above example shows how the common `as_expr!` macro could be moved *into* the publicly exported macro that is using it.

The reason for using `@` is that, as of Rust 1.2, the `@` token is *not* used in prefix position; as such, it cannot conflict with anything.  Other symbols or unique prefixes may be used as desired, but use of `@` has started to become widespread, so using it may aid readers in understanding your code.

> **Note**: the `@` token was previously used in prefix position to denote a garbage-collected pointer, back when the language used sigils to denote pointer types.  Its only *current* purpose is for binding names to patterns.  For this, however, it is used as an *infix* operator, and thus does not conflict with its use here.

Additionally, internal rules will often come *before* any "bare" rules, to avoid issues with `macro_rules!` incorrectly attempting to parse an internal invocation as something it cannot possibly be, such as an expression.

If exporting at least one internal macro is unavoidable (*e.g.* you have many macros that depend on a common set of utility rules), you can use this pattern to combine *all* internal macros into a single uber-macro.

```ignore
macro_rules! crate_name_util {
    (@as_expr $e:expr) => {$e};
    (@as_item $i:item) => {$i};
    (@count_tts) => {0usize};
    // ...
}
```
