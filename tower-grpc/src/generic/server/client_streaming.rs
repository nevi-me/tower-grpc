use super::streaming;
use super::unary::Once;
use crate::generic::{Encode, Encoder};
use crate::Response;

use futures::{try_ready, Future, Poll};

#[derive(Debug)]
pub struct ResponseFuture<T, E> {
    inner: streaming::ResponseFuture<Inner<T>, E>,
}

#[derive(Debug)]
struct Inner<T> {
    inner: T,
}

// ===== impl ResponseFuture ======

impl<T, E> ResponseFuture<T, E>
where
    T: Future<Item = Response<E::Item>, Error = crate::Status>,
    E: Encoder,
{
    pub fn new(inner: T, encoder: E) -> Self {
        let inner = Inner { inner };
        let inner = streaming::ResponseFuture::new(inner, encoder);
        ResponseFuture { inner }
    }
}

impl<T, E> Future for ResponseFuture<T, E>
where
    T: Future<Item = Response<E::Item>, Error = crate::Status>,
    E: Encoder,
{
    type Item = http::Response<Encode<E, Once<E::Item>>>;
    type Error = crate::error::Never;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner.poll()
    }
}

// ===== impl Inner ======

impl<T, U> Future for Inner<T>
where
    T: Future<Item = Response<U>, Error = crate::Status>,
{
    type Item = Response<Once<U>>;
    type Error = crate::Status;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let response = try_ready!(self.inner.poll());
        Ok(Once::map(response).into())
    }
}
