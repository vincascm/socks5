use std::future::Future;

use bytes::{BufMut, Bytes, BytesMut};
use futures_lite::AsyncReadExt;

use crate::{
    error::{Error, Result},
    message::Replies,
    VERSION,
};

pub trait Encode {
    fn encode(&self) -> Bytes;

    fn as_bytes(&self) -> Bytes {
        let mut buffer = BytesMut::with_capacity(1);
        buffer.put_u8(VERSION);
        buffer.extend(self.encode());
        buffer.freeze()
    }
}

pub trait Decode<T: AsyncReadExt + Unpin>
where
    Self: Sized,
{
    fn decode(r: &mut T) -> impl Future<Output = Result<Self>>;

    fn read_u8(r: &mut T) -> impl Future<Output = Result<u8>> {
        async {
            let mut buf = vec![0; 1];
            r.read_exact(&mut buf).await?;
            Ok(buf[0])
        }
    }

    fn read(r: &mut T) -> impl Future<Output = Result<Self>> {
        async {
            let version = Self::read_u8(r).await?;
            if version != VERSION {
                return Err(Error::new(
                    Replies::ConnectionRefused,
                    format!("unsupported socks version {version:#x}"),
                ));
            }

            Self::decode(r).await
        }
    }
}
