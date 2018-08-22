use std::io;


use serde::{Serialize};
use serde_json::ser::{Formatter, PrettyFormatter, Serializer};

pub fn to_string_escaped<V: Serialize>(v: &V) -> String {
    let mut serializer = Serializer::with_formatter(
        Vec::<u8>::new(),
        EscapingFormatter(PrettyFormatter::new())
    );
    v.serialize(&mut serializer).unwrap();
    String::from_utf8(serializer.into_inner()).unwrap()
}

struct EscapingFormatter<F>(pub F);
impl<F: Formatter> Formatter for EscapingFormatter<F> {
    fn write_string_fragment<W: ?Sized>(&mut self, writer: &mut W, fragment: &str) -> io::Result<()> where
        W: io::Write, {
        for c in fragment.chars() {
            if c.is_ascii() {
                let mut buffer = [0u8; 4];
                writer.write_all(c.encode_utf8(&mut buffer).as_bytes())?;
            } else {
                let mut buffer = [0u16; 2];
                // Escape all unicode into their UTF16 representation(s)
                for &escaped in c.encode_utf16(&mut buffer).iter() {
                    write!(writer, "\\u{:04X}", escaped)?;
                }
            }
        }
        Ok(())
    }
    // Delegates
    #[inline]
    fn begin_array<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
        where
            W: io::Write,
    {
        self.0.begin_array(writer)
    }

    #[inline]
    fn end_array<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
        where
            W: io::Write,
    {
        self.0.end_array(writer)
    }

    #[inline]
    fn begin_array_value<W: ?Sized>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
        where
            W: io::Write,
    {
        self.0.begin_array_value(writer, first)
    }

    #[inline]
    fn end_array_value<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
        where
            W: io::Write,
    {
        self.0.end_array_value(writer)
    }

    #[inline]
    fn begin_object<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
        where
            W: io::Write,
    {
        self.0.begin_object(writer)
    }

    #[inline]
    fn end_object<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
        where
            W: io::Write,
    {

        self.0.end_object(writer,)
    }

    #[inline]
    fn begin_object_key<W: ?Sized>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
        where
            W: io::Write,
    {
        self.0.begin_object_key(writer, first)
    }

    #[inline]
    fn begin_object_value<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
        where
            W: io::Write, {
        self.0.begin_object_value(writer)
    }

    #[inline]
    fn end_object_value<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
        where
            W: io::Write,
    {
        self.0.end_object_value(writer)
    }
}