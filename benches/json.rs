use criterion::{black_box, criterion_group, criterion_main, Criterion};
use elegance::{Printer, Render};
use serde_json::Value;

fn escape_string(s: &str) -> String {
    serde_json::to_string(s).unwrap()
}

pub fn print_json<'a, R: Render>(json: &'a Value, pp: &mut Printer<'a, R>) -> Result<(), R::Error> {
    match json {
        Value::Null => pp.text("null")?,
        Value::Bool(true) => pp.text("true")?,
        Value::Bool(false) => pp.text("false")?,
        Value::Number(n) => pp.text(n.to_string())?,
        Value::String(s) => pp.text(escape_string(s))?,
        Value::Array(arr) => pp.igroup(2, |pp| {
            pp.text("[")?;
            if let Some((first, rest)) = arr.split_first() {
                pp.zero_break()?;
                print_json(first, pp)?;
                for v in rest {
                    pp.text(",")?;
                    pp.space()?;
                    print_json(v, pp)?;
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
                pp.text(escape_string(k))?;
                pp.text(": ")?;
                print_json(v, pp)?;
                for (k, v) in obj {
                    pp.text(",")?;
                    pp.space()?;
                    pp.text(escape_string(k))?;
                    pp.text(": ")?;
                    print_json(v, pp)?;
                }
                pp.scan_break(0, -2)?;
            }
            pp.text("}")
        })?,
    }
    Ok(())
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let obj: Value = black_box(
        serde_json::from_reader(
            std::fs::File::open("benches/data.json").expect("failed to open data.json"),
        )
        .expect("failed to parse data.json"),
    );

    c.bench_function("string", |b| {
        b.iter(|| {
            let mut pp = Printer::new(String::new(), 40);
            print_json(&obj, &mut pp).unwrap();
            pp.finish().unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
