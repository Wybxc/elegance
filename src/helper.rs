use std::borrow::Cow;

use crate::{core::Printer, render::Render};

impl<'a, R: Render> Printer<'a, R> {
    /// Write a text element.
    ///
    /// ```
    /// # use elegance::Printer;
    /// let mut pp = Printer::new(String::new(), 40);
    /// pp.text("Hello, world!")?;
    /// assert_eq!(pp.finish()?, "Hello, world!");
    /// # Ok::<(), ()>(())
    /// ```
    #[inline]
    pub fn text(&mut self, text: impl Into<Cow<'a, str>>) -> Result<(), R::Error> {
        let text = text.into();
        let width = text.len();
        self.scan_text(text, width)
    }

    /// Write a hard line break.
    ///
    /// ```
    /// # use elegance::Printer;
    /// let mut pp = Printer::new(String::new(), 40);
    /// pp.text("Hello,")?;
    /// pp.hard_break()?;
    /// pp.text("world!")?;
    /// assert_eq!(pp.finish()?, "Hello,\nworld!");
    /// # Ok::<(), ()>(())
    /// ```
    #[inline]
    pub fn hard_break(&mut self) -> Result<(), R::Error> {
        self.scan_break(Self::MAX_WIDTH, 0)
    }

    /// Write a zero-width line break.
    ///
    /// ```
    /// # use elegance::Printer;
    /// let mut pp = Printer::new(String::new(), 40);
    /// pp.text("Hello,")?;
    /// pp.zero_break()?;
    /// pp.text("world!")?;
    /// assert_eq!(pp.finish()?, "Hello,world!");
    /// # Ok::<(), ()>(())
    /// ```
    #[inline]
    pub fn zero_break(&mut self) -> Result<(), R::Error> {
        self.scan_break(0, 0)
    }

    /// Write a number of spaces.
    ///
    /// ```
    /// # use elegance::Printer;
    /// let mut pp = Printer::new(String::new(), 40);
    /// pp.text("Hello,")?;
    /// pp.spaces(2)?;
    /// pp.text("world!")?;
    /// assert_eq!(pp.finish()?, "Hello,  world!");
    /// # Ok::<(), ()>(())
    /// ```
    #[inline]
    pub fn spaces(&mut self, n: usize) -> Result<(), R::Error> {
        self.scan_break(n, 0)
    }

    /// Write a space (soft line break).
    ///
    /// ```
    /// # use elegance::Printer;
    /// let mut pp = Printer::new(String::new(), 40);
    /// pp.text("Hello,")?;
    /// pp.space()?;
    /// pp.text("world!")?;
    /// assert_eq!(pp.finish()?, "Hello, world!");
    /// # Ok::<(), ()>(())
    /// ```
    #[inline]
    pub fn space(&mut self) -> Result<(), R::Error> {
        self.scan_break(1, 0)
    }

    /// Write a group.
    ///
    /// ```
    /// # use elegance::Printer;
    /// let mut pp = Printer::new(String::new(), 40);
    /// pp.group(2, |pp| {
    ///     pp.text("Hello,")?;
    ///     pp.hard_break()?;
    ///     pp.text("world!")?;
    ///     Ok(())
    /// })?;
    /// assert_eq!(pp.finish()?, "Hello,\n  world!");
    /// # Ok::<(), ()>(())
    /// ```
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
