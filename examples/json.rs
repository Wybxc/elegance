use std::io;

use elegance::{
    core::Printer,
    render::{Io, Render},
};

enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
}

impl Value {
    pub fn print<R: Render>(&self, pp: &mut Printer<R>) -> Result<(), R::Error> {
        match self {
            Value::Null => pp.text("null")?,
            Value::Bool(true) => pp.text("true")?,
            Value::Bool(false) => pp.text("false")?,
            Value::Number(x) => pp.text(format!("{}", x))?,
            Value::String(s) => pp.text(format!("\"{}\"", s))?,
            Value::Array(arr) => pp.igroup(2, |pp| {
                pp.text("[")?;
                if let Some((first, rest)) = arr.split_first() {
                    pp.zero_break()?;
                    first.print(pp)?;
                    for v in rest {
                        pp.text(",")?;
                        pp.space()?;
                        v.print(pp)?;
                    }
                    pp.scan_break(0, -2)?;
                }
                pp.text("]")
            })?,
            Value::Object(obj) => pp.cgroup(2, |pp| {
                let mut obj = obj.iter();
                pp.text("{")?;
                if let Some((k, v)) = obj.next() {
                    pp.zero_break()?;
                    pp.text(format!("\"{}\": ", k))?;
                    v.print(pp)?;
                    for (k, v) in obj {
                        pp.text(",")?;
                        pp.space()?;
                        pp.text(format!("\"{}\": ", k))?;
                        v.print(pp)?;
                    }
                    pp.scan_break(0, -2)?;
                }
                pp.text("}")
            })?,
        };
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let obj = Value::Object(vec![
        ("name".into(), Value::String("hello".into())),
        ("age".into(), Value::Number(10.0)),
        ("is_ok".into(), Value::Bool(true)),
        ("null".into(), Value::Null),
        (
            "arr".into(),
            Value::Array((0..20).map(|i| Value::String(format!("{}", i))).collect()),
        ),
    ]);

    let mut printer = Printer::new(Io(io::stdout()), 40);
    obj.print(&mut printer)?;
    printer.finish()?;

    Ok(())
}
