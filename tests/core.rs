use elegance::Printer;

#[track_caller]
fn test_printer(f: impl FnOnce(&mut Printer) -> Result<(), ()>, expected: &str) {
    let mut pp = Printer::new(String::new(), 40);
    f(&mut pp).unwrap();
    assert_eq!(pp.finish().unwrap(), expected);
}

#[test]
fn test_text() {
    test_printer(|pp| pp.text("Hello, world!"), "Hello, world!");
}

#[test]
fn test_space() {
    test_printer(|pp| pp.space(), " ");
}

#[test]
fn test_spaces() {
    test_printer(|pp| pp.spaces(5), "     ");
}

#[test]
fn test_hard_break() {
    test_printer(|pp| pp.hard_break(), "\n");
}

#[test]
fn test_zero_break() {
    test_printer(|pp| pp.zero_break(), "");
}

#[test]
fn test_group_horizontal() {
    test_printer(
        |pp| {
            pp.group(2, |pp| {
                pp.text("Hello,")?;
                pp.space()?;
                pp.text("world!")
            })
        },
        "Hello, world!",
    );
}

#[test]
fn test_group_vertical() {
    test_printer(
        |pp| {
            pp.group(2, |pp| {
                pp.text("Hello,")?;
                pp.hard_break()?;
                pp.text("world!")
            })
        },
        "Hello,\n  world!",
    );
}

#[test]
fn test_text_overflow() {
    test_printer(
        |pp| {
            pp.text("x".repeat(40))?;
            pp.zero_break()?;
            pp.text("Hello,world!")
        },
        &("x".repeat(40) + "\nHello,world!"),
    );
}

#[test]
fn test_multiple_newlines() {
    test_printer(
        |pp| {
            pp.zero_break()?;
            pp.space()?;
            pp.hard_break()?;
            pp.hard_break()
        },
        "\n\n\n\n",
    );
}

#[test]
fn test_break_indent() {
    test_printer(
        |pp| {
            pp.group(2, |pp| {
                pp.zero_break()?;
                pp.text("Hello,")?;
                pp.scan_break(40, 2)?;
                pp.text("world!")
            })
        },
        "\n  Hello,\n    world!",
    );
}
