use std::{ffi::OsString, io, iter};

pub trait Render {
    type Error;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error>;
    fn write_spaces(&mut self, n: usize) -> Result<(), Self::Error> {
        self.write_str(&" ".repeat(n))
    }
}

impl Render for String {
    type Error = ();

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.push_str(s);
        Ok(())
    }

    fn write_spaces(&mut self, n: usize) -> Result<(), Self::Error> {
        self.reserve(n);
        self.extend(iter::repeat(' ').take(n));
        Ok(())
    }
}

impl Render for OsString {
    type Error = ();

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.push(s);
        Ok(())
    }

    fn write_spaces(&mut self, n: usize) -> Result<(), Self::Error> {
        self.reserve(n);
        (0..n).for_each(|_| self.push(" "));
        Ok(())
    }
}

pub struct Io<W: io::Write>(pub W);

impl<W: io::Write> Render for Io<W> {
    type Error = io::Error;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.0.write_all(s.as_bytes())?;
        Ok(())
    }
}
