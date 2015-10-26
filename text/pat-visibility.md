% Visibility

Matching and substituting visibility can be tricky in Rust, due to the absence of any kind of `vis` matcher.

## Matching and Ignoring

Depending on context, this can be done using a repetition:

```rust
macro_rules! struct_name {
    ($(pub)* struct $name:ident $($rest:tt)*) => { stringify!($name) };
}
# 
# fn main() {
#     assert_eq!(struct_name!(pub struct Jim;), "Jim");
# }
```

The above example will match `struct` items that are private or public.  Or `pub pub` (very public), or even `pub pub pub pub` (really very quite public).  The best defense against this is to simply hope that people using the macro are not excessively silly.

## Matching and Substituting

Because you cannot bind a repetition in and of itself to a variable, there is no way to store the contents of `$(pub)*` such that it can be substituted.  As a result, multiple rules are needed.

```rust
macro_rules! newtype_new {
    (struct $name:ident($t:ty);) => { newtype_new! { () struct $name($t); } };
    (pub struct $name:ident($t:ty);) => { newtype_new! { (pub) struct $name($t); } };
    
    (($($vis:tt)*) struct $name:ident($t:ty);) => {
        as_item! {
            impl $name {
                $($vis)* fn new(value: $t) -> Self {
                    $name(value)
                }
            }
        }
    };
}

macro_rules! as_item { ($i:item) => {$i} }
# 
# #[derive(Debug, Eq, PartialEq)]
# struct Dummy(i32);
# 
# newtype_new! { struct Dummy(i32); }
# 
# fn main() {
#     assert_eq!(Dummy::new(42), Dummy(42));
# }
```

> **See also**: [AST Coercion].

In this case, we are using the ability to match an arbitrary sequence of tokens inside a group to match either `()` or `(pub)`, then substitute the contents into the output.  Because the parser will not expect a `tt` repetition expansion in this position, we need to use [AST coercion] to get the expansion to parse correctly.

[AST Coercion]: blk-ast-coercion.html
