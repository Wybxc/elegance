//! Pretty printing arithmetic expressions with minimal parentheses.
//!
//! This example demonstrates using the `extra` field on `Printer` to track
//! operator precedence, allowing us to insert parentheses only when necessary.

use std::io;
use std::ops::{Add, Div, Mul, Neg, Sub};

use elegance::{
    core::Printer,
    render::{Io, Render},
};

/// Operator precedence levels.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Prec {
    /// Lowest precedence (top-level, no parens needed)
    Top = 0,
    /// Addition and subtraction
    AddSub = 1,
    /// Multiplication and division
    MulDiv = 2,
    /// Unary operators
    Unary = 3,
    /// Atomic expressions (literals, variables)
    Atom = 4,
}

/// Associativity of binary operators.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Assoc {
    Left,
    Right,
}

/// Arithmetic expressions.
enum Expr {
    /// Integer literal
    Int(i64),
    /// Variable
    Var(&'static str),
    /// Negation: -e
    Neg(Box<Expr>),
    /// Addition: e1 + e2
    Add(Box<Expr>, Box<Expr>),
    /// Subtraction: e1 - e2
    Sub(Box<Expr>, Box<Expr>),
    /// Multiplication: e1 * e2
    Mul(Box<Expr>, Box<Expr>),
    /// Division: e1 / e2
    Div(Box<Expr>, Box<Expr>),
}

impl Expr {
    /// Returns the precedence level of this expression.
    fn prec(&self) -> Prec {
        match self {
            Expr::Int(_) | Expr::Var(_) => Prec::Atom,
            Expr::Neg(_) => Prec::Unary,
            Expr::Mul(_, _) | Expr::Div(_, _) => Prec::MulDiv,
            Expr::Add(_, _) | Expr::Sub(_, _) => Prec::AddSub,
        }
    }

    /// Pretty print the expression.
    ///
    /// The `extra` field stores the current precedence context:
    /// - `extra.0`: the precedence of the surrounding context
    /// - `extra.1`: whether we're on the right side of a left-associative operator
    ///              (or left side of right-associative), which requires extra parens
    pub fn print<R: Render>(
        &self,
        pp: &mut Printer<R, String, (Prec, bool)>,
    ) -> Result<(), R::Error> {
        let (ctx_prec, needs_assoc_parens) = pp.extra;
        let my_prec = self.prec();

        // We need parentheses if:
        // 1. Our precedence is lower than the context, OR
        // 2. Our precedence equals the context AND we're in the "wrong" associative position
        let needs_parens = my_prec < ctx_prec
            || (my_prec == ctx_prec && my_prec != Prec::Atom && needs_assoc_parens);

        pp.cgroup(0, |pp| {
            if needs_parens {
                pp.text("(")?;
                pp.scan_break(0, 2)?;
            }

            match self {
                Expr::Int(n) => pp.text_owned(n.to_string())?,
                Expr::Var(name) => pp.text(*name)?,
                Expr::Neg(e) => {
                    pp.text("-")?;
                    pp.extra = (Prec::Unary, false);
                    e.print(pp)?;
                }
                Expr::Add(lhs, rhs) => {
                    self.print_binop(pp, lhs, "+", rhs, Prec::AddSub, Assoc::Left)?
                }
                Expr::Sub(lhs, rhs) => {
                    self.print_binop(pp, lhs, "-", rhs, Prec::AddSub, Assoc::Left)?
                }
                Expr::Mul(lhs, rhs) => {
                    self.print_binop(pp, lhs, "*", rhs, Prec::MulDiv, Assoc::Left)?
                }
                Expr::Div(lhs, rhs) => {
                    self.print_binop(pp, lhs, "/", rhs, Prec::MulDiv, Assoc::Left)?
                }
            }

            if needs_parens {
                pp.scan_break(0, 0)?;
                pp.text(")")?;
            }
            Ok(())
        })?;

        // Restore the context
        pp.extra = (ctx_prec, needs_assoc_parens);
        Ok(())
    }

    fn print_binop<R: Render>(
        &self,
        pp: &mut Printer<R, String, (Prec, bool)>,
        lhs: &Expr,
        op: &'static str,
        rhs: &Expr,
        prec: Prec,
        assoc: Assoc,
    ) -> Result<(), R::Error> {
        pp.igroup(0, |pp| {
            // Left operand: needs parens if we're right-associative
            pp.extra = (prec, assoc == Assoc::Right);
            lhs.print(pp)?;

            pp.space()?;
            pp.text(op)?;
            pp.scan_break(1, 2)?;

            // Right operand: needs parens if we're left-associative
            pp.extra = (prec, assoc == Assoc::Left);
            rhs.print(pp)
        })
    }
}

// Convenient expression construction via operator overloading
type E = Box<Expr>;

fn v(name: &'static str) -> E {
    Box::new(Expr::Var(name))
}

impl From<i64> for Box<Expr> {
    fn from(n: i64) -> Self {
        Box::new(Expr::Int(n))
    }
}

macro_rules! impl_op {
    ($($op: path, $f: ident, $c:ident);* $(;)?) => {
        $(  impl $op for Box<Expr> {
                type Output = Self;
                fn $f(self, rhs: Self) -> Self::Output {
                    Box::new(Expr::$c(self, rhs))
                }
            } )*
    };
}

impl_op! {
    Add, add, Add;
    Sub, sub, Sub;
    Mul, mul, Mul;
    Div, div, Div;
}
//

impl Neg for Box<Expr> {
    type Output = Self;
    fn neg(self) -> Self {
        Box::new(Expr::Neg(self))
    }
}

fn main() -> io::Result<()> {
    // Example expressions demonstrating minimal parenthesization.
    let (a, b, c, d, e, f) = (v("a"), v("b"), v("c"), v("d"), v("e"), v("f"));

    let examples: &[(&str, E)] = &[
        // 1 + 2 * 3  (no parens needed, precedence handles it)
        ("1 + 2 * 3", E::from(1) + E::from(2) * E::from(3)),
        // (1 + 2) * 3  (parens needed for lower precedence on left)
        ("(1 + 2) * 3", (E::from(1) + E::from(2)) * E::from(3)),
        // a - b - c  (left associative, no parens: means (a - b) - c)
        ("a - b - c (left assoc)", (v("a") - v("b")) - v("c")),
        // a - (b - c)  (parens needed on right for left-associative op)
        ("a - (b - c)", v("a") - (v("b") - v("c"))),
        // a / b * c  (left-to-right, same precedence)
        ("a / b * c", v("a") / v("b") * v("c")),
        // a / (b * c)  (parens needed)
        ("a / (b * c)", v("a") / (v("b") * v("c"))),
        // Complex: (a + b) * (c - d) / (e + f)
        ("(a + b) * (c - d) / (e + f)", (a + b) * (c - d) / (e + f)),
        // Negation: -a * b vs -(a * b)
        ("-a * b", -v("a") * v("b")),
        ("-(a * b)", -(v("a") * v("b"))),
        // Very long expression that will wrap
        (
            "long expr",
            v("alpha") * v("beta")
                + v("gamma") * v("delta")
                + v("epsilon") * v("zeta")
                + v("eta") / v("theta"),
        ),
        // Line wrap with parenthesized subexpression
        (
            "wrap with parens",
            v("foo") * (v("alpha") + v("beta") + v("gamma") + v("delta"))
                / (v("epsilon") - v("zeta")),
        ),
    ];

    for (desc, expr) in examples {
        println!("{}:", desc);

        // Create printer with extra = (Prec::Top, false) for top-level context
        let mut printer = Printer::new_extra(Io(io::stdout()), 40, (Prec::Top, false));
        expr.print(&mut printer)?;
        printer.finish()?;
        println!();
        println!();
    }

    Ok(())
}
