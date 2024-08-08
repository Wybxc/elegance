//! The core implementation of the printer.
//!
//! The algorithm is mostly based on the Haskell implementation in the paper
//! "Linear, bounded, functional pretty-printing" by O. Chitil.

use std::{
    borrow::Cow,
    collections::VecDeque,
    ops::{AddAssign, Sub},
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
    type Output = usize;

    fn sub(self, rhs: Position) -> Self::Output {
        debug_assert!(self.0 >= rhs.0);
        self.0 - rhs.0
    }
}

enum Token<'a> {
    Text(Cow<'a, str>),
    Break { indent: usize },
    Group(OutGroup<'a>),
}

struct OutGroup<'a> {
    tokens: Vec<(Token<'a>, usize)>,
}

#[derive(Clone, Copy)]
enum RenderFrame {
    Fits,
    Break { consistent: bool },
}

/// The `Printer` is a pretty printing engine. It takes a sequence of layout elements and
/// produces a pretty printed representation of the elements.
pub struct Printer<'a, R: Render = String> {
    // common
    line_width: usize,

    // scanner
    position: Position,
    indent: Vec<isize>,
    dq: VecDeque<(Position, OutGroup<'a>)>,

    // renderer
    renderer: R,
    remaining: usize,
    render_stack: Vec<RenderFrame>,
    pending_indent: usize,
}

impl<'a, R: Render> Printer<'a, R> {
    /// Create a new printer.
    ///
    /// # Panics
    ///
    /// If line width is not between 1 and 65536.
    pub fn new(renderer: R, line_width: usize) -> Self {
        assert!(
            line_width > 0 && line_width <= Self::MAX_WIDTH,
            "line width must be between 1 and {}",
            Self::MAX_WIDTH
        );
        let mut pp = Self {
            line_width,
            position: Position(0),
            indent: vec![0],
            dq: VecDeque::new(),
            renderer,
            remaining: line_width,
            render_stack: Vec::new(),
            pending_indent: 0,
        };
        pp.scan_begin(0);
        pp
    }

    /// Maximum line width.
    pub const MAX_WIDTH: usize = 65536;

    /// Write a text element.
    pub fn scan_text(&mut self, text: Cow<'a, str>, width: usize) -> Result<(), R::Error> {
        self.scan(width, Token::Text(text))
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
        self.scan(size, Token::Break { indent })
    }

    /// Begin a group.
    pub fn scan_begin(&mut self, indent: isize) {
        self.indent.push(self.indent() + indent);
        self.dq.push_back((
            self.position,
            OutGroup {
                tokens: Vec::with_capacity(12),
            },
        ));
    }

    /// End a group.
    pub fn scan_end(&mut self) -> Result<(), R::Error> {
        self.indent.pop();
        if let Some((s, grp1)) = self.dq.pop_back() {
            let width = self.position - s;
            if let Some((_, grp2)) = self.dq.back_mut() {
                grp2.tokens.push((Token::Group(grp1), width));
            } else {
                self.render_begin(grp1, width)?;
                self.render_end()?;
            }
        } else {
            self.render_end()?;
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

    fn scan(&mut self, width: usize, out: Token<'a>) -> Result<(), R::Error> {
        self.position += width;
        if let Some((_, grp)) = self.dq.back_mut() {
            grp.tokens.push((out, width));
            self.prune()?;
        } else {
            self.render_token(out, width)?;
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
            self.render_begin(grp, Self::MAX_WIDTH)?;
        }
        Ok(())
    }

    fn render_token(&mut self, token: Token<'a>, width: usize) -> Result<(), R::Error> {
        match token {
            Token::Text(text) => self.render_text(text, width),
            Token::Break { indent } => self.render_break(indent, width),
            Token::Group(group) => {
                self.render_begin(group, width)?;
                self.render_end()
            }
        }
    }

    fn render_text(&mut self, text: Cow<'a, str>, width: usize) -> Result<(), R::Error> {
        if self.pending_indent > 0 {
            self.renderer.write_spaces(self.pending_indent)?;
            self.pending_indent = 0;
        }
        self.renderer.write_str(&text)?;
        self.remaining = self.remaining.saturating_sub(width);
        Ok(())
    }

    fn render_break(&mut self, indent: usize, width: usize) -> Result<(), R::Error> {
        let frame = self
            .render_stack
            .last()
            .copied()
            .unwrap_or(RenderFrame::Break { consistent: false });
        let fits = match frame {
            RenderFrame::Fits => true,
            RenderFrame::Break { consistent, .. } => !consistent && width <= self.remaining,
        };
        if fits {
            self.renderer.write_spaces(width)?;
            self.remaining = self.remaining.saturating_sub(width);
        } else {
            self.renderer.write_str("\n")?;
            self.pending_indent = indent;
            self.remaining = self.line_width.saturating_sub(indent);
        }
        Ok(())
    }

    fn render_begin(&mut self, group: OutGroup<'a>, width: usize) -> Result<(), R::Error> {
        self.render_stack.push(if width <= self.remaining {
            RenderFrame::Fits
        } else {
            RenderFrame::Break { consistent: true }
        });
        for (out, width) in group.tokens {
            self.render_token(out, width)?;
        }
        Ok(())
    }

    fn render_end(&mut self) -> Result<(), R::Error> {
        self.render_stack.pop();
        Ok(())
    }
}
