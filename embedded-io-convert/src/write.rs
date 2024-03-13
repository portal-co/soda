use std::error::Error;
use std::task::Poll;
use std::{pin::Pin, task::Context};
use std::io;

use embedded_io_async::Write;
use futures::{Future, AsyncRead, AsyncWrite};
use pin_project::pin_project;

use crate::MutexFuture;

#[pin_project]
pub struct SimpleAsyncWriter<R>
where
    R: embedded_io_async::Write,
{
    state: State<R>,
}
impl<R: Write> SimpleAsyncWriter<R>{
    pub fn new(r: R) -> Self{
        return Self{state: State::Idle(r)};
    }
}
type BoxFut<T> = Pin<Box<dyn Future<Output = T> + Send + Sync>>;

enum State<R> {
    Idle(R),
    Pending(BoxFut<(R, io::Result<usize>)>),
    Transitional,
}
impl<R> AsyncWrite for SimpleAsyncWriter<R>
where
    // new: R must now be `'static`, since it's captured
    // by the future which is, itself, `'static`.
    R: embedded_io_async::Write + Send + Sync + 'static,
    R::Error: Into<std::io::Error> + Send + Sync + 'static,
{
    fn poll_write(
                self: Pin<&mut Self>,
                cx: &mut Context<'_>,
                buf: &[u8],
            ) -> Poll<io::Result<usize>> {
                let proj = self.project();
        let mut state = State::Transitional;
        std::mem::swap(proj.state, &mut state);
let buf = buf.to_vec();
        let mut fut = match state {
            State::Idle(mut inner) => {

                Box::pin(async move {
                    let res = MutexFuture::new(inner.write(&buf)).await;
                    (inner,  res.map_err(|e|e.into()))
                })
            }
            State::Pending(fut) => {
                // tracing::debug!("polling existing future...");
                fut
            }
            State::Transitional => unreachable!(),
        };

        match fut.as_mut().poll(cx) {
            Poll::Ready((inner, result)) => {
                // tracing::debug!("future was ready!");
                // if let Ok(n) = &result {
                //     let n = *n;
                //     // unsafe { internal_buf.set_len(n) }

                //     // let dst = &mut buf[..n];
                //     // let src = &internal_buf[..];
                //     // dst.copy_from_slice(src);
                // } else {
                //     // unsafe { internal_buf.set_len(0) }
                // }
                *proj.state = State::Idle(inner);
                Poll::Ready(result)
            }
            Poll::Pending => {
                // tracing::debug!("future was pending!");
                *proj.state = State::Pending(fut);
                Poll::Pending
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let proj = self.project();
        let mut state = State::Transitional;
        std::mem::swap(proj.state, &mut state);
// let buf = buf.to_vec();
        let mut fut = match state {
            State::Idle(mut inner) => {

                Box::pin(async move {
                    let res = MutexFuture::new(inner.flush()).await;
                    (inner,  res.map_err(|e|e.into()).map(|a|0))
                })
            }
            State::Pending(fut) => {
                // tracing::debug!("polling existing future...");
                fut
            }
            State::Transitional => unreachable!(),
        };

        match fut.as_mut().poll(cx) {
            Poll::Ready((inner, result)) => {
                // tracing::debug!("future was ready!");
                // if let Ok(n) = &result {
                //     let n = *n;
                //     // unsafe { internal_buf.set_len(n) }

                //     // let dst = &mut buf[..n];
                //     // let src = &internal_buf[..];
                //     // dst.copy_from_slice(src);
                // } else {
                //     // unsafe { internal_buf.set_len(0) }
                // }
                *proj.state = State::Idle(inner);
                Poll::Ready(result.map(|_|()))
            }
            Poll::Pending => {
                // tracing::debug!("future was pending!");
                *proj.state = State::Pending(fut);
                Poll::Pending
            }
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        return Poll::Ready(Ok(()));
    }

}