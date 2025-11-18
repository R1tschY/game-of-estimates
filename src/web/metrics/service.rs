use http::{Request, Response};
use http_body::Body;
use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{ready, Context, Poll};
use tower::{Layer, Service};

pub trait RequestMetricsEmitterFactory {
    type RequestBody;
    type ResponseBody;
    type Emitter: RequestMetricEmitter<ResponseBody = Self::ResponseBody>;

    fn new_emitter(&self, request: &Request<Self::RequestBody>) -> Self::Emitter;
}

pub trait RequestMetricEmitter {
    type ResponseBody;

    fn handle_response(&mut self, response: &Response<Self::ResponseBody>);
    fn emit(self);
}

#[derive(Clone)]
pub struct RequestMetricsLayer<M> {
    collector: M,
}

impl<M> RequestMetricsLayer<M> {
    pub fn new(collector: M) -> Self {
        Self { collector }
    }
}

impl<S, M: RequestMetricsEmitterFactory + Clone + Send + Sync> Layer<S> for RequestMetricsLayer<M> {
    type Service = RequestMetrics<S, M>;

    fn layer(&self, inner: S) -> Self::Service {
        RequestMetrics {
            inner,
            collector: self.collector.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RequestMetrics<S, M> {
    inner: S,
    collector: M,
}

impl<S, R, ResBody, M> Service<Request<R>> for RequestMetrics<S, M>
where
    S: Service<Request<R>, Response = Response<ResBody>>,
    M: RequestMetricsEmitterFactory<RequestBody = R, ResponseBody = ResBody>,
{
    type Response = Response<ResponseBody<ResBody, M::Emitter>>;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future, M::Emitter>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<R>) -> Self::Future {
        let emitter = self.collector.new_emitter(&request);
        let future = self.inner.call(request);
        ResponseFuture {
            inner: future,
            emitter: Some(GuardedEmitter(Some(emitter))),
        }
    }
}

struct GuardedEmitter<M: RequestMetricEmitter>(Option<M>);

impl<M: RequestMetricEmitter> Drop for GuardedEmitter<M> {
    fn drop(&mut self) {
        if let Some(inner) = self.0.take() {
            inner.emit();
        }
    }
}

pin_project! {
    /// Response future for [`RequestMetrics`].
    pub struct ResponseFuture<F, M: RequestMetricEmitter> {
        #[pin]
        inner: F,
        emitter: Option<GuardedEmitter<M>>
    }
}

impl<F, B, E, M> Future for ResponseFuture<F, M>
where
    F: Future<Output = Result<Response<B>, E>>,
    M: RequestMetricEmitter<ResponseBody = B>,
{
    type Output = Result<Response<ResponseBody<B, M>>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let response = ready!(this.inner.poll(cx))?;
        let mut emitter = this.emitter.take().unwrap();
        if let Some(inner) = emitter.0.as_mut() {
            inner.handle_response(&response);
        }
        let response = response.map(move |body| ResponseBody {
            inner: body,
            emitter,
        });

        Poll::Ready(Ok(response))
    }
}

pin_project! {
    /// Response body for [`RequestMetrics`].
    pub struct ResponseBody<B, M: RequestMetricEmitter> {
        #[pin]
        inner: B,
        emitter: GuardedEmitter<M>,
    }
}

impl<B, M> Body for ResponseBody<B, M>
where
    B: Body,
    M: RequestMetricEmitter,
{
    type Data = B::Data;
    type Error = B::Error;

    #[inline]
    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        self.project().inner.poll_frame(cx)
    }

    #[inline]
    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    #[inline]
    fn size_hint(&self) -> http_body::SizeHint {
        self.inner.size_hint()
    }
}
