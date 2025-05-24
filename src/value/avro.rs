use crate::error;
use crate::value;
use avro_rs;
use std;
use std::fmt;
use std::io;

pub struct Source<'a, R>(avro_rs::Reader<'a, R>)
where
    R: io::Read;

pub struct Sink<'a, W>(avro_rs::Writer<'a, W>)
where
    W: io::Write;

#[inline]
pub fn source<'a, R>(r: R) -> error::Result<Source<'a, R>>
where
    R: io::Read,
{
    Ok(Source(avro_rs::Reader::new(r)?))
}

#[inline]
pub fn sink<W>(schema: &avro_rs::Schema, w: W, codec: avro_rs::Codec) -> error::Result<Sink<W>>
where
    W: io::Write,
{
    Ok(Sink(avro_rs::Writer::with_codec(schema, w, codec)))
}

impl<'a, R> value::Source for Source<'a, R>
where
    R: io::Read,
{
    #[inline]
    fn read(&mut self) -> error::Result<Option<value::Value>> {
        match self.0.next() {
            Some(Ok(v)) => Ok(Some(value_from_avro(v))),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }
}

fn value_from_avro(value: avro_rs::types::Value) -> value::Value {
    use avro_rs::types::Value as From;
    use value::Value as To;
    match value {
        From::Null => To::Unit,
        From::Boolean(v) => To::Bool(v),
        From::Int(v) => To::I32(v),
        From::Long(v) => To::I64(v),
        From::Float(v) => To::from_f32(v),
        From::Double(v) => To::from_f64(v),
        From::Bytes(v) | From::Fixed(_, v) => To::Bytes(v),
        From::String(v) | From::Enum(_, v) => To::String(v),
        From::Union(boxed) => value_from_avro(*boxed),
        From::Array(v) => To::Sequence(v.into_iter().map(value_from_avro).collect()),
        From::Map(v) => To::Map(
            v.into_iter()
                .map(|(k, v)| (To::String(k), value_from_avro(v)))
                .collect(),
        ),
        From::Record(v) => To::Map(
            v.into_iter()
                .map(|(k, v)| (To::String(k), value_from_avro(v)))
                .collect(),
        ),
        From::Date(v) => todo!(),
        From::TimeMillis(v) => To::I32(v),
        From::TimeMicros(v) => To::I64(v),
        From::TimestampMillis(v) => To::I64(v),
        From::TimestampMicros(v) => To::I64(v),
        From::Duration(v) => To::from_f64(v.to_secs_f64()),
        From::Decimal(v) => todo!(),
        // Possibly, this might need its own value variant, because human-readable datatypes need different formatting. TODO
        From::Uuid(v) => To::Bytes(v.as_bytes().to_vec()),
    }
}

impl<'a, W> value::Sink for Sink<'a, W>
where
    W: io::Write,
{
    #[inline]
    fn write(&mut self, value: value::Value) -> error::Result<()> {
        self.0.append(value_to_avro(value)?)?;
        Ok(())
    }
}

fn value_to_avro(value: value::Value) -> error::Result<avro_rs::types::Value> {
    use avro_rs::types::Value;
    use std::convert::TryFrom;
    match value {
        value::Value::Unit => Ok(Value::Null),
        value::Value::Bool(v) => Ok(Value::Boolean(v)),

        value::Value::I8(v) => Ok(Value::Int(i32::from(v))),
        value::Value::I16(v) => Ok(Value::Int(i32::from(v))),
        value::Value::I32(v) => Ok(Value::Int(v)),
        value::Value::I64(v) => Ok(Value::Long(v)),

        value::Value::U8(v) => Ok(Value::Int(i32::from(v))),
        value::Value::U16(v) => Ok(Value::Int(i32::from(v))),
        value::Value::U32(v) => Ok(Value::Long(i64::from(v))),
        value::Value::U64(v) => {
            if let Ok(v) = i64::try_from(v) {
                Ok(Value::Long(v))
            } else {
                Err(error::Error::Format {
                    msg: format!(
                        "Avro output does not support unsigned 64 bit integer: {}",
                        v
                    ),
                })
            }
        }

        value::Value::F32(ordered_float::OrderedFloat(v)) => Ok(Value::Float(v)),
        value::Value::F64(ordered_float::OrderedFloat(v)) => Ok(Value::Double(v)),

        value::Value::Char(v) => Ok(Value::String(format!("{}", v))),
        value::Value::String(v) => Ok(Value::String(v)),
        value::Value::Bytes(v) => Ok(Value::Bytes(v)),

        value::Value::Sequence(v) => Ok(Value::Array(
            v.into_iter()
                .map(value_to_avro)
                .collect::<error::Result<Vec<_>>>()?,
        )),
        value::Value::Map(v) => Ok(Value::Record(
            v.into_iter()
                .map(|(k, v)| match (value_to_string(k), value_to_avro(v)) {
                    (Ok(k), Ok(v)) => Ok((k, v)),
                    (Ok(_), Err(e)) | (Err(e), Ok(_)) | (Err(_), Err(e)) => Err(e),
                })
                .collect::<error::Result<Vec<_>>>()?,
        )),
    }
}

fn value_to_string(value: value::Value) -> error::Result<String> {
    match value {
        value::Value::Char(v) => Ok(format!("{}", v)),
        value::Value::String(v) => Ok(v),
        x => Err(error::Error::Format {
            msg: format!("Avro can only output string keys, got: {:?}", x),
        }),
    }
}

impl<'a, R> fmt::Debug for Source<'a, R>
where
    R: io::Read,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AvroSource").finish()
    }
}

impl<'a, W> fmt::Debug for Sink<'a, W>
where
    W: io::Write,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AvroSink").finish()
    }
}

impl<'a, W> Drop for Sink<'a, W>
where
    W: io::Write,
{
    fn drop(&mut self) {
        match self.0.flush() {
            Ok(_) => (),
            Err(error) => panic!("{}", error),
        }
    }
}
