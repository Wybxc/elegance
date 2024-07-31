# Elegance

A pretty-printing library for Rust with a focus on speed and compactness.

## Usage

Create a printer:

```rust,ignore
let mut pp = Printer::new(String::new(), 40);
```

Add some text and spaces:

```rust,ignore
pp.text("Hello, world!")?;
pp.space()?; // breakable space
pp.hard_break()?; // forced line break
```

Enclose structures in groups:

```rust,ignore
pp.group(2, |pp| {
    pp.text("foo")?;
    pp.space()?;
    pp.text("bar")
})?;
```

Finish the document:

```rust,ignore
let result = pp.finish()?;
println!("{}", result);
```

### Streaming output

The printer can write to any `std::io::Write` implementation.

```rust,ignore
use elegant::{Printer, Io};
let mut pp = Printer::new(Io(std::io::stdout()), 40);
```

## Examples

```rust
use elegance::*;

enum SExp {
    Atom(u32),
    List(Vec<SExp>),
}

impl SExp {
    pub fn print<R: Render>(&self, pp: &mut Printer<R>) -> Result<(), R::Error> {
        match self {
            SExp::Atom(x) => pp.text(format!("{}", x))?,
            SExp::List(xs) => pp.group(1, |pp| {
                pp.text("(")?;
                if let Some((first, rest)) = xs.split_first() {
                    first.print(pp)?;
                    for v in rest {
                        pp.space()?;
                        v.print(pp)?;
                    }
                }
                pp.text(")")
            })?,
        }
        Ok(())
    }
}

fn main() {
    let exp = SExp::List(vec![
        SExp::List(vec![SExp::Atom(1)]),
        SExp::List(vec![SExp::Atom(2), SExp::Atom(3)]),
        SExp::List(vec![SExp::Atom(4), SExp::Atom(5), SExp::Atom(6)]),
    ]);

    let mut printer = Printer::new(String::new(), 10);
    exp.print(&mut printer).unwrap();
    let result = printer.finish().unwrap();

    assert_eq!(result, indoc::indoc! {"
        ((1)
         (2 3)
         (4 5 6))"});
}
```

## Differences from other libraries

This crate implements an Oppen-style pretty-printing library, while the [pretty](https://docs.rs/pretty/latest/pretty/) crate follows a Walder-style approach.

In Walder-style pretty-printing, documents are constructed using a composable `Doc` type and combinators. Here's an example:

```rust,ignore
impl SExp {
    /// Returns a pretty printed representation of `self`.
    pub fn to_doc(&self) -> RcDoc<()> {
        match *self {
            Atom(ref x) => RcDoc::as_string(x),
            List(ref xs) =>
                RcDoc::text("(")
                    .append(RcDoc::intersperse(xs.iter().map(|x| x.to_doc()), Doc::line()).nest(1).group())
                    .append(RcDoc::text(")"))
        }
    }
}
```

This method is particularly suitable for functional programming languages but may not be ideal for Rust. Converting a syntax tree into a `Doc` requires additional memory allocation proportional to the size of the entire document.

The key difference with this library is that it represents the structure of the printed document through control flow rather than data structures. As a result, the printing process is fully streamed and operates within a constant memory footprint.

## References

- Oppen, Dereck C. "Prettyprinting." ACM Transactions on Programming Languages and Systems (TOPLAS) 2.4 (1980): 465-483. <https://doi.org/10.1145/357114.357115>
- Swierstra, S. Doaitse, and Olaf Chitil. "Linear, bounded, functional pretty-printing." Journal of Functional Programming 19.1 (2009): 1-16. <https://doi.org/10.1017/s0956796808006990>
