% macro_rules!

With all that in mind, we can introduce `macro_rules!` itself.  As noted previously, `macro_rules!` is *itself* a syntax extension, meaning it is *technically* not part of the Rust syntax.  It uses the following form:

```ignore
macro_rules! $name {
    $rule0 ;
    $rule1 ;
    // …
    $ruleN ;
}
```

There must be *at least* one rule, and you can omit the semicolon after the last rule.

Each "`rule`" looks like so:

```ignore
    ($pattern) => {$expansion}
```

Actually, the parens and braces can be any kind of group, but parens around the pattern and braces around the expansion are somewhat conventional.

If you are wondering, the `macro_rules!` invocation expands to... *nothing*.  At least, nothing that appears in the AST; rather, it manipulates compiler-internal structures to register the macro.  As such, you can *technically* use `macro_rules!` in any position where an empty expansion is valid.

## Matching

When a macro is invoked, the `macro_rules!` interpreter goes through the rules one by one, in lexical order.  For each rule, it tries to match the contents of the input token tree against that rule's `pattern`.  A pattern must match the *entirety* of the input to be considered a match.

If the input matches the pattern, the invocation is replaced by the `expansion`; otherwise, the next rule is tried.  If all rules fail to match, macro expansion fails with an error.

The simplest example is of an empty pattern:

```ignore
macro_rules! four {
    () => {1 + 3};
}
```

This matches if and only if the input is also empty (*i.e.* `four!()`, `four![]` or `four!{}`).

Note that the specific grouping tokens you use when you invoke the macro *are not* matched.  That is, you can invoke the above macro as `four![]` and it will still match.  Only the *contents* of the input token tree are considered.

Patterns can also contain literal token trees, which must be matched exactly.  This is done by simply writing the token trees normally.  For example, to match the sequence `4 fn ['spang "whammo"] @_@`, you would use:

```ignore
macro_rules! gibberish {
    (4 fn ['spang "whammo"] @_@) => {...};
}
```

You can use any token tree that you can write.

## Captures

Patterns can also contain captures.  These allow input to be matched based on some general grammar category, with the result captured to a variable which can then be substituted into the output.

Captures are written as a dollar (`$`) followed by an identifier, a colon (`:`), and finally the kind of capture, which must be one of the following:

* `item`: an item, like a function, struct, module, etc.
* `block`: a block (i.e. a block of statements and/or an expression, surrounded by braces)
* `stmt`: a statement
* `pat`: a pattern
* `expr`: an expression
* `ty`: a type
* `ident`: an identifier
* `path`: a path (e.g. `foo`, `::std::mem::replace`, `transmute::<_, int>`, …)
* `meta`: a meta item; the things that go inside `#[...]` and `#![...]` attributes
* `tt`: a single token tree
* `vis`: a visibility modifier such as `pub` or simply nothing

For example, here is a macro which captures its input as an expression:

```ignore
macro_rules! one_expression {
    ($e:expr) => {...};
}
```

These captures leverage the Rust compiler's parser, ensuring that they are always "correct".  An `expr` capture will *always* capture a complete, valid expression for the version of Rust being compiled.

You can mix literal token trees and captures, within limits (explained below).

A capture `$name:kind` can be substituted into the expansion by writing `$name`.  For example:

```ignore
macro_rules! times_five {
    ($e:expr) => {5 * $e};
}
```

Much like macro expansion, captures are substituted as complete AST nodes.  This means that no matter what sequence of tokens is captured by `$e`, it will be interpreted as a single, complete expression.

You can also have multiple captures in a single pattern:

```ignore
macro_rules! multiply_add {
    ($a:expr, $b:expr, $c:expr) => {$a * ($b + $c)};
}
```

## Repetitions

Patterns can contain repetitions.  These allow a sequence of tokens to be matched.  These have the general form `$ ( ... ) sep rep`.

* `$` is a literal dollar token.
* `( ... )` is the paren-grouped pattern being repeated.
* `sep` is an *optional* separator token.  Common examples are `,`, and `;`.
* `rep` is the *required* repeat control.  Currently, this can be *either* `*` (indicating zero or more repeats) or `+` (indicating one or more repeats).  You cannot write "zero or one" or any other more specific counts or ranges.

Repetitions can contain any other valid pattern, including literal token trees, captures, and other repetitions.

Repetitions use the same syntax in the expansion.

For example, below is a macro which formats each element as a string.  It matches zero or more comma-separated expressions and expands to an expression that constructs a vector.

```rust
macro_rules! vec_strs {
    (
        // Start a repetition:
        $(
            // Each repeat must contain an expression...
            $element:expr
        )
        // ...separated by commas...
        ,
        // ...zero or more times.
        *
    ) => {
        // Enclose the expansion in a block so that we can use
        // multiple statements.
        {
            let mut v = Vec::new();

            // Start a repetition:
            $(
                // Each repeat will contain the following statement, with
                // $element replaced with the corresponding expression.
                v.push(format!("{}", $element));
            )*

            v
        }
    };
}
# 
# fn main() {
#     let s = vec_strs![1, "a", true, 3.14159f32];
#     assert_eq!(&*s, &["1", "a", "true", "3.14159"]);
# }
```
