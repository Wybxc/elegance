use elegance::render::{Io, Render};
use elegance::Printer;
use std::cell::{Cell, RefCell};
use std::ffi::OsString;
use std::rc::Rc;

#[track_caller]
fn test_printer(f: impl FnOnce(&mut Printer) -> Result<(), std::convert::Infallible>, expected: &str) {
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
            pp.cgroup(2, |pp| {
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
            pp.cgroup(2, |pp| {
                pp.text("Hello,")?;
                pp.hard_break()?;
                pp.text("world!")
            })
        },
        "Hello,\n  world!",
    );
}

#[test]
fn test_igroup() {
    test_printer(
        |pp| {
            pp.igroup(2, |pp| {
                for _ in 0..40 {
                    pp.text("x")?;
                    pp.zero_break()?;
                }
                pp.text("x")?;
                pp.space()?;
                pp.cgroup(0, |pp| {
                    pp.text("Hello,")?;
                    pp.hard_break()?;
                    pp.text("world!")
                })
            })
        },
        &("x".repeat(40) + "\n  x Hello,\n  world!"),
    );
}

#[test]
fn test_text_overflow() {
    test_printer(
        |pp| {
            pp.text_owned("x".repeat(40))?;
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
            pp.cgroup(0, |pp| {
                pp.zero_break()?;
                pp.space()?;
                pp.hard_break()?;
                pp.hard_break()
            })
        },
        "\n\n\n\n",
    );
}

#[test]
fn test_break_indent() {
    test_printer(
        |pp| {
            pp.cgroup(2, |pp| {
                pp.zero_break()?;
                pp.text("Hello,")?;
                pp.scan_break(40, 2)?;
                pp.text("world!")
            })
        },
        "\n  Hello,\n    world!",
    );
}

#[test]
fn test_text_len_forces_wrap() {
    let mut pp = Printer::new(String::new(), 3);
    pp.cgroup(2, |pp| {
        pp.text("ab")?;
        pp.space()?;
        pp.text("cd")
    })
    .unwrap();

    assert_eq!(pp.finish().unwrap(), "ab\n  cd");
}

#[test]
fn test_osstring_renderer_spaces() {
    let mut pp = Printer::new(OsString::new(), 16);
    pp.text("hi")
        .and_then(|_| pp.space())
        .and_then(|_| pp.spaces(2))
        .and_then(|_| pp.text("there"))
        .unwrap();
    assert_eq!(pp.finish().unwrap(), OsString::from("hi   there"));
}

#[test]
fn test_io_renderer_writes_all() {
    let mut pp = Printer::new(Io(Vec::new()), 8);
    pp.text("foo")
        .and_then(|_| pp.space())
        .and_then(|_| pp.text("bar"))
        .unwrap();

    let Io(buf) = pp.finish().unwrap();
    assert_eq!(buf, b"foo bar");
}

#[test]
fn test_long_text_wraps_by_length() {
    let mut pp = Printer::new(String::new(), 5);
    pp.text("123456")
        .and_then(|_| pp.space())
        .and_then(|_| pp.text("x"))
        .unwrap();

    assert_eq!(pp.finish().unwrap(), "123456\nx");
}

#[derive(Clone)]
struct SharedRender {
    buf: Rc<RefCell<String>>,
    writes: Rc<Cell<usize>>,
}

impl Render for SharedRender {
    type Error = std::convert::Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.writes.set(self.writes.get() + 1);
        self.buf.borrow_mut().push_str(s);
        Ok(())
    }

    fn write_spaces(&mut self, n: usize) -> Result<(), Self::Error> {
        self.writes.set(self.writes.get() + 1);
        self.buf
            .borrow_mut()
            .extend(std::iter::repeat(' ').take(n));
        Ok(())
    }
}

struct NoZeroSpacesRender(String);

impl Render for NoZeroSpacesRender {
    type Error = std::convert::Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.0.push_str(s);
        Ok(())
    }

    fn write_spaces(&mut self, n: usize) -> Result<(), Self::Error> {
        if n == 0 {
            panic!("zero-space write should be skipped");
        }
        self.0.extend(std::iter::repeat(' ').take(n));
        Ok(())
    }
}

#[test]
fn test_prune_flushes_on_overflow() {
    let buf = Rc::new(RefCell::new(String::new()));
    let writes = Rc::new(Cell::new(0));
    let render = SharedRender {
        buf: buf.clone(),
        writes: writes.clone(),
    };

    let mut pp = Printer::new(render, 5);
    pp.text("123456").unwrap();

    assert!(writes.get() > 0, "prune should flush immediately");
    let renderer = pp.finish().unwrap();
    assert!(Rc::ptr_eq(&renderer.buf, &buf));
    assert_eq!(buf.borrow().as_str(), "123456");
}

#[test]
fn test_prune_not_triggered_on_exact_fit() {
    let mut pp = Printer::new(String::new(), 5);
    pp.cgroup(0, |pp| {
        pp.text("ab")?;
        pp.space()?;
        pp.text("cd")
    })
    .unwrap();

    assert_eq!(pp.finish().unwrap(), "ab cd");
}

#[cfg(feature = "unicode-width")]
#[test]
fn test_unicode_width_wraps() {
    let mut pp = Printer::new(String::new(), 3);
    pp.cgroup(0, |pp| {
        pp.text("你")?;
        pp.space()?;
        pp.text("好")
    })
    .unwrap();

    assert_eq!(pp.finish().unwrap(), "你\n好");
}

#[cfg(not(feature = "unicode-width"))]
#[test]
fn test_ascii_width_fallback_wraps() {
    let mut pp = Printer::new(String::new(), 3);
    pp.cgroup(0, |pp| {
        pp.text("abcd")?;
        pp.space()?;
        pp.text("e")
    })
    .unwrap();

    assert_eq!(pp.finish().unwrap(), "abcd\ne");
}

#[test]
fn test_no_zero_space_write() {
    let mut pp = Printer::new(NoZeroSpacesRender(String::new()), 10);
    pp.text("abc").unwrap();

    let NoZeroSpacesRender(out) = pp.finish().unwrap();
    assert_eq!(out, "abc");
}
