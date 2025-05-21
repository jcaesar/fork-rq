use ansi_term;
use dtoa;

use crate::error;
use crate::value;
use itoa;
use serde;
use serde_json;
use std::fmt;
use std::io;
use std::str;

pub struct Source<'de, R>(
    serde_json::StreamDeserializer<'de, serde_json::de::IoRead<R>, value::Value>,
)
where
    R: io::Read;

pub struct Sink<W, F>(W, F)
where
    W: io::Write,
    F: Clone + serde_json::ser::Formatter;

#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct ReadableFormatter {
    current_indent: usize,
    is_in_object_key: bool,
    has_value: bool,

    null_style: ansi_term::Style,

    true_style: ansi_term::Style,
    false_style: ansi_term::Style,

    number_style: ansi_term::Style,

    string_quote_style: ansi_term::Style,
    string_char_style: ansi_term::Style,
    string_escape_style: ansi_term::Style,

    array_bracket_style: ansi_term::Style,
    array_comma_style: ansi_term::Style,

    object_brace_style: ansi_term::Style,
    object_colon_style: ansi_term::Style,
    object_comma_style: ansi_term::Style,
    object_key_quote_style: ansi_term::Style,
    object_key_char_style: ansi_term::Style,
    object_key_escape_style: ansi_term::Style,

    dtoa: dtoa::Buffer,
    itoa: itoa::Buffer,
}

#[inline]
pub fn source<'de, R>(r: R) -> Source<'de, R>
where
    R: io::Read,
{
    Source(serde_json::Deserializer::new(serde_json::de::IoRead::new(r)).into_iter())
}

#[inline]
pub fn sink_compact<W>(w: W) -> Sink<W, serde_json::ser::CompactFormatter>
where
    W: io::Write,
{
    Sink(w, serde_json::ser::CompactFormatter)
}

#[inline]
pub fn sink_readable<W>(w: W) -> Sink<W, ReadableFormatter>
where
    W: io::Write,
{
    Sink(w, ReadableFormatter::new())
}

#[inline]
pub fn sink_indented<'a, W>(w: W) -> Sink<W, serde_json::ser::PrettyFormatter<'a>>
where
    W: io::Write,
{
    Sink(w, serde_json::ser::PrettyFormatter::new())
}

impl<'de, R> value::Source for Source<'de, R>
where
    R: io::Read,
{
    #[inline]
    fn read(&mut self) -> error::Result<Option<value::Value>> {
        match self.0.next() {
            Some(Ok(v)) => Ok(Some(v)),
            Some(Err(e)) => Err(error::Error::from(e)),
            None => Ok(None),
        }
    }
}

impl<W, F> value::Sink for Sink<W, F>
where
    W: io::Write,
    F: Clone + serde_json::ser::Formatter,
{
    #[inline]
    fn write(&mut self, v: value::Value) -> error::Result<()> {
        {
            let mut serializer =
                serde_json::ser::Serializer::with_formatter(&mut self.0, self.1.clone());
            serde::Serialize::serialize(&v, &mut serializer)?;
        }
        self.0.write_all(b"\n")?;
        Ok(())
    }
}

impl ReadableFormatter {
    fn new() -> Self {
        use ansi_term::{Colour, Style};

        Self {
            current_indent: 0,
            is_in_object_key: false,
            has_value: false,

            null_style: Colour::Black.dimmed().bold().italic(),

            true_style: Colour::Green.bold().italic(),
            false_style: Colour::Red.bold().italic(),

            number_style: Colour::Blue.normal(),

            string_quote_style: Colour::Green.dimmed(),
            string_char_style: Colour::Green.normal(),
            string_escape_style: Colour::Green.dimmed(),

            array_bracket_style: Style::default().bold(),
            array_comma_style: Style::default().bold(),

            object_brace_style: Style::default().bold(),
            object_colon_style: Style::default().bold(),
            object_comma_style: Style::default().bold(),
            object_key_quote_style: Colour::Blue.dimmed(),
            object_key_char_style: Colour::Blue.normal(),
            object_key_escape_style: Colour::Blue.dimmed(),

            dtoa: dtoa::Buffer::new(),
            itoa: itoa::Buffer::new(),
        }
    }

    /// Writes an integer value like `-123` to the specified writer.
    #[inline]
    fn write_integer<W, I>(&mut self, writer: &mut W, value: I) -> io::Result<()>
    where
        W: io::Write + ?Sized,
        I: itoa::Integer,
    {
        write!(
            writer,
            "{}{}{}",
            self.number_style.prefix(),
            self.itoa.format(value),
            self.number_style.suffix(),
        )?;
        Ok(())
    }

    /// Writes a floating point value like `-31.26e+12` to the
    /// specified writer.
    #[inline]
    fn write_floating<W, F>(&mut self, writer: &mut W, value: F) -> io::Result<()>
    where
        W: io::Write + ?Sized,
        F: dtoa::Float,
    {
        write!(
            writer,
            "{}{}{}",
            self.number_style.prefix(),
            self.dtoa.format(value),
            self.number_style.suffix(),
        )?;
        Ok(())
    }
}

impl serde_json::ser::Formatter for ReadableFormatter {
    /// Writes a `null` value to the specified writer.
    #[inline]
    fn write_null<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        write!(writer, "{}", self.null_style.paint("null")).map_err(From::from)
    }

    /// Writes a `true` or `false` value to the specified writer.
    #[inline]
    fn write_bool<W>(&mut self, writer: &mut W, value: bool) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        let s = if value {
            self.true_style.paint("true")
        } else {
            self.false_style.paint("false")
        };
        write!(writer, "{}", s).map_err(From::from)
    }

    #[inline]
    fn write_i8<W>(&mut self, writer: &mut W, value: i8) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        self.write_integer(writer, value)
    }

    #[inline]
    fn write_i16<W>(&mut self, writer: &mut W, value: i16) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        self.write_integer(writer, value)
    }

    #[inline]
    fn write_i32<W>(&mut self, writer: &mut W, value: i32) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        self.write_integer(writer, value)
    }

    #[inline]
    fn write_i64<W>(&mut self, writer: &mut W, value: i64) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        self.write_integer(writer, value)
    }

    #[inline]
    fn write_u8<W>(&mut self, writer: &mut W, value: u8) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        self.write_integer(writer, value)
    }

    #[inline]
    fn write_u16<W>(&mut self, writer: &mut W, value: u16) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        self.write_integer(writer, value)
    }

    #[inline]
    fn write_u32<W>(&mut self, writer: &mut W, value: u32) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        self.write_integer(writer, value)
    }

    #[inline]
    fn write_u64<W>(&mut self, writer: &mut W, value: u64) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        self.write_integer(writer, value)
    }

    #[inline]
    fn write_f32<W>(&mut self, writer: &mut W, value: f32) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        self.write_floating(writer, value)
    }

    #[inline]
    fn write_f64<W>(&mut self, writer: &mut W, value: f64) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        self.write_floating(writer, value)
    }

    /// Called before each series of `write_string_fragment` and
    /// `write_char_escape`.  Writes a `"` to the specified writer.
    #[inline]
    fn begin_string<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        let style = if self.is_in_object_key {
            self.object_key_quote_style
        } else {
            self.string_quote_style
        };

        write!(writer, "{}", style.paint("\"")).map_err(From::from)
    }

    /// Called after each series of `write_string_fragment` and
    /// `write_char_escape`.  Writes a `"` to the specified writer.
    #[inline]
    fn end_string<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        let style = if self.is_in_object_key {
            self.object_key_quote_style
        } else {
            self.string_quote_style
        };

        write!(writer, "{}", style.paint("\"")).map_err(From::from)
    }

    /// Writes a string fragment that doesn't need any escaping to the
    /// specified writer.
    #[inline]
    fn write_string_fragment<W>(&mut self, writer: &mut W, fragment: &str) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        let style = if self.is_in_object_key {
            self.object_key_char_style
        } else {
            self.string_char_style
        };

        write!(writer, "{}", style.paint(fragment)).map_err(From::from)
    }

    /// Writes a character escape code to the specified writer.
    #[inline]
    fn write_char_escape<W>(
        &mut self,
        writer: &mut W,
        char_escape: serde_json::ser::CharEscape,
    ) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        use serde_json::ser::CharEscape::*;

        let style = if self.is_in_object_key {
            self.object_key_escape_style
        } else {
            self.string_escape_style
        };

        let s = match char_escape {
            Quote => "\\\"",
            ReverseSolidus => "\\\\",
            Solidus => "\\/",
            Backspace => "\\b",
            FormFeed => "\\f",
            LineFeed => "\\n",
            CarriageReturn => "\\r",
            Tab => "\\t",
            AsciiControl(byte) => {
                static HEX_DIGITS: [u8; 16] = *b"0123456789abcdef";
                let bytes = &[
                    b'\\',
                    b'u',
                    b'0',
                    b'0',
                    HEX_DIGITS[(byte >> 4) as usize],
                    HEX_DIGITS[(byte & 0xF) as usize],
                ];
                let s = unsafe { str::from_utf8_unchecked(bytes) };

                // Need to return early because of allocated String
                return write!(writer, "{}", style.paint(s)).map_err(From::from);
            }
        };

        write!(writer, "{}", style.paint(s)).map_err(From::from)
    }

    /// Called before every array.  Writes a `[` to the specified
    /// writer.
    #[inline]
    fn begin_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        self.current_indent += 1;
        self.has_value = false;

        write!(writer, "{}", self.array_bracket_style.paint("[")).map_err(From::from)
    }

    /// Called after every array.  Writes a `]` to the specified
    /// writer.
    #[inline]
    fn end_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        self.current_indent -= 1;

        if self.has_value {
            writeln!(writer)?;
            indent(writer, self.current_indent)?;
        }

        write!(writer, "{}", self.array_bracket_style.paint("]")).map_err(From::from)
    }

    /// Called before every array value.  Writes a `,` if needed to
    /// the specified writer.
    #[inline]
    fn begin_array_value<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        if !first {
            write!(writer, "{}", self.array_comma_style.paint(","))?;
        }

        writeln!(writer)?;
        indent(writer, self.current_indent)?;
        Ok(())
    }

    /// Called after every array value.
    #[inline]
    fn end_array_value<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        self.has_value = true;
        Ok(())
    }

    /// Called before every object.  Writes a `{` to the specified
    /// writer.
    #[inline]
    fn begin_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        self.current_indent += 1;
        self.has_value = false;

        write!(writer, "{}", self.object_brace_style.paint("{")).map_err(From::from)
    }

    /// Called after every object.  Writes a `}` to the specified
    /// writer.
    #[inline]
    fn end_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        self.current_indent -= 1;

        if self.has_value {
            writeln!(writer)?;
            indent(writer, self.current_indent)?;
        }

        write!(writer, "{}", self.object_brace_style.paint("}")).map_err(From::from)
    }

    /// Called before every object key.
    #[inline]
    fn begin_object_key<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        self.is_in_object_key = true;

        if !first {
            write!(writer, "{}", self.object_comma_style.paint(","))?;
        }

        writeln!(writer)?;
        indent(writer, self.current_indent)?;
        Ok(())
    }

    /// Called after every object key.  A `:` should be written to the
    /// specified writer by either this method or
    /// `begin_object_value`.
    #[inline]
    fn end_object_key<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        self.is_in_object_key = false;
        Ok(())
    }

    /// Called before every object value.  A `:` should be written to
    /// the specified writer by either this method or
    /// `end_object_key`.
    #[inline]
    fn begin_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        write!(writer, "{}", self.object_colon_style.paint(": ")).map_err(From::from)
    }

    /// Called after every object value.
    #[inline]
    fn end_object_value<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        self.has_value = true;
        Ok(())
    }
}

fn indent<W>(wr: &mut W, n: usize) -> io::Result<()>
where
    W: io::Write + ?Sized,
{
    for _ in 0..n {
        wr.write_all(b"  ")?;
    }

    Ok(())
}

impl<'de, R> fmt::Debug for Source<'de, R>
where
    R: io::Read,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CsvSource").finish()
    }
}

impl<W, F> fmt::Debug for Sink<W, F>
where
    W: io::Write,
    F: Clone + serde_json::ser::Formatter,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("JsonSink").finish()
    }
}
