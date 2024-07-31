use std::io;

use elegance::{
    core::Printer,
    render::{Io, Render},
};

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
                        pp.soft_break()?;
                        v.print(pp)?;
                    }
                }
                pp.text(")")
            })?,
        }
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let exp = SExp::List(vec![
        SExp::List(vec![SExp::Atom(1)]),
        SExp::List(vec![SExp::Atom(2), SExp::Atom(3)]),
        SExp::List(vec![SExp::Atom(4), SExp::Atom(5), SExp::Atom(6)]),
    ]);

    let mut printer = Printer::new(Io(io::stdout()), 10);
    exp.print(&mut printer)?;
    printer.finish()?;

    Ok(())
}
