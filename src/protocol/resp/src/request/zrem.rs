// Copyright 2025 Pelikan Foundation LLC.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::io::Error;
use std::sync::Arc;

use super::*;

#[metric(name = "zrem")]
pub static ZREM: Counter = Counter::new();

#[metric(name = "zrem_ex")]
pub static ZREM_EX: Counter = Counter::new();

#[derive(Debug, PartialEq, Eq)]
pub struct SortedSetRemove {
    key: Arc<[u8]>,
    members: Vec<Arc<[u8]>>,
}

impl TryFrom<Message> for SortedSetRemove {
    type Error = Error;

    fn try_from(other: Message) -> Result<Self, Error> {
        let array = match other {
            Message::Array(array) => array,
            _ => return Err(Error::new(ErrorKind::Other, "malformed command")),
        };

        if array.inner.is_none() {
            return Err(Error::new(ErrorKind::Other, "malformed command"));
        }

        let mut array = array.inner.unwrap();

        if array.len() < 3 {
            return Err(Error::new(ErrorKind::Other, "malformed command"));
        }

        let _command = take_bulk_string(&mut array)?;
        let key = take_bulk_string(&mut array)?
            .ok_or_else(|| Error::new(ErrorKind::Other, "malformed command"))?;

        let mut members = Vec::with_capacity(array.len());

        while let Some(member) = take_bulk_string(&mut array)? {
            if member.is_empty() {
                return Err(Error::new(ErrorKind::Other, "malformed command"));
            }
            members.push(member);
        }

        Ok(Self { key, members })
    }
}

impl SortedSetRemove {
    pub fn new(key: &[u8], members: &[&[u8]]) -> Self {
        Self {
            key: key.into(),
            members: members.iter().map(|m| (*m).into()).collect(),
        }
    }

    pub fn key(&self) -> &[u8] {
        &self.key
    }

    pub fn members(&self) -> &[Arc<[u8]>] {
        &self.members
    }
}

impl From<&SortedSetRemove> for Message {
    fn from(value: &SortedSetRemove) -> Message {
        Message::Array(Array {
            inner: Some(vec![
                Message::BulkString(BulkString::new(b"ZREM")),
                Message::BulkString(BulkString::new(value.key())),
                Message::Array(Array {
                    inner: Some(
                        value
                            .members
                            .iter()
                            .map(|m| Message::BulkString(BulkString::new(m)))
                            .collect(),
                    ),
                }),
            ]),
        })
    }
}

impl Compose for SortedSetRemove {
    fn compose(&self, buf: &mut dyn BufMut) -> usize {
        Message::from(self).compose(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser() {
        let parser = RequestParser::new();
        assert_eq!(
            parser.parse(b"ZREM z a b c\r\n").unwrap().into_inner(),
            Request::SortedSetRemove(SortedSetRemove::new(b"z", &[b"a", b"b", b"c"]))
        );

        assert_eq!(
            parser.parse(b"ZREM z a b c\r\n").unwrap().into_inner(),
            Request::SortedSetRemove(SortedSetRemove::new(b"z", &[b"a", b"b", b"c"]))
        );

        assert_eq!(
            parser
                .parse(b"*5\r\n$4\r\nZREM\r\n$1\r\nz\r\n$1\r\na\r\n$1\r\nb\r\n$1\r\nc\r\n")
                .unwrap()
                .into_inner(),
            Request::SortedSetRemove(SortedSetRemove::new(b"z", &[b"a", b"b", b"c"]))
        );
    }
}
