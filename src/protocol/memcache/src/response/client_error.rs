// Copyright 2022 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::binary::response::header::ResponseStatus;
use crate::binary::Opcode;

use super::*;

const MSG_PREFIX: &[u8] = b"CLIENT_ERROR ";

#[derive(Debug, PartialEq, Eq)]
pub struct ClientError {
    pub(crate) inner: String,
}

impl ClientError {
    pub fn is_empty(&self) -> bool {
        false
    }

    pub fn len(&self) -> usize {
        MSG_PREFIX.len() + self.inner.len() + 2
    }

    pub fn write_binary_response(&self, opcode: Opcode, buffer: &mut dyn BufMut) -> usize {
        let mut header = ResponseStatus::InternalError.as_empty_response(opcode);
        let message = self.inner.as_bytes();
        header.total_body_len = message.len() as u32;
        header.write_to(buffer);
        buffer.put_slice(message);
        24 + message.len()
    }
}

impl Compose for ClientError {
    fn compose(&self, session: &mut dyn BufMut) -> usize {
        let msg = self.inner.as_bytes();

        let size = MSG_PREFIX.len() + msg.len() + CRLF.len();

        session.put_slice(MSG_PREFIX);
        session.put_slice(msg);
        session.put_slice(CRLF);

        size
    }
}

pub fn parse(input: &[u8]) -> IResult<&[u8], ClientError> {
    let (input, _) = space0(input)?;
    let (input, string) = not_line_ending(input)?;
    let (input, _) = crlf(input)?;
    Ok((
        input,
        ClientError {
            inner: unsafe { std::str::from_utf8_unchecked(string).to_owned() },
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        assert_eq!(
            response(b"CLIENT_ERROR Error message\r\n"),
            Ok((&b""[..], Response::client_error("Error message"),))
        );

        assert_eq!(
            response(b"CLIENT_ERROR\r\n"),
            Ok((&b""[..], Response::client_error(""),))
        );
    }
}
