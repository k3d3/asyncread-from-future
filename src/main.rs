//! Test project to convert an `async`-defined future returning Vec<u8> into something
//! that can be used by an AsyncRead (and AsyncReadExt) consumer.
//!
//! Many thanks to Talchas on the Rust Discord for helping me figure this out!
//!
//! I release this code to the public domain, or under CCO where that's not possible.

use std::marker::PhantomData;
use futures::FutureExt;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, ReadBuf, Result},
    task::yield_now,
};

#[tokio::main]
async fn main() {
    // Just some tests with scoping to make sure everything drops properly
    let mut f = Fancy {};
    {
        let mut fr = f.reader();
        let mut buf = [0u8; 16];

        fr.read_exact(&mut buf[..4]).await.unwrap();

        fr.read_exact(&mut buf[8..12]).await.unwrap();

        assert_eq!(buf, [1,1,1,1,0,0,0,0,1,1,1,1,0,0,0,0]);
        println!("Success!");
    }

    {
        let mut fr = f.reader();
        let mut buf = [0u8; 16];

        fr.read_exact(&mut buf[..4]).await.unwrap();

        fr.read_exact(&mut buf[8..12]).await.unwrap();

        assert_eq!(buf, [1,1,1,1,0,0,0,0,1,1,1,1,0,0,0,0]);
        println!("Success!");
    }
}

/// Some arbitrary struct that has a read() future on it.
/// Realistically, this struct doesn't need to be here and
/// I could just use a plain async function instead for this
/// test, but this is closer to what I needed.
struct Fancy {}

impl Fancy {
    /// Some "fancy" reader that does some sort of work.
    async fn fancy_read(&mut self, size: usize) -> Vec<u8> {
        // Just so the future actually returns Poll::Pending at some point.
        yield_now().await;
        vec![1u8; size]
    }

    /// Return a type that implements AsyncRead.
    fn reader<'a>(&'a mut self) -> FancyReader<'a> {
        FancyReader {
            fancy: self,
            last_fut: None,
            _phantom: PhantomData
        }
    }
}

impl Drop for Fancy {
    fn drop(&mut self) {
        println!("Dropped Fancy");
    }
}

/// A type that implements AsyncRead. This holds a mutable reference to a parent object,
/// so that the parent object can't be used by anything else while the reader exists.
struct FancyReader<'a> {
    fancy: *mut Fancy,
    last_fut: Option<Pin<Box<dyn Future<Output = Vec<u8>> + 'static>>>, // reference to fancy, must not escape
    _phantom: PhantomData<&'a mut Fancy>
}

impl<'a> AsyncRead for FancyReader<'a> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        let fancy = self.fancy;
        let last_fut = self.last_fut.get_or_insert_with(|| {
            unsafe { &mut *fancy }.fancy_read(buf.remaining()).boxed()
        });
        match last_fut.poll_unpin(cx) {
            Poll::Ready(x) => {
                buf.put_slice(x.as_slice());
                self.last_fut = None;
                Poll::Ready(Ok(()))
            }
            Poll::Pending => {
                Poll::Pending
            },
        }
    }
}

impl<'a> Drop for FancyReader<'a> {
    fn drop(&mut self) {
        println!("Dropped FancyReader");
    }
}