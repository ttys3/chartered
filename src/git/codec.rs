#![allow(clippy::module_name_repetitions)]

use bytes::{Buf, Bytes, BytesMut};
use tokio_util::codec;

use super::PktLine;

pub struct Encoder {
    // buf: BytesMut,
}

impl codec::Encoder<PktLine<'_>> for Encoder {
    type Error = anyhow::Error;

    fn encode(&mut self, item: PktLine<'_>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        item.encode_to(dst)?;
        Ok(())
    }
}

#[derive(Default)]
pub struct GitCodec;

impl codec::Decoder for GitCodec {
    type Item = Bytes;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None);
        }

        let mut length_bytes = [0_u8; 4];
        length_bytes.copy_from_slice(&src[..4]);
        let length = u16::from_str_radix(std::str::from_utf8(&length_bytes)?, 16)? as usize;

        if length == 0 // flush-pkt
            || length == 1 // delim-pkt
            || length == 2
        // response-end-pkt
        {
            eprintln!("pkt: {}", length);
            src.advance(4);
            return self.decode(src);
        }

        if !(4..=65520).contains(&length) {
            return Err(
                std::io::Error::new(std::io::ErrorKind::InvalidData, "protocol abuse").into(),
            );
        }

        if src.len() < length {
            src.reserve(length - src.len());
            return Ok(None);
        }

        let mut bytes = src.split_to(length);
        bytes.advance(4);

        if bytes.ends_with(b"\n") {
            bytes.truncate(bytes.len() - 1);
        }

        Ok(Some(bytes.freeze()))
    }
}

#[cfg(test)]
mod test {
    use bytes::BytesMut;
    use std::fmt::Write;
    use tokio_util::codec::Decoder;

    #[test]
    fn decode() {
        let mut codec = super::GitCodec;

        let mut bytes = BytesMut::new();

        bytes.write_str("0015agent=git/2.32.0").unwrap();
        let res = codec.decode(&mut bytes).unwrap();
        assert_eq!(res, None);

        bytes.write_char('\n').unwrap();
        bytes.write_str("0002").unwrap();
        bytes.write_str("0004").unwrap();
        bytes.write_str("0005a").unwrap();

        let res = codec.decode(&mut bytes).unwrap();
        assert_eq!(res.as_deref(), Some("agent=git/2.32.0".as_bytes()));

        let res = codec.decode(&mut bytes).unwrap();
        assert_eq!(res.as_deref(), Some("".as_bytes()));

        let res = codec.decode(&mut bytes).unwrap();
        assert_eq!(res.as_deref(), Some("a".as_bytes()));

        let res = codec.decode(&mut bytes).unwrap();
        assert_eq!(res.as_deref(), None);
    }
}
