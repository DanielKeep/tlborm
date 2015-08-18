#![recursion_limit = "158"]

type CellType = u8;
const MEM_SIZE: usize = 30_000;

macro_rules! Ook {
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

    /*
    
    ## Everything Else
    
    */
    
    (@e $syms:tt; ()) => {};

    (@e ($a:expr, $i:expr, $inc:expr, $dec:expr, $r:expr, $w:expr, $re:expr); (Ook. Ook? $($tail:tt)*)) => {
        $i = ($i + 1) % MEM_SIZE;
        Ook!(@e ($a, $i, $inc, $dec, $r, $w, $re); ($($tail)*));
    };
    
    (@e ($a:expr, $i:expr, $inc:expr, $dec:expr, $r:expr, $w:expr, $re:expr); (Ook? Ook. $($tail:tt)*)) => {
        $i = if $i == 0 { MEM_SIZE } else { $i } - 1;
        Ook!(@e ($a, $i, $inc, $dec, $r, $w, $re); ($($tail)*));
    };
    
    (@e ($a:expr, $i:expr, $inc:expr, $dec:expr, $r:expr, $w:expr, $re:expr); (Ook. Ook. $($tail:tt)*)) => {
        $inc($a, $i);
        Ook!(@e ($a, $i, $inc, $dec, $r, $w, $re); ($($tail)*));
    };
    
    (@e ($a:expr, $i:expr, $inc:expr, $dec:expr, $r:expr, $w:expr, $re:expr); (Ook! Ook! $($tail:tt)*)) => {
        $dec($a, $i);
        Ook!(@e ($a, $i, $inc, $dec, $r, $w, $re); ($($tail)*));
    };
    
    (@e ($a:expr, $i:expr, $inc:expr, $dec:expr, $r:expr, $w:expr, $re:expr); (Ook! Ook. $($tail:tt)*)) => {
        try!($w.write_all(&$a[$i .. $i+1]));
        Ook!(@e ($a, $i, $inc, $dec, $r, $w, $re); ($($tail)*));
    };
    
    (@e ($a:expr, $i:expr, $inc:expr, $dec:expr, $r:expr, $w:expr, $re:expr); (Ook. Ook! $($tail:tt)*)) => {
        try!(
            match $r.read(&mut $a[$i .. $i+1]) {
                Ok(0) => Err($re()),
                ok @ Ok(..) => ok,
                err @ Err(..) => err
            }
        );
        Ook!(@e ($a, $i, $inc, $dec, $r, $w, $re); ($($tail)*));
    };
    
    (@e ($a:expr, $i:expr, $inc:expr, $dec:expr, $r:expr, $w:expr, $re:expr); (Ook! Ook? $($tail:tt)*)) => {
        while $a[$i] != 0 {
            Ook!(@x ($a, $i, $inc, $dec, $r, $w, $re); (); (); ($($tail)*));
        }
        Ook!(@s ($a, $i, $inc, $dec, $r, $w, $re); (); ($($tail)*));
    };

    /*
    
    ## Loop Extraction

    The input is of the form `(@x (syms...) (depth...); (buf...); tail...)`.

    `syms` is the set of symbols (expression, actually), that we need for the
    actual expanded code.  They're parenthesised so that we can pass them
    around as a `tt`.
    
    `depth` is the current nesting depth, represented as a paren'd sequence of
    `@`s.  The parens are *empty* in the outer-most loop.
    
    `buf` is the sequence of collected tokens that belong inside the outer-most
    loop.  These are kept inside parens.
    
    `tail` is the sequence of tokens yet to be processed.
    */
    (@x $syms:tt; (); ($($buf:tt)*); (Ook? Ook! $($tail:tt)*)) => {
        // Outer-most loop is closed.  Process the buffered tokens.
        Ook!(@e $syms; ($($buf)*));
    };
    
    (@x $syms:tt; ($($depth:tt)*); ($($buf:tt)*); (Ook! Ook? $($tail:tt)*)) => {
        // One level deeper.
        Ook!(@x $syms; (@ $($depth)*); ($($buf)* Ook! Ook?); ($($tail)*));
    };
    
    (@x $syms:tt; (@ $($depth:tt)*); ($($buf:tt)*); (Ook? Ook! $($tail:tt)*)) => {
        // One level higher.
        Ook!(@x $syms; ($($depth)*); ($($buf)* Ook? Ook!); ($($tail)*));
    };
    
    (@x $syms:tt; $depth:tt; ($($buf:tt)*); (Ook $op0:tt Ook $op1:tt $($tail:tt)*)) => {
        Ook!(@x $syms; $depth; ($($buf)* Ook $op0 Ook $op1); ($($tail)*));
    };
    
    /*
    
    ## Loop Skipping
    
    This is the same as above, except sans-buffer.  We just need to find the
    end of the loop, then resume normal processing.
    
    */
    (@s $syms:tt; (); (Ook? Ook! $($tail:tt)*)) => {
        // Outer-most loop is closed.  Resume normal operation.
        Ook!(@e $syms; ($($tail)*));
    };
    
    (@s $syms:tt; ($($depth:tt)*); (Ook! Ook? $($tail:tt)*)) => {
        // One level deeper.
        Ook!(@s $syms; (@ $($depth)*); ($($tail)*));
    };
    
    (@s $syms:tt; (@ $($depth:tt)*); (Ook? Ook! $($tail:tt)*)) => {
        // One level higher.
        Ook!(@s $syms; ($($depth)*); ($($tail)*));
    };

    (@s $syms:tt; ($($depth:tt)*); (Ook $op0:tt Ook $op1:tt $($tail:tt)*)) => {
        Ook!(@s $syms; ($($depth)*); ($($tail)*));
    };

    // This is dangerous if you get it wrong!
    ($($Ooks:tt)*) => {
        Ook!(@start $($Ooks)*)
    };
}

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
