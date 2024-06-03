use std::{
	ffi::c_void,
	future::Future,
	pin::Pin,
	sync::{Arc, Mutex},
	task::{Context, Poll, Waker}
};

use crate::{context::SharedContextHandle, sys, BuildResult};

struct ContextBuildAsyncCallbackData {
	cb: *mut Box<dyn FnOnce(BuildResult) + Send>,
	_context_handle: Arc<SharedContextHandle>
}

pub(crate) fn context_build_async_inner<F: FnOnce(BuildResult) + Send + 'static>(context: Arc<SharedContextHandle>, cb: F) {
	let handle = context.handle();
	let data = Box::into_raw(Box::new(ContextBuildAsyncCallbackData {
		_context_handle: context,
		cb: Box::into_raw(Box::new(Box::new(cb) as Box<dyn FnOnce(BuildResult) + Send>))
	}));

	unsafe { sys::Zsb_Context_BuildAsync(handle, Some(context_build_async_callback), data as *mut _) };
}

extern "C" fn context_build_async_callback(build_result: *mut sys::BuildResult, data: *mut c_void) {
	let data = unsafe { Box::from_raw(data as *mut ContextBuildAsyncCallbackData) };
	let cb = unsafe { Box::from_raw(data.cb) };
	(*cb)(BuildResult::new(build_result))
}

#[derive(Default)]
pub struct BuildFutureState {
	pub(crate) result: Option<BuildResult>,
	pub(crate) waker: Option<Waker>
}

impl BuildFutureState {
	pub(crate) fn set_and_wake(&mut self, res: BuildResult) {
		self.result = Some(res);
		if let Some(waker) = self.waker.take() {
			waker.wake();
		}
	}
}

pub struct BuildFuture {
	context: Option<Arc<SharedContextHandle>>,
	cancellable: bool,
	state: Arc<Mutex<BuildFutureState>>
}

impl BuildFuture {
	pub(crate) fn new(state: Arc<Mutex<BuildFutureState>>, context: Option<&Arc<SharedContextHandle>>) -> Self {
		BuildFuture {
			context: context.cloned(),
			cancellable: false,
			state
		}
	}

	pub fn cancellable(mut self) -> Self {
		self.cancellable = true;
		self
	}
}

impl Future for BuildFuture {
	type Output = BuildResult;

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let mut state = self.state.lock().unwrap();
		match state.result.take() {
			Some(result) => Poll::Ready(result),
			None => {
				state.waker = Some(cx.waker().clone());
				Poll::Pending
			}
		}
	}
}

impl Drop for BuildFuture {
	fn drop(&mut self) {
		if self.cancellable {
			if let Some(context) = &self.context {
				unsafe { sys::Zsb_Context_Cancel(context.handle()) };
			}
		}
	}
}
