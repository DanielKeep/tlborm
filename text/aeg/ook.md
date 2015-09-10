% Ook!

This macro is an implementation of the [Ook! esoteric language](http://www.dangermouse.net/esoteric/ook.html), which is isomorphic to the [Brainfuck esoteric language](http://www.muppetlabs.com/~breadbox/bf/).

The execution model for the language is very simple: memory is represented as an array of "cells" (typically at least 8-bits) of some indeterminate number (usually at least 30,000).  There is a pointer into memory which starts off at position 0.  Finally, there is an execution stack (used to implement looping) and pointer into the program, although these last two are not exposed to the running program; they are properties of the runtime itself.

The language itself is comprised of just three tokens: `Ook.`, `Ook?`, and `Ook!`.  These are combined in pairs to form the eight different operations:

* `Ook. Ook?` - increment pointer.
* `Ook? Ook.` - decrement pointer.
* `Ook. Ook.` - increment pointed-to memory cell.
* `Ook! Ook!` - decrement pointed-to memory cell.
* `Ook! Ook.` - write pointed-to memory cell to standard output.
* `Ook. Ook!` - read from standard input into pointed-to memory cell.
* `Ook! Ook?` - begin a loop.
* `Ook? Ook!` - jump back to start of loop if pointed-to memory cell is not zero; otherwise, continue.

Ook! is interesting because it is known to be Turing-complete, meaning that any environment in which you can implement it must *also* be Turing-complete.

## Implementation

```ignore
#![recursion_limit = "158"]
```

This is, in fact, the lowest possible recursion limit for which the example program provided at the end will actually compile.  If you're wondering what could be so fantastically complex that it would *justify* a recursion limit nearly five times the default limit... [take a wild guess](https://en.wikipedia.org/wiki/Hello_world_program).

```ignore
type CellType = u8;
const MEM_SIZE: usize = 30_000;
```

These are here purely to ensure they are visible to the macro expansion.[^*]

[^*]: They *could* have been defined within the macro, but then they would have to have been explicitly passed around (due to hygiene).  To be honest, by the time I realised I *needed* to define these, the macro was already mostly written and... well, would *you* want to go through and fix this thing up if you didn't *absolutely need* to?

```ignore
macro_rules! Ook {
```

The name should *probably* have been `ook!` to match the standard naming convention, but the opportunity was simply too good to pass up.

The rules for this macro are broken up into sections using the [internal rules](../pat/README.html#internal-rules) pattern.

The first of these will be a `@start` rule, which takes care of setting up the block in which the rest of our expansion will happen.  There is nothing particularly interesting in this: we define some variables and helper functions, then do the bulk of the expansion.

A few small notes:

* We are expanding into a function largely so that we can use `try!` to simplify error handling.
* The use of underscore-prefixed names is so that the compiler will not complain about unused functions or variables if, for example, the user writes an Ook! program that does no I/O.

```ignore
    (@start $($Ooks:tt)*) => {
        {
            fn ook() -> ::std::io::Result<Vec<CellType>> {
                use ::std::io;
                use ::std::io::prelude::*;
    
                fn _re() -> io::Error {
                    io::Error::new(
                        io::ErrorKind::Other,
                        String::from("ran out of input"))
                }
                
                fn _inc(a: &mut [u8], i: usize) {
                    let c = &mut a[i];
                    *c = c.wrapping_add(1);
                }
                
                fn _dec(a: &mut [u8], i: usize) {
                    let c = &mut a[i];
                    *c = c.wrapping_sub(1);
                }
    
                let _r = &mut io::stdin();
                let _w = &mut io::stdout();
        
                let mut _a: Vec<CellType> = Vec::with_capacity(MEM_SIZE);
                _a.extend(::std::iter::repeat(0).take(MEM_SIZE));
                let mut _i = 0;
                {
                    let _a = &mut *_a;
                    Ook!(@e (_a, _i, _inc, _dec, _r, _w, _re); ($($Ooks)*));
                }
                Ok(_a)
            }
            ook()
        }
    };
```

### Opcode parsing

Next are the "execute" rules, which are used to parse opcodes from the input.

The general form of these rules is `(@e $syms; ($input))`.  As you can see from the `@start` rule, `$syms` is the collection of symbols needed to actually implement the program: input, output, the memory array, *etc.*.  We are using [TT bundling](../pat/README.html#tt-bundling) to simplify forwarding of these symbols through later, intermediate rules.

First, is the rule that terminates our recursion: once we have no more input, we stop.

```ignore
    (@e $syms:tt; ()) => {};
```

Next, we have a single rule for *almost* each opcode.  For these, we strip off the opcode, emit the corresponding Rust code, then recurse on the input tail: a textbook [TT muncher](../pat/README.html#incremental-tt-munchers).

```ignore
    // Increment pointer.
    (@e ($a:expr, $i:expr, $inc:expr, $dec:expr, $r:expr, $w:expr, $re:expr);
        (Ook. Ook? $($tail:tt)*))
    => {
        $i = ($i + 1) % MEM_SIZE;
        Ook!(@e ($a, $i, $inc, $dec, $r, $w, $re); ($($tail)*));
    };
    
    // Decrement pointer.
    (@e ($a:expr, $i:expr, $inc:expr, $dec:expr, $r:expr, $w:expr, $re:expr);
        (Ook? Ook. $($tail:tt)*))
    => {
        $i = if $i == 0 { MEM_SIZE } else { $i } - 1;
        Ook!(@e ($a, $i, $inc, $dec, $r, $w, $re); ($($tail)*));
    };
    
    // Increment pointee.
    (@e ($a:expr, $i:expr, $inc:expr, $dec:expr, $r:expr, $w:expr, $re:expr);
        (Ook. Ook. $($tail:tt)*))
    => {
        $inc($a, $i);
        Ook!(@e ($a, $i, $inc, $dec, $r, $w, $re); ($($tail)*));
    };
    
    // Decrement pointee.
    (@e ($a:expr, $i:expr, $inc:expr, $dec:expr, $r:expr, $w:expr, $re:expr);
        (Ook! Ook! $($tail:tt)*))
    => {
        $dec($a, $i);
        Ook!(@e ($a, $i, $inc, $dec, $r, $w, $re); ($($tail)*));
    };
    
    // Write to stdout.
    (@e ($a:expr, $i:expr, $inc:expr, $dec:expr, $r:expr, $w:expr, $re:expr);
        (Ook! Ook. $($tail:tt)*))
    => {
        try!($w.write_all(&$a[$i .. $i+1]));
        Ook!(@e ($a, $i, $inc, $dec, $r, $w, $re); ($($tail)*));
    };
    
    // Read from stdin.
    (@e ($a:expr, $i:expr, $inc:expr, $dec:expr, $r:expr, $w:expr, $re:expr);
        (Ook. Ook! $($tail:tt)*))
    => {
        try!(
            match $r.read(&mut $a[$i .. $i+1]) {
                Ok(0) => Err($re()),
                ok @ Ok(..) => ok,
                err @ Err(..) => err
            }
        );
        Ook!(@e ($a, $i, $inc, $dec, $r, $w, $re); ($($tail)*));
    };
```

Here is where things get more complicated.  This opcode, `Ook! Ook?`, marks the start of a loop.  Ook! loops are translated to the following Rust code:

> **Note**: this is *not* part of the larger code.
>
> ```ignore
> while memory[ptr] != 0 {
>     // Contents of loop
> }
> ```

Of course, we cannot *actually* emit an incomplete loop.  This *could* be solved by using [pushdown](../pat/README.html#push-down-accumulation), were it not for a more fundamental problem: we cannot *write* `while memory[ptr] != {`, at all, *anywhere*.  This is because doing so would introduce an unbalanced brace.

The solution to this is to actually split the input into two parts: everything *inside* the loop, and everything *after* it.  The `@x` rules handle the first, `@s` the latter.

```ignore
    (@e ($a:expr, $i:expr, $inc:expr, $dec:expr, $r:expr, $w:expr, $re:expr);
        (Ook! Ook? $($tail:tt)*))
    => {
        while $a[$i] != 0 {
            Ook!(@x ($a, $i, $inc, $dec, $r, $w, $re); (); (); ($($tail)*));
        }
        Ook!(@s ($a, $i, $inc, $dec, $r, $w, $re); (); ($($tail)*));
    };
```

### Loop extraction

Next are the `@x`, or "extraction", rules.  These are responsible for taking an input tail and extracting the contents of a loop.  The general form of these rules is: `(@x $sym; $depth; $buf; $tail)`.

The purpose of `$sym` is the same as above.  `$tail` is the input to be parsed, whilst `$buf` is a [push-down accumulation buffer](../pat/README.html#push-down-accumulation) into which we will collect the opcodes that are inside the loop.  But what of `$depth`?

A complication to all this is that loops can be *nested*.  Thus, we must have some way of keeping track of how many levels deep we currently are.  We must track this accurately enough to not stop parsing too early, nor too late, but when the level is *just right*.[^justright]

[^justright]:
    It is a little known fact[^fact] that the story of Goldie Locks was actually an allegory for accurate lexical parsing techniques.

[^fact]: And by "fact" I mean "shameless fabrication".

Since we cannot do arithmetic in macros, and it would be infeasible to write out explicit integer-matching rules (imagine the following rules all copy & pasted for a non-trivial number of positive integers), we will instead fall back on one of the most ancient and venerable counting methods in history: counting on our fingers.

But as macros don't *have* fingers, we'll use a [token abacus counter](../pat/README.html#abacus-counters) instead.  Specifically, we will use `@`s, where each `@` represents one additional level of depth.  If we keep these `@`s contained in a group, we can implement the three important operations we need:

* Increment: match `($($depth:tt)*)`, substitute `(@ $($depth)*)`.
* Decrement: match `(@ $($depth:tt)*)`, substitute `($($depth)*)`.
* Compare to zero: match `()`.

First is a rule to detect when we find the matching `Ook? Ook!` sequence that closes the loop we're parsing.  In this case, we feed the accumulated loop contents to the previously defined `@e` rules.

Note that we *do not* need to do anything with the remaining input tail (that will be handled by the `@s` rules).

```ignore
    (@x $syms:tt; (); ($($buf:tt)*);
        (Ook? Ook! $($tail:tt)*))
    => {
        // Outer-most loop is closed.  Process the buffered tokens.
        Ook!(@e $syms; ($($buf)*));
    };
```

Next, we have rules for entering and exiting nested loops.  These adjust the counter and add the opcodes to the buffer.

```ignore
    (@x $syms:tt; ($($depth:tt)*); ($($buf:tt)*);
        (Ook! Ook? $($tail:tt)*))
    => {
        // One level deeper.
        Ook!(@x $syms; (@ $($depth)*); ($($buf)* Ook! Ook?); ($($tail)*));
    };
    
    (@x $syms:tt; (@ $($depth:tt)*); ($($buf:tt)*);
        (Ook? Ook! $($tail:tt)*))
    => {
        // One level higher.
        Ook!(@x $syms; ($($depth)*); ($($buf)* Ook? Ook!); ($($tail)*));
    };
```

Finally, we have a rule for "everything else".  Note the `$op0` and `$op1` captures: as far as Rust is concerned, our Ook! tokens are always *two* Rust tokens: the identifier `Ook`, and another token.  Thus, we can generalise over all non-loop opcodes by matching `!`, `?`, and `.` as `tt`s.

Here, we leave `$depth` untouched and just add the opcodes to the buffer.

```ignore
    (@x $syms:tt; $depth:tt; ($($buf:tt)*);
        (Ook $op0:tt Ook $op1:tt $($tail:tt)*))
    => {
        Ook!(@x $syms; $depth; ($($buf)* Ook $op0 Ook $op1); ($($tail)*));
    };
```

### Loop Skipping

This is *broadly* the same as loop extraction, except we don't care about the *contents* of the loop (and as such, don't need the accumulation buffer).  All we need to know is when we are *past* the loop.  At that point, we resume processing the input using the `@e` rules.

As such, these rules are presented without further exposition.

```ignore
    // End of loop.
    (@s $syms:tt; ();
        (Ook? Ook! $($tail:tt)*))
    => {
        Ook!(@e $syms; ($($tail)*));
    };

    // Enter nested loop.
    (@s $syms:tt; ($($depth:tt)*);
        (Ook! Ook? $($tail:tt)*))
    => {
        Ook!(@s $syms; (@ $($depth)*); ($($tail)*));
    };
    
    // Exit nested loop.
    (@s $syms:tt; (@ $($depth:tt)*);
        (Ook? Ook! $($tail:tt)*))
    => {
        Ook!(@s $syms; ($($depth)*); ($($tail)*));
    };

    // Not a loop opcode.
    (@s $syms:tt; ($($depth:tt)*);
        (Ook $op0:tt Ook $op1:tt $($tail:tt)*))
    => {
        Ook!(@s $syms; ($($depth)*); ($($tail)*));
    };
```

### Entry point

This is the only non-internal rule.

It is worth noting that because this formulation simply matches *all* tokens provided to it, it is *extremely dangerous*.  Any mistake can cause an invocation to fail to match all the above rules, thus falling down to this one and triggering an infinite recursion.

When you are writing, modifying, or debugging a macro like this, it is wise to temporarily prefix rules such as this one with something, such as `@entry`.  This prevents the infinite recursion case, and you are more likely to get matcher errors at the appropriate place.

```ignore
    ($($Ooks:tt)*) => {
        Ook!(@start $($Ooks)*)
    };
}
```

### Usage

Here, finally, is our test program.

```ignore
fn main() {
    let _ = Ook!(
        Ook. Ook?  Ook. Ook.  Ook. Ook.  Ook. Ook.
        Ook. Ook.  Ook. Ook.  Ook. Ook.  Ook. Ook.
        Ook. Ook.  Ook. Ook.  Ook! Ook?  Ook? Ook.
        Ook. Ook.  Ook. Ook.  Ook. Ook.  Ook. Ook.
        Ook. Ook.  Ook. Ook.  Ook. Ook.  Ook. Ook.
        Ook. Ook?  Ook! Ook!  Ook? Ook!  Ook? Ook.
        Ook! Ook.  Ook. Ook?  Ook. Ook.  Ook. Ook.
        Ook. Ook.  Ook. Ook.  Ook. Ook.  Ook. Ook.
        Ook. Ook.  Ook! Ook?  Ook? Ook.  Ook. Ook.
        Ook. Ook.  Ook. Ook.  Ook. Ook.  Ook. Ook?
        Ook! Ook!  Ook? Ook!  Ook? Ook.  Ook. Ook.
        Ook! Ook.  Ook. Ook.  Ook. Ook.  Ook. Ook.
        Ook. Ook.  Ook. Ook.  Ook. Ook.  Ook. Ook.
        Ook! Ook.  Ook! Ook.  Ook. Ook.  Ook. Ook.
        Ook. Ook.  Ook! Ook.  Ook. Ook?  Ook. Ook?
        Ook. Ook?  Ook. Ook.  Ook. Ook.  Ook. Ook.
        Ook. Ook.  Ook. Ook.  Ook. Ook.  Ook. Ook.
        Ook. Ook.  Ook! Ook?  Ook? Ook.  Ook. Ook.
        Ook. Ook.  Ook. Ook.  Ook. Ook.  Ook. Ook?
        Ook! Ook!  Ook? Ook!  Ook? Ook.  Ook! Ook.
        Ook. Ook?  Ook. Ook?  Ook. Ook?  Ook. Ook.
        Ook. Ook.  Ook. Ook.  Ook. Ook.  Ook. Ook.
        Ook. Ook.  Ook. Ook.  Ook. Ook.  Ook. Ook.
        Ook. Ook.  Ook! Ook?  Ook? Ook.  Ook. Ook.
        Ook. Ook.  Ook. Ook.  Ook. Ook.  Ook. Ook.
        Ook. Ook.  Ook. Ook.  Ook. Ook.  Ook. Ook.
        Ook. Ook?  Ook! Ook!  Ook? Ook!  Ook? Ook.
        Ook! Ook!  Ook! Ook!  Ook! Ook!  Ook! Ook.
        Ook? Ook.  Ook? Ook.  Ook? Ook.  Ook? Ook.
        Ook! Ook.  Ook. Ook.  Ook. Ook.  Ook. Ook.
        Ook! Ook.  Ook! Ook!  Ook! Ook!  Ook! Ook!
        Ook! Ook!  Ook! Ook!  Ook! Ook!  Ook! Ook.
        Ook! Ook!  Ook! Ook!  Ook! Ook!  Ook! Ook!
        Ook! Ook!  Ook! Ook!  Ook! Ook!  Ook! Ook!
        Ook! Ook.  Ook. Ook?  Ook. Ook?  Ook. Ook.
        Ook! Ook.  Ook! Ook?  Ook! Ook!  Ook? Ook!
        Ook. Ook.  Ook. Ook.  Ook. Ook.  Ook. Ook.
        Ook. Ook.  Ook. Ook.  Ook. Ook.  Ook. Ook.
        Ook. Ook.  Ook. Ook.  Ook! Ook.
    );
}
```

The output when run (after a considerable pause for the compiler to do hundreds of recursive macro expansions) is:

```text
Hello World!
```

With that, we have demonstrated the horrifying truth that `macro_rules!` is Turing-complete!

### An aside

This was based on a macro implementing an isomorphic language called "Hodor!".  Manish Goregaokar then [implemented a Brainfuck interpreter using the Hodor! macro](https://www.reddit.com/r/rust/comments/39wvrm/hodor_esolang_as_a_rust_macro/cs76rqk?context=10000).  So that is a Brainfuck interpreter written in Hodor! which was itself implemented using `macro_rules!`.

Legend has it that after raising the recursion limit to *three million* and allowing it to run for *four days*, it finally finished.

...by overflowing the stack and aborting.  To this day, esolang-as-macro remains a decidedly *non-viable* method of development with Rust.
