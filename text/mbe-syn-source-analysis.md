% Source Analysis

The first stage of compilation for a Rust program is tokenization.  This is where the source text is transformed into a sequence of tokens (*i.e.* indivisible lexical units; the programming language equivalent of "words").  Rust has various kinds of tokens, such as:

* Identifiers: `foo`, `Bambous`, `self`, `we_can_dance`, `LaCaravane`, …
* Integers: `42`, `72u32`, `0_______0`, …
* Keywords: `_`, `fn`, `self`, `match`, `yield`, `macro`, …
* Lifetimes: `'a`, `'b`, `'a_rare_long_lifetime_name`, …
* Strings: `""`, `"Leicester"`, `r##"venezuelan beaver"##`, …
* Symbols: `[`, `:`, `::`, `->`, `@`, `<-`, …

…among others.  There are some things to note about the above: first, `self` is both an identifier *and* a keyword.  In almost all cases, `self` is a keyword, but it *is* possible for it to be *treated* as an identifier, which will come up later (along with much cursing).  Secondly, the list of keywords includes some suspicious entries such as `yield` and `macro` that aren't *actually* in the language, but *are* parsed by the compiler—these are reserved for future use.  Third, the list of symbols *also* includes entries that aren't used by the language.  In the case of `<-`, it is vestigial: it was removed from the grammar, but not from the lexer.  As a final point, note that `::` is a distinct token; it is not simply two adjacent `:` tokens.  The same is true of all multi-character symbol tokens in Rust, as of Rust 1.2. [^wither-at]

[^wither-at]: `@` has a purpose, though most people seem to forget about it completely: it is used in patterns to bind a non-terminal part of the pattern to a name.  Even a member of the Rust core team, proof-reading this chapter, who *specifically* brought up this section, didn't remember that `@` has a purpose.  Poor, poor swirly.

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

Token trees are somewhere between tokens and the AST.  Firstly, *almost* all tokens are also token trees; more specifically, they are *leaves*.  There is one other kind of thing that can be a token tree leaf, but we will come back to that later.

The only basic tokens that are *not* leaves are the "grouping" tokens: `(...)`, `[...]`, and `{...}`.  These three are the *interior nodes* of token trees, and what give them their structure.  To give a concrete example, this sequence of tokens:

```ignore
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
