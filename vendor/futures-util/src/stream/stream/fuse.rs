use core::pin::Pin;
use futures_core::stream::{FusedStream, Stream};
use futures_core::task::{Context, Poll};
#[cfg(feature = "sink")]
use futures_sink::Sink;
use pin_project::{pin_project, project};

/// Stream for the [`fuse`](super::StreamExt::fuse) method.
#[pin_project]
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct Fuse<St> {
    #[pin]
    stream: St,
    done: bool,
}

impl<St> Fuse<St> {
    pub(super) fn new(stream: St) -> Fuse<St> {
        Fuse { stream, done: false }
    }

    /// Returns whether the underlying stream has finished or not.
    ///
    /// If this method returns `true`, then all future calls to poll are
    /// guaranteed to return `None`. If this returns `false`, then the
    /// underlying stream is still in use.
    pub fn is_done(&self) -> bool {
        self.done
    }

    delegate_access_inner!(stream, St, ());
}

impl<S: Stream> FusedStream for Fuse<S> {
    fn is_terminated(&self) -> bool {
        self.done
    }
}

impl<S: Stream> Stream for Fuse<S> {
    type Item = S::Item;

    #[project]
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<S::Item>> {
        #[project]
        let Fuse { stream, done } = self.project();

        if *done {
            return Poll::Ready(None);
        }

        let item = ready!(stream.poll_next(cx));
        if item.is_none() {
            *done = true;
        }
        Poll::Ready(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.done {
            (0, Some(0))
        } else {
            self.stream.size_hint()
        }
    }
}

// Forwarding impl of Sink from the underlying stream
#[cfg(feature = "sink")]
impl<S: Stream + Sink<Item>, Item> Sink<Item> for Fuse<S> {
    type Error = S::Error;

    delegate_sink!(stream, Item);
}
