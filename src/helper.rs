use std::borrow::Cow;

use crate::{core::Printer, render::Render};

impl<'a, R: Render> Printer<'a, R> {
    #[inline]
    pub fn text(&mut self, text: impl Into<Cow<'a, str>>) -> Result<(), R::Error> {
        self.scan_text(text.into())
    }

    #[inline]
    pub fn hard_break(&mut self) -> Result<(), R::Error> {
        self.scan_break(65536, 0)
    }

    #[inline]
    pub fn soft_break(&mut self) -> Result<(), R::Error> {
        self.scan_break(1, 0)
    }

    #[inline]
    pub fn zero_break(&mut self) -> Result<(), R::Error> {
        self.scan_break(0, 0)
    }

    #[inline]
    pub fn group(
        &mut self,
        indent: isize,
        f: impl FnOnce(&mut Self) -> Result<(), R::Error>,
    ) -> Result<(), R::Error> {
        self.scan_begin(indent);
        f(self)?;
        self.scan_end()
    }
}
