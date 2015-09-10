% Macros

This chapter will introduce Rust's Macro-By-Example system: `macro_rules!`.  Rather than trying to cover it based on practical examples, it will instead attempt to give you a complete and thorough explanation of *how* the system works.  As such, this is intended for people who just want the system as a whole explained, rather than be guided through it.

In terms of learning resources, there is also the [Macros chapter of the Rust Book](http://doc.rust-lang.org/book/macros.html) which is a more approachable, high-level explanation, and [A Practical Intro to Macros in Rust 1.0](https://danielkeep.github.io/practical-intro-to-macros.html) which is a guided implementation of a single macro.

## Rust Source Analysis

> **TODO**: How is source text lexed and then parsed?

Before talking about *macros*, it is worthwhile discussing the general mechanism they are built on: *syntax extensions*.  To do *that*, we must discuss how Rust source is processed by the compiler.

The first stage of compilation for a Rust program is tokenisation.  This is where the source text is transformed into a sequence of tokens (*i.e.* indivisible lexical units; the programming language equivalent of "words").  Rust has various kinds of tokens, such as:

* Identifiers: `foo`, `Bambous`, `self`, `we_can_dance`, `LaCaravane`, …
* Integers: `42`, `72u32`, `0_______0`, …
* Keywords: `_`, `fn`, `self`, `match`, `yield`, `macro`, …
* Lifetimes: `'a`, `'b`, `'a_rare_long_lifetime_name`, …
* Strings: `""`, `"Leicester"`, `r##"venezuelan beaver"##`, …
* Symbols: `[`, `:`, `::`, `->`, `@`, `<-`, …

…among others.  There are some things to note about the above: first, `self` is both an identifier *and* a keyword.  In almost all cases, `self` is a keyword, but it *is* possible for it to *become* a keyword, which will come up later (along with much cursing).  Secondly, the list of keywords includes some suspicious entries such as `yield` and `macro` that aren't *actually* in the language, but *are* parsed by the compiler—these are reserved for future use.  Third, the list of symbols *also* includes entries that aren't used by the language.  In the case of `@` and `<-`, both are vestigial: they were removed from the language, but not from the lexer.  As a final point, note that `::` is a distinct token; it is not simply two adjacent `:` tokens.  The same is true of all multi-character symbol tokens in Rust, as of Rust 1.2.

As a point of comparison, it is at *this* stage that some languages have their macro layer, though Rust does *not*.  For example, C/C++ macros are *effectively* processed at this point.[^lies-damn-lies-cpp]  This is why the following code works:[^cpp-it-seemed-like-a-good-idea-at-the-time]

[^lies-damn-lies-cpp]: In fact, the C preprocessor uses a different lexical structure to C itself, but the distinction is *broadly* irrelevant.

[^cpp-it-seemed-like-a-good-idea-at-the-time]: *Whether* it should work is an entirely *different* question.

```c
#define SUB void
#define BEGIN {
#define END }

SUB main() BEGIN
    printf("Oh, the horror!\n");
END
```

The next stage is parsing, where the stream of tokens is turned into an Abstract Syntax Tree (AST).  This involves building up the syntactic structure of the program in memory.  For example, the token sequence `1 + 2` is transformed into the equivalent of:

```text
┌─────────┐   ┌─────────┐
│ BinOp   │ ┌╴│ LitInt  │
│ op: Add │ │ │ val: 1  │
│ lhs: ◌  │╶┘ └─────────┘
│ rhs: ◌  │╶┐ ┌─────────┐
└─────────┘ └╴│ LitInt  │
              │ val: 2  │
              └─────────┘
```

The AST contains the structure of the *entire* program, though it is based on purely *lexical* information.  For example, although the compiler may know that a particular expression is referring to a variable called "`a`", at this stage, it has *no way* of knowing what "`a`" is, or even *where* it comes from.

It is *after* the AST has been constructed that macros are processed.  However, before we can discuss that, we have to talk about token trees.

## Token trees

> **TODO**: What is a token, what is a token tree.  Mention NTs.

Token trees are somewhere between tokens and the AST.  Firstly, *almost* all tokens are also token trees; more specifically, they are *leaves*.  There is one other kind of thing that can be a token tree leaf, but we will come back to that later.

The only basic tokens that are *not* leaves are the "grouping" tokens: `(...)`, `[...]`, and `{...}`.  These three are the *interior nodes* of token trees, and what give them their structure.  To give a concrete example, this sequence of tokens:

```rust
a + b + (c + d[0]) + e
```

would be parsed into the following token trees:

```text
«a» «+» «b» «+» «(   )» «+» «e»
          ╭────────┴──────────╮
           «c» «+» «d» «[   ]»
                        ╭─┴─╮
                         «0»
```

Note that this has *no relationship* to the AST the expression would produce; instead of a single root node, there are *nine* token trees at the root level.  For reference, the AST would be:

```text
              ┌─────────┐
              │ BinOp   │
              │ op: Add │
            ┌╴│ lhs: ◌  │
┌─────────┐ │ │ rhs: ◌  │╶┐ ┌─────────┐
│ Var     │╶┘ └─────────┘ └╴│ BinOp   │
│ name: a │                 │ op: Add │
└─────────┘               ┌╴│ lhs: ◌  │
              ┌─────────┐ │ │ rhs: ◌  │╶┐ ┌─────────┐
              │ Var     │╶┘ └─────────┘ └╴│ BinOp   │
              │ name: b │                 │ op: Add │
              └─────────┘               ┌╴│ lhs: ◌  │
                            ┌─────────┐ │ │ rhs: ◌  │╶┐ ┌─────────┐
                            │ BinOp   │╶┘ └─────────┘ └╴│ Var     │
                            │ op: Add │                 │ name: e │
                          ┌╴│ lhs: ◌  │                 └─────────┘
              ┌─────────┐ │ │ rhs: ◌  │╶┐ ┌─────────┐
              │ Var     │╶┘ └─────────┘ └╴│ Index   │
              │ name: c │               ┌╴│ arr: ◌  │
              └─────────┘   ┌─────────┐ │ │ ind: ◌  │╶┐ ┌─────────┐
                            │ Var     │╶┘ └─────────┘ └╴│ LitInt  │
                            │ name: d │                 │ val: 0  │
                            └─────────┘                 └─────────┘
```

It is important to understand the distinction between the AST and token trees.  When writing macros, you have to deal with *both* as distinct things.

One other aspect of this to note: it is *impossible* to have an unpaired paren, bracket or brace; nor is it possible to have incorrectly nested groups in a token tree.

## Macros in the AST

> **TODO**: The `ident ! $tt` syntax, that it might be a syntax extension (not a macro).

> **TODO**: Talk about the various classes of syntax extension, and how they are processed.  In particular: prior to names and types are resolved.  They *aren't* items and aren't in the same namespace as anything else.  List major limitations.

As previously mentioned, macro processing in Rust happens *after* the construction of the AST.  As such, the syntax used to invoke a macro *must* be a proper part of the language's syntax.  In fact, there are several "syntax extension" forms which are part of Rust's syntax.  Specifically, the following forms (by way of examples):

* `# [ $arg ]`; *e.g.* `#[derive(Clone)]`, `#[no_mangle]`, …
* `# ! [ $arg ]`; *e.g.* `#![allow(dead_code)]`, `#![crate_name="blang"]`, …
* `$name ! $arg`; *e.g.* `println!("Hi!")`, `concat!("a", "b")`, …
* `$name ! $arg0 $arg1`; *e.g.* `macro_rules! dummy { () => {}; }`.

The first two are "attributes", and are shared between both language-specific constructs (such as `#[repr(C)]` which is used to request a C-compatible ABI for user-defined types) and syntax extensions (such as `#[derive(Clone)]`).  There is currently no way to define a macro that uses these forms.

The third is the one of interest to us: it is the form available for use with macros.  It is *also* used by various syntax extensions and compiler plugins.  For example, whilst `format!` is a macro, `format_args!` (which is used to *implement* `format!`) is *not*.

The fourth is essentially a variation which is *not* available to macros.  In fact, the only case where this form is used *at all* is with `macro_rules!` which, again we will come back to.

Disregarding all but the third form (`$name ! $arg`), the question becomes: how does the Rust parser know what `$arg` looks like for every possible macro?  The answer is that it doesn't *have to*.  Instead, the argument of a macro invocation is a *single* token tree.  More specifically, it is a single, *non-leaf* token tree; `(...)`, `[...]`, or `{...}`.  With that knowledge, it should become apparent how the parser can understand all of the following invocation forms:

```rust
bitflags! {
    flags Color: u8 {
        const RED    = 0b0001,
        const GREEN  = 0b0010,
        const BLUE   = 0b0100,
        const BRIGHT = 0b1000,
    }
}

lazy_static! {
    static ref FIB_100: u32 = {
        fn fib(a: u32) -> u32 {
            match a {
                0 => 0,
                1 => 1,
                a => fib(a-1) + fib(a-2)
            }
        }

        fib(100)
    };
}

fn main() {
    let colors = vec![RED, GREEN, BLUE];
    println!("Hello, World!");
}
```

Although the above macro invocations may *look* like they contain various kinds of Rust code, the parser simply sees a collection of meaningless token trees.  To make this clearer, we can replace all these syntactic "black boxes" with ⬚, leaving us with:

```text
bitflags! ⬚

lazy_static! ⬚

fn main() {
    let colors = vec! ⬚;
    println! ⬚;
}
```

Just to reiterate: the parser does not assume *anything* about ⬚; it remembers the tokens it contains, but doesn't try to *understand* them.

The important takeaways are:

* There are multiple kinds of syntax extension in Rust.  We will *only* be talking about macros defined by the `macro_rules!` construct.
* Just because you see something of the form `$name! $arg`, doesn't mean it's actually a macro; it might be another kind of syntax extension.
* The input to every macro is a single non-leaf token tree.
* Macros are parsed as *part* of the abstract syntax tree.

The last point is the most important, as it has *significant* implications.  Because macros are parsed into the AST, they can **only** appear in positions where they are explicitly supported.  Specifically macros can appear in place of the following:

* Patterns
* Statements
* Expressions
* Items
* `impl` Items

Some things *not* on this list:

* Identifiers
* Match arms
* Struct fields
* Types

There is absolutely, definitely *no way* to use macros in any position *not* on the first list.

## Expansion

> **TODO**: Macros are expanded from the outside in.  The expansion can contain macro invocations, and will be forced into one a limited set of syntax elements.  Note the recursion limit and that it can be raised.

Expansion is a relatively simple affair.  At some point *after* the construction of the AST, but before the compiler begins constructing its semantic understanding of the program, it will expand all macros.

This involves traversing the AST, locating macro invocations and replacing them with their expansion.  In the case of non-macro syntax extensions, *how* this happens is undefined.  That said, syntax extensions go through *exactly* the same process that macros do once their expansion is complete.

Once the compiler has run a syntax extension, it expects an opaque object as a result.  The compiler will convert this object into one of a limited set of syntax elements, based on context.  For example, if you invoke a macro at module scope, the compiler will demand the result turn itself into an AST node that represents an item.  If you invoke a macro in expression position, the compiler will demand the result turn itself into an expression AST node.

In fact, it can turn a syntax extension result into any of the following:

* an expression,
* a pattern,
* zero or more items,
* zero or more `impl` items, or
* zero or more statements.

In other words, *where* you can invoke a macro determines what its result will be interpreted as.

The compiler will take this AST node and completely replace the macro's invocation node with the output node.  *This is a structural operation*, not a textural one!

For example, consider the following:

```rust
let eight = 2 * four!();
```

We can visualise this partial AST as follows:

```text
┌─────────────┐
│ Let         │
│ name: eight │   ┌─────────┐
│ init: ◌     │╶─╴│ BinOp   │
└─────────────┘   │ op: Mul │
                ┌╴│ lhs: ◌  │
     ┌────────┐ │ │ rhs: ◌  │╶┐ ┌────────────┐
     │ LitInt │╶┘ └─────────┘ └╴│ Macro      │
     │ val: 2 │                 │ name: four │
     └────────┘                 │ body: ()   │
                                └────────────┘
```

From context, `four!()` *must* expand to an expression (the initialiser can *only* be an expression).  Thus, whatever the actual expansion is, it will be interpreted as a complete expression.  In this case, we will assume `four!` is defined such that it expands to the expression `1 + 3`.  As a result, expanding this invocation will result in the AST changing to:

```text
┌─────────────┐
│ Let         │
│ name: eight │   ┌─────────┐
│ init: ◌     │╶─╴│ BinOp   │
└─────────────┘   │ op: Mul │
                ┌╴│ lhs: ◌  │
     ┌────────┐ │ │ rhs: ◌  │╶┐ ┌─────────┐
     │ LitInt │╶┘ └─────────┘ └╴│ BinOp   │
     │ val: 2 │                 │ op: Add │
     └────────┘               ┌╴│ lhs: ◌  │
                   ┌────────┐ │ │ rhs: ◌  │╶┐ ┌────────┐
                   │ LitInt │╶┘ └─────────┘ └╴│ LitInt │
                   │ val: 1 │                 │ val: 3 │
                   └────────┘                 └────────┘
```

This can be written out like so:

```rust
let eight = 2 * (1 + 3);
```

Note that we added parens *despite* them not being in the expansion.  Remember that the compiler always treats the expansion of a macro as a complete AST node, **not** as a mere sequence of tokens.  To put it another way, even if you don't explicitly wrap a complex expression in parentheses, there is no way for the compiler to "misinterpret" the result, or change the order of evaluation.

It is important to understand that macro expansions are treated as AST nodes, as this design has two further implications:

* In addition to there being a limited number of invocation *positions*, macros can *only* expand to the kind of AST node the parser *expects* at that position.
* As a consequence of the above, macros *absolutely cannot* expand to incomplete or syntactically invalid constructs.

There are two further things to note about expansion.  The first is what happens when a syntax extension expands to something that contains *another* syntax extension invocation.  For example, consider an alternative definition of `four!`; what happens if it expands to `1 + three!()`?

```rust
let x = four!();
```

Expands to:

```rust
let x = 1 + three!();
```

This is resolved by the compiler checking the result of expansions for additional macro invocations, and expanding them.  Thus, a second expansion step turns the above into:

```rust
let x = 1 + 3;
```

The takeaway here is that expansion happens in "passes"; as many as is needed to completely expand all invocations.

Well, not *quite*.  In fact, the compiler imposes an upper limit on the number of such recursive passes it is willing to run before giving up.  This is known as the macro recursion limit and defaults to 32.  If the 32nd expansion contains a macro invocation, the compiler will abort with an error indicating that the recursion limit was exceeded.

This limit can be raised using the `#![recursion_limit="…"]` attribute, though it *must* be done crate-wide.  Generally, it is recommended to try and keep macros below this limit wherever possible.

# Macro-by-example

> **TODO**: Introduce `macro_rules!` and the basic syntax.  Perhaps also a reminder that recursion is handled *after* expansion, not *during*.

With all that in mind, we can introduce `macro_rules!` itself.  As noted previously, `macro_rules!` is *itself* a syntax extension, meaning it is *technically* not part of the Rust syntax.  It uses the following form:

```rust
macro_rules! $name {
    $rule0 ;
    $rule1 ;
    // …
    $ruleN ;
}
```

There must be *at least* one rule, and you can omit the semicolon after the last rule.

Where each "`rule`" looks like so:

```rust
    ($pattern) => {$expansion}
```

Actually, the parens and braces can be any kind of group, but parens around the pattern and braces around the expansion are somewhat conventional.

If you are wondering, the `macro_rules!` invocation expands to... *nothing*.  At least, nothing that appears in the AST; rather, it manipulates compiler-internal structures to register the macro.  As such, you can *technically* use `macro_rules!` in any position where an empty expansion is valid.

## Matching

> **TODO**: How MR handles an invocation.  First example should be literal token matching.

When a macro is invoked, the `macro_rules!` interpreter goes through the rules one by one, in lexical order.  For each rule, it tries to match the contents of the input token tree against that rule's `pattern`.  A pattern must match the *entirety* of the input to be considered a match.

If the input matches the pattern, the invocation is replaced by the `expansion`; otherwise, the next rule is tried.  If all rules fail to match, macro expansion fails with an error.

The simplest example is of an empty pattern:

```rust
macro_rules! four {
    () => {1 + 3};
}
```

This matches if and only if the input is also empty (*i.e.* `four!()`, `four![]` or `four!{}`).

Note that the specific grouping tokens you use when you invoke the macro *are not* matched.  That is, you can invoke the above macro as `four![]` and it will still match.  Only the *contents* of the input token tree are considered.

Patterns can also contain literal token trees, which must be matched exactly.  This is done by simply writing the token trees normally.  For example, to match the sequence `4 fn ['spang "whammo"] @_@`, you would use:

```rust
macro_rules! gibberish {
    (4 fn ['spang "whammo"] @_@) => {...};
}
```

You can use any token tree that you can write.

## Captures

> **TODO**: Second example: introduce straight captures and substitutions.  Full list of capture kinds.

Patterns can also contain captures.  These allow input to be matched based on some general grammar category, with the result captured to a variable which can then be substituted into the output.

Captures are written as a dollar (`$`) followed by an identifier, a colon (`:`), and finally the kind of capture, which must be one of the following:

* `item`: an item, like a function, struct, module, etc.
* `block`: a block (i.e. a block of statments and/or an expression, surrounded by braces)
* `stmt`: a statement
* `pat`: a pattern
* `expr`: an expression
* `ty`: a type
* `ident`: an identifier
* `path`: a path (e.g. `foo`, `::std::mem::replace`, `transmute::<_, int>`, …)
* `meta`: a meta item; the things that go inside `#[...]` and `#![...]` attributes
* `tt`: a single token tree

> **TODO**: Does `ident` prevent matching against literal idents?  Prove it.

For example, here is a macro which captures its input as an expression:

```rust
macro_rules! one_expression {
    ($e:expr) => {...};
}
```

These captures leverage the Rust compiler's parser, ensuring that they are always "correct".  An `expr` capture will *always* capture a complete, valid expression for the version of Rust being compiled.

You can mix literal token trees and captures, within limits (explained below).

A capture `$name:kind` can be substituted into the expansion by writing `$name`.  For example:

```rust
macro_rules! times_five {
    ($e:expr) => {5 * $e};
}
```

Much like macro expansion, captures are substituted as complete AST nodes.  This means that no matter what sequence of tokens is captured by `$e`, it will be interpreted as a single, complete expression.

## Repetitions

> **TODO**: Third example: add repetitions with and without captures.  Note that repetitions must have a consistent "depth", and can't be mixed.

Finally, patterns can contain repetitions.  These allow a sequence of tokens to be matched.  These have the general form `$ ( ... ) sep rep`.

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

# Details

## Captures and Expansion Redux

> **TODO**: Captures are unabortable.

Once the parser begins consuming tokens for a capture, *it cannot stop or backtrack*.  This means that the second rule of the following macro *cannot ever match*, no matter what input is provided:

```rust
macro_rules! dead_rule {
    ($e:expr) => { ... };
    ($i:ident +) => { ... };
}
```

Consider what happens if this macro is invoked as `dead_rule!(x+)`.  The interpreter will start at the first rule, and attempt to parse the input as an expression.  The first token (`x`) is valid as an expression.  The second token is *also* valid in an expression, forming a binary addition node.

At this point, given that there is no left-hand side of the addition, you might expect the parser to give up and try the next rule.  Instead, the parser will panic and abort the entire compilation, citing a syntax error.

As such, it is important in general that you write macro rules from most-specific to least-specific.

> **TODO**: Captures restrict what can come after (TY_FOLLOW).

To defend against future syntax changes altering the interpretation of macro input, `macro_rules!` restricts what can follow various captures.  The complete list, as of Rust 1.3 is as follows:

* `item`: anything.
* `block`: anything.
* `stmt`: `=>` `,` `;`
* `pat`: `=>` `,` `=` `if` `in`
* `expr`: `=>` `,` `;`
* `ty`: `,` `=>` `:` `=` `>` `;` `as`
* `ident`: anything.
* `path`: `,` `=>` `:` `=` `>` `;` `as`
* `meta`: anything.
* `tt`: anything.

Additionally, `macro_rules!` generally forbids a repetition to be followed by another repetition, even if the contents do not conflict.

> **TODO**: Substitutions create NTs (except for TTs), which *cannot* be destructured afterwards.  Good example are meta items.

One aspect of substitution that often surprises people is that substitution is *not* token-based, despite very much *looking* like it.  Here is a simple demonstration:

```rust
macro_rules! capture_expr_then_stringify {
    ($e:expr) => {
        stringify!($e)
    };
}

fn main() {
    println!("{:?}", stringify!(dummy(2 * (1 + (3)))));
    println!("{:?}", capture_expr_then_stringify!(dummy(2 * (1 + (3)))));
}
```

The output when run is:

```text
"dummy ( 2 * ( 1 + ( 3 ) ) )"
"dummy(2 * (1 + (3)))"
```

Note that *despite* having the same input, the output is different.  This is because the first invocation is stringifying a sequence of token trees, whereas the second is stringifying *an AST expression node*.

To visualise the difference another way, here is what the `stringify!` macro gets invoked with in the first case:

```text
«dummy» «(   )»
   ╭───────┴───────╮
    «2» «*» «(   )»
       ╭───────┴───────╮
        «1» «+» «(   )»
                 ╭─┴─╮
                  «3»
```

…and here is what it gets invoked with in the second case:

```text
« »
 │ ┌─────────────┐
 └╴│ Call        │
   │ fn: dummy   │   ┌─────────┐
   │ args: ◌     │╶─╴│ BinOp   │
   └─────────────┘   │ op: Mul │
                   ┌╴│ lhs: ◌  │
        ┌────────┐ │ │ rhs: ◌  │╶┐ ┌─────────┐
        │ LitInt │╶┘ └─────────┘ └╴│ BinOp   │
        │ val: 2 │                 │ op: Add │
        └────────┘               ┌╴│ lhs: ◌  │
                      ┌────────┐ │ │ rhs: ◌  │╶┐ ┌────────┐
                      │ LitInt │╶┘ └─────────┘ └╴│ LitInt │
                      │ val: 1 │                 │ val: 3 │
                      └────────┘                 └────────┘
```

As you can see, there is exactly *one* token tree, which contains the AST which was parsed from the input to the `capture_expr_then_stringify!` invocation.  Hence, what you see in the output is not the stringified tokens, it's the stringified *AST node*.

This has further implications.  Consider the following:

```rust
macro_rules! capture_then_match_tokens {
    ($e:expr) => {match_tokens!($e)};
}

macro_rules! match_tokens {
    ($a:tt + $b:tt) => {"got an addition"};
    (($i:ident)) => {"got an identifier"};
    ($($other:tt)*) => {"got something else"};
}

fn main() {
    println!("{}\n{}\n{}\n",
        match_tokens!((caravan)),
        match_tokens!(3 + 6),
        match_tokens!(5));
    println!("{}\n{}\n{}",
        capture_then_match_tokens!((caravan)),
        capture_then_match_tokens!(3 + 6),
        capture_then_match_tokens!(5));
}
```

The output is:

```text
got an identifier
got an addition
got something else

got something else
got something else
got something else
```

By parsing the input into an AST node, the substituted result becomes *un-destructible*; *i.e.* you cannot examine the contents or match against it ever again.

> **TODO**: Verify this is true *re.* `ident`.

The only way to avoid this is to capture using the `tt` or `ident` kinds.  Once you capture with anything else, the only thing you can do with the result from then on is substitute it directly into the output.

## Hygiene

> **TODO**: Show syntax context colouring.

Macros in Rust are *partially* hygienic.  Specifically, they are hygienic when it comes to most identifiers, but *not* when it comes to generic type parameters or lifetimes.

Hygiene works by attaching an invisible "syntax context" value to all identifiers.  When two identifiers are compared, *both* the identifiers' textural names *and* syntax contexts must be identical for the two to be considered equal.

To illustrate this, consider the following code:

<pre class="rust rust-example-rendered"><span class="synctx-0"><span class="macro">macro_rules</span><span class="macro">!</span> <span class="ident">using_a</span> {
    (<span class="macro-nonterminal">$</span><span class="macro-nonterminal">e</span>:<span class="ident">expr</span>) <span class="op">=&gt;</span> {
        {
            <span class="kw">let</span> <span class="ident">a</span> <span class="op">=</span> <span class="number">42</span>;
            <span class="macro-nonterminal">$</span><span class="macro-nonterminal">e</span>
        }
    }
}

<span class="kw">let</span> <span class="ident">four</span> <span class="op">=</span> <span class="macro">using_a</span><span class="macro">!</span>(<span class="ident">a</span> <span class="op">/</span> <span class="number">10</span>);</span></pre>

We will use the background colour to denote the syntax context.  Now, let's expand the macro invocation:

<pre class="rust rust-example-rendered"><span class="synctx-0"><span class="kw">let</span> <span class="ident">four</span> <span class="op">=</span> </span><span class="synctx-1">{
    <span class="kw">let</span> <span class="ident">a</span> <span class="op">=</span> <span class="number">42</span>;
    </span><span class="synctx-0"><span class="ident">a</span> <span class="op">/</span> <span class="number">10</span></span><span class="synctx-1">
}</span><span class="synctx-0">;</span></pre>

First, recall that `macro_rules!` invocations effectively *disappear* during expansion.

Second, if you attempt to compile this code, the compiler will respond with something along the following lines:

```text
<anon>:11:21: 11:22 error: unresolved name `a`
<anon>:11 let four = using_a!(a / 10);
```

Note that the background colour (*i.e.* syntax context) for the expanded macro *changes* as part of expansion.  Each macro expansion is given a new, unique syntax context for its contents.  As a result, there are *two different `a`s* in the expanded code: one in the first syntax context, the second in the other.  In other words, <code><span class="synctx-0">a</span></code> is not the same identifier as <code><span class="synctx-1">a</span></code>, however similar they may appear.

That said, tokens that were substituted *into* the expanded output *retain* their original syntax context (by virtue of having been provided to the macro as opposed to being part of the macro itself).  Thus, the solution is to modify the macro as follows:

<pre class="rust rust-example-rendered"><span class="synctx-0"><span class="macro">macro_rules</span><span class="macro">!</span> <span class="ident">using_a</span> {
    (<span class="macro-nonterminal">$</span><span class="macro-nonterminal">a</span>:<span class="ident">ident</span>, <span class="macro-nonterminal">$</span><span class="macro-nonterminal">e</span>:<span class="ident">expr</span>) <span class="op">=&gt;</span> {
        {
            <span class="kw">let</span> <span class="macro-nonterminal">$</span><span class="macro-nonterminal">a</span> <span class="op">=</span> <span class="number">42</span>;
            <span class="macro-nonterminal">$</span><span class="macro-nonterminal">e</span>
        }
    }
}

<span class="kw">let</span> <span class="ident">four</span> <span class="op">=</span> <span class="macro">using_a</span><span class="macro">!</span>(<span class="ident">a</span>, <span class="ident">a</span> <span class="op">/</span> <span class="number">10</span>);</span></pre>

Which, upon expansion becomes:

<pre class="rust rust-example-rendered"><span class="synctx-0"><span class="kw">let</span> <span class="ident">four</span> <span class="op">=</span> </span><span class="synctx-1">{
    <span class="kw">let</span> </span><span class="synctx-0"><span class="ident">a</span></span><span class="synctx-1"> <span class="op">=</span> <span class="number">42</span>;
    </span><span class="synctx-0"><span class="ident">a</span> <span class="op">/</span> <span class="number">10</span></span><span class="synctx-1">
}</span><span class="synctx-0">;</span></pre>

The compiler will accept this code because there is only one `a` being used.

## Debugging

> **TODO**: `trace_macros!`, `log_syntax!`, `--pretty expanded`, `--pretty expanded,hygiene`.

`rustc` provides a number of tools to debug macros.  One of the most useful is `trace_macros!`, which is a directive to the compiler instructing it to dump every macro invocation prior to expansion.  For example, given the following:

```rust
# // Note: make sure to use a nightly channel compiler.
#![feature(trace_macros)]

macro_rules! each_tt {
    () => {};
    ($_tt:tt $($rest:tt)*) => {each_tt!($($rest)*);};
}

each_tt!(foo bar baz quux);
trace_macros!(true);
each_tt!(spim wak plee whum);
trace_macros!(false);
each_tt!(trom qlip winp xod);
# 
# fn main() {}
```

The output is:

```text
each_tt! { spim wak plee whum }
each_tt! { wak plee whum }
each_tt! { plee whum }
each_tt! { whum }
each_tt! {  }
```

This is *particularly* invaluable when debugging deeply recursive macros.

Secondly, there is `log_syntax!` which causes the compiler to output all tokens passed to it.  For example, this example makes the compiler sing a song:

```rust
# // Note: make sure to use a nightly channel compiler.
#![feature(log_syntax)]

macro_rules! sing {
    () => {};
    ($tt:tt $($rest:tt)*) => {log_syntax!($tt); sing!($($rest)*);};
}

sing! {
    ^ < @ < . @ *
    '\x08' '{' '"' _ # ' '
    - @ '$' && / _ %
    ! ( '\t' @ | = >
    ; '\x08' '\'' + '$' ? '\x7f'
    , # '"' ~ | ) '\x07'
}
# 
# fn main() {}
```

This can be used to do slightly more targeted debugging than `trace_macros!`.

Sometimes, it is what the macro *expands to* that proves problematic.  For this, the `--pretty` argument to the compiler can be used.  Given the following code:

```ignore
// Shorthand for initialising a `String`.
macro_rules! S {
    ($e:expr) => {String::from($e)};
}

fn main() {
    let world = S!("World");
    println!("Hello, {}!", world);
}
```

compiled with the following command:

```shell
rustc -Z unstable-options --pretty expanded hello.rs
```

produces the following output (modified for formatting):

```ignore
#![feature(no_std, prelude_import)]
#![no_std]
#[prelude_import]
use std::prelude::v1::*;
#[macro_use]
extern crate std as std;
// Shorthand for initialising a `String`.
fn main() {
    let world = String::from("World");
    ::std::io::_print(::std::fmt::Arguments::new_v1(
        {
            static __STATIC_FMTSTR: &'static [&'static str]
                = &["Hello, ", "!\n"];
            __STATIC_FMTSTR
        },
        &match (&world,) {
             (__arg0,) => [
                ::std::fmt::ArgumentV1::new(__arg0, ::std::fmt::Display::fmt)
            ],
        }
    ));
}
```

Other options to `--pretty` can be listed using `rustc -Z unstable-options --help -v`; a full list is not provided since, as implied by the name, any such list would be subject to change at any time.

## Scoping

> **TODO**: scoping in modules, functions.

The way in which macros are scoped can be somewhat unintuitive.  Firstly, unlike everything else in the languages, macros will remain visible in sub-modules.

```rust
macro_rules! X { () => {}; }
mod a {
    X!(); // defined
}
mod b {
    X!(); // defined
}
mod c {
    X!(); // defined
}
# fn main() {}
```

> **Note**: In these examples, remember that all of them have the *same behaviour* when the module contents are in separate files.

Secondly, *also* unlike everything else in the language, macros are only accessible *after* their definition.  Also note that this example demonstrates how macros do not "leak" out of their defining scope:

```rust
mod a {
    // X!(); // undefined
}
mod b {
    // X!(); // undefined
    macro_rules! X { () => {}; }
    X!(); // defined
}
mod c {
    // X!(); // undefined
}
# fn main() {}
```

To be clear, this lexical order dependency applies even if you move the macro to an outer scope:

```rust
mod a {
    // X!(); // undefined
}
macro_rules! X { () => {}; }
mod b {
    X!(); // defined
}
mod c {
    X!(); // defined
}
# fn main() {}
```

However, this dependency *does not* apply to macros themselves:

```rust
mod a {
    // X!(); // undefined
}
macro_rules! X { () => { Y!(); }; }
mod b {
    // X!(); // defined, but Y! is undefined
}
macro_rules! Y { () => {}; }
mod c {
    X!(); // defined, and so is Y!
}
# fn main() {}
```

Macros can be exported from a module using the `#[macro_use]` attribute.

```rust
mod a {
    // X!(); // undefined
}
#[macro_use]
mod b {
    macro_rules! X { () => {}; }
    X!(); // defined
}
mod c {
    X!(); // defined
}
# fn main() {}
```

Note that this can interact in somewhat bizarre ways due to the fact that identifiers in a macro (including other macros) are only resolved upon expansion:

```rust
mod a {
    // X!(); // undefined
}
#[macro_use]
mod b {
    macro_rules! X { () => { Y!(); }; }
    // X!(); // defined, but Y! is undefined
}
macro_rules! Y { () => {}; }
mod c {
    X!(); // defined, and so is Y!
}
# fn main() {}
```

Finally, note that with the exception of `#[macro_use]` (which doesn't apply), these scoping behaviours apply to *functions* as well:

```rust
macro_rules! X {
    () => { Y!() };
}

fn a() {
    macro_rules! Y { () => {"Hi!"} }
    assert_eq!(X!(), "Hi!");
    {
        assert_eq!(X!(), "Hi!");
        macro_rules! Y { () => {"Bye!"} }
        assert_eq!(X!(), "Bye!");
    }
    assert_eq!(X!(), "Hi!");
}

fn b() {
    macro_rules! Y { () => {"One more"} }
    assert_eq!(X!(), "One more");
}
# 
# fn main() {
#     a();
#     b();
# }
```

These scoping rules are why a common piece of advice is to place all macros which should be accessible "crate wide" at the very top of your root module, before any other modules.  This ensures they are available *consistently*.

## Import/Export

> **TODO**: `macro_use`, `macro_export`.

> **TODO**: `$crate`.
