use std::sync::{Arc, Mutex};

use crate::{
	build::{
		r#async::{BuildFuture, BuildFutureState},
		options::BuildOptions,
		BuildResult
	},
	error::Message,
	sys, util
};

pub struct ContextError(sys::ContextResult);

impl ContextError {
	pub fn messages(&self) -> &[Message<'_>] {
		unsafe { util::slice_from_raw_parts_or_empty(self.0.messages.cast_const().cast::<Message>(), self.0.messages_len) }
	}
}

impl Drop for ContextError {
	fn drop(&mut self) {
		unsafe { sys::Zsb_ContextResult_Destroy(self.0) };
	}
}

pub(crate) struct SharedContextHandle(pub(crate) u64);

impl SharedContextHandle {
	pub(crate) fn handle(&self) -> u64 {
		self.0
	}
}

impl Drop for SharedContextHandle {
	fn drop(&mut self) {
		unsafe { sys::Zsb_Context_Destroy(self.0) };
	}
}

pub struct Context {
	handle: Arc<SharedContextHandle>
}

impl Context {
	#[inline]
	pub fn new(options: &BuildOptions) -> Result<Self, ContextError> {
		let mut handle = 0;
		let res = unsafe { sys::Zsb_Context_Create(options.handle(), &mut handle) };
		if res.is_err {
			return Err(ContextError(res));
		}
		Ok(Context {
			handle: Arc::new(SharedContextHandle(handle))
		})
	}

	pub(crate) fn handle(&self) -> u64 {
		self.handle.handle()
	}

	pub fn cancel_all(&self) {
		unsafe { sys::Zsb_Context_Cancel(self.handle()) };
	}

	pub fn build(&self) -> BuildResult {
		BuildResult::new(unsafe { sys::Zsb_Context_Build(self.handle()) })
	}

	pub fn build_async(&self) -> BuildFuture {
		let state = Arc::new(Mutex::new(BuildFutureState::default()));
		let _state = state.clone();
		crate::build::r#async::context_build_async_inner(Arc::clone(&self.handle), move |res| {
			let mut state = _state.lock().unwrap();
			state.set_and_wake(res);
		});
		BuildFuture::new(state, Some(&self.handle))
	}
}
