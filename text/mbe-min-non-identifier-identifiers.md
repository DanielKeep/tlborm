% Non-Identifier Identifiers

There are two tokens which you are likely to run into eventually that *look* like identifiers, but aren't.  Except when they are.

First is `self`.  This is *very definitely* a keyword.  However, it also happens to fit the definition of an identifier.  In regular Rust code, there's no way for `self` to be interpreted as an identifier, but it *can* happen with macros:

```rust
macro_rules! what_is {
    (self) => {"the keyword `self`"};
    ($i:ident) => {concat!("the identifier `", stringify!($i), "`")};
}

macro_rules! call_with_ident {
    ($c:ident($i:ident)) => {$c!($i)};
}

fn main() {
    println!("{}", what_is!(self));
    println!("{}", call_with_ident!(what_is(self)));
}
```

The above outputs:

```text
the keyword `self`
the keyword `self`
```

But that makes no sense; `call_with_ident!` required an identifier, matched one, and substituted it!  So `self` is both a keyword and not a keyword at the same time.  You might wonder how this is in any way important.  Take this example:

```ignore
macro_rules! make_mutable {
    ($i:ident) => {let mut $i = $i;};
}

struct Dummy(i32);

impl Dummy {
    fn double(self) -> Dummy {
        make_mutable!(self);
        self.0 *= 2;
        self
    }
}
# 
# fn main() {
#     println!("{:?}", Dummy(4).double().0);
# }
```

This fails to compile with:

```text
<anon>:2:28: 2:30 error: expected identifier, found keyword `self`
<anon>:2     ($i:ident) => {let mut $i = $i;};
                                    ^~
```

So the macro will happily match `self` as an identifier, allowing you to use it in cases where you can't actually use it.  But, fine; it somehow remembers that `self` is a keyword even when it's an identifier, so you *should* be able to do this, right?

```ignore
macro_rules! make_self_mutable {
    ($i:ident) => {let mut $i = self;};
}

struct Dummy(i32);

impl Dummy {
    fn double(self) -> Dummy {
        make_self_mutable!(mut_self);
        mut_self.0 *= 2;
        mut_self
    }
}
# 
# fn main() {
#     println!("{:?}", Dummy(4).double().0);
# }
```

This fails with:

```text
<anon>:2:33: 2:37 error: `self` is not available in a static method. Maybe a `self` argument is missing? [E0424]
<anon>:2     ($i:ident) => {let mut $i = self;};
                                         ^~~~
```

*That* doesn't make any sense, either.  It's *not* in a static method.  It's almost like it's complaining that the `self` it's trying to use isn't the *same* `self`... as though the `self` keyword has hygiene, like an... identifier.

```ignore
macro_rules! double_method {
    ($body:expr) => {
        fn double(mut self) -> Dummy {
            $body
        }
    };
}

struct Dummy(i32);

impl Dummy {
    double_method! {{
        self.0 *= 2;
        self
    }}
}
# 
# fn main() {
#     println!("{:?}", Dummy(4).double().0);
# }
```

Same error.  What about...

```rust
macro_rules! double_method {
    ($self_:ident, $body:expr) => {
        fn double(mut $self_) -> Dummy {
            $body
        }
    };
}

struct Dummy(i32);

impl Dummy {
    double_method! {self, {
        self.0 *= 2;
        self
    }}
}
# 
# fn main() {
#     println!("{:?}", Dummy(4).double().0);
# }
```

At last, *this works*.  So `self` is both a keyword *and* an identifier when it feels like it.  Surely this works for other, similar constructs, right?

```ignore
macro_rules! double_method {
    ($self_:ident, $body:expr) => {
        fn double($self_) -> Dummy {
            $body
        }
    };
}

struct Dummy(i32);

impl Dummy {
    double_method! {_, 0}
}
# 
# fn main() {
#     println!("{:?}", Dummy(4).double().0);
# }
```

```text
<anon>:12:21: 12:22 error: expected ident, found _
<anon>:12     double_method! {_, 0}
                              ^
```

No, of course not.  `_` is a keyword that is valid in patterns and expressions, but somehow *isn't* an identifier like the keyword `self` is, despite matching the definition of an identifier just the same.

You might think you can get around this by using `$self_:pat` instead; that way, `_` will match!  Except, no, because `self` isn't a pattern.  Joy.

The only work around for this (in cases where you want to accept some combination of these tokens) is to use a `tt` matcher instead.
