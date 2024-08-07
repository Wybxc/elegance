//! The core implementation of the printer.
//! 
//! The algorithm is mostly based on the Haskell implementation in the paper 
//! "Linear, bounded, functional pretty-printing" by O. Chitil.

use std::{
    borrow::Cow,
    collections::VecDeque,
    ops::{AddAssign, Sub, SubAssign},
};

use crate::render::Render;

#[derive(Clone, Copy)]
struct Position(pub usize);

impl AddAssign<usize> for Position {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs
    }
}

impl Sub<Position> for Position {
    type Output = Width;

    fn sub(self, rhs: Position) -> Self::Output {
        Width(self.0 as isize - rhs.0 as isize)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Width(pub isize);

impl Sub<usize> for Width {
    type Output = Width;

    fn sub(self, rhs: usize) -> Self::Output {
        Width(self.0 - rhs as isize)
    }
}

impl SubAssign<usize> for Width {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs as isize
    }
}

enum Out<'a> {
    Text(Cow<'a, str>),
    Break { size: usize, indent: usize },
    Group { size: Width, out: OutGroup<'a> },
}

struct OutGroup<'a> {
    outs: Vec<Out<'a>>,
}

/// The `Printer` is a pretty printing engine. It takes a sequence of layout elements and
/// produces a pretty printed representation of the elements.
pub struct Printer<'a, R: Render = String> {
    renderer: R,
    width: Width,
    position: Position,
    remaining: Width,
    indent: Vec<isize>,
    pending_indent: usize,
    dq: VecDeque<(Position, OutGroup<'a>)>,
}

impl<'a, R: Render> Printer<'a, R> {
    /// Create a new printer.
    ///
    /// # Panics
    ///
    /// If the width is not between 1 and 65536.
    pub fn new(renderer: R, width: usize) -> Self {
        assert!(
            width > 0 && width <= Self::MAX_WIDTH,
            "the width must be between 1 and {}",
            Self::MAX_WIDTH
        );
        let width = Width(width.try_into().unwrap());
        let mut pp = Self {
            renderer,
            width,
            position: Position(0),
            remaining: width,
            indent: vec![0],
            pending_indent: 0,
            dq: VecDeque::new(),
        };
        pp.scan_begin(0);
        pp
    }

    /// Maximum line width.
    pub const MAX_WIDTH: usize = 65536;

    /// Write a text element.
    pub fn scan_text(&mut self, text: Cow<'a, str>) -> Result<(), R::Error> {
        self.scan(text.len(), Out::Text(text))
    }

    /// Write a break element.
    ///
    /// A break is `size` spaces if there is enough space, or a new line if not.
    ///
    /// After line break, the indent is increased by `indent`. The value can be
    /// negative, in which case the indent is decreased.
    ///
    /// # Panics
    ///
    /// If the total indent is negative.
    pub fn scan_break(&mut self, size: usize, indent: isize) -> Result<(), R::Error> {
        let indent = (self.indent() + indent)
            .try_into()
            .expect("indent must >= 0");
        self.scan(size, Out::Break { size, indent })
    }

    /// Begin a group.
    pub fn scan_begin(&mut self, indent: isize) {
        self.indent.push(self.indent() + indent);
        self.dq
            .push_back((self.position, OutGroup { outs: Vec::with_capacity(12) }));
    }

    /// End a group.
    pub fn scan_end(&mut self) -> Result<(), R::Error> {
        self.indent.pop();
        if let Some((s, grp1)) = self.dq.pop_back() {
            if let Some((_, grp2)) = self.dq.back_mut() {
                grp2.outs.push(Out::Group {
                    size: self.position - s,
                    out: grp1,
                });
            } else {
                self.print_group(grp1, true)?;
            }
        }
        Ok(())
    }

    /// Finish the printer and return the result.
    ///
    /// # Panics
    ///
    /// If there is an unclosed group.
    pub fn finish(mut self) -> Result<R, R::Error> {
        self.scan_end()?;
        assert!(self.dq.is_empty(), "unclosed group");
        Ok(self.renderer)
    }

    fn indent(&self) -> isize {
        *self.indent.last().unwrap()
    }

    fn scan(&mut self, length: usize, out: Out<'a>) -> Result<(), R::Error> {
        self.position += length;
        if let Some((_, grp)) = self.dq.back_mut() {
            grp.outs.push(out);
            self.prune()?;
        } else {
            self.print(out, false)?;
        }
        Ok(())
    }

    fn prune(&mut self) -> Result<(), R::Error> {
        while self
            .dq
            .front()
            .is_some_and(|&(s, _)| self.position - s > self.remaining)
        {
            let (_, grp) = self.dq.pop_front().unwrap();
            self.print_group(grp, false)?;
        }
        Ok(())
    }

    fn print_group(&mut self, group: OutGroup<'a>, horizontal: bool) -> Result<(), R::Error> {
        for out in group.outs {
            self.print(out, horizontal)?;
        }
        Ok(())
    }

    fn print(&mut self, group: Out<'a>, horizontal: bool) -> Result<(), R::Error> {
        match group {
            Out::Text(text) => {
                if self.pending_indent > 0 {
                    self.renderer.write_spaces(self.pending_indent)?;
                    self.pending_indent = 0;
                }
                self.renderer.write_str(&text)?;
                self.remaining -= text.len();
            }
            Out::Break { size, indent } => {
                if horizontal {
                    self.renderer.write_spaces(size)?;
                    self.remaining -= size;
                } else {
                    self.renderer.write_str("\n")?;
                    self.pending_indent = indent;
                    self.remaining = self.width - indent;
                }
            }
            Out::Group { size, out } => {
                let horizontal = self.remaining >= size;
                self.print_group(out, horizontal)?;
            }
        }
        Ok(())
    }
}
