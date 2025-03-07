use std::ops::Deref;

/// A borrowed or owned string.
pub enum CowString<'a, T: AsRef<str> = String> {
    Borrowed(&'a str),
    Owned(T),
}

impl<T: AsRef<str>> Deref for CowString<'_, T> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            CowString::Borrowed(b) => b,
            CowString::Owned(o) => o.as_ref(),
        }
    }
}
