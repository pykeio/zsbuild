use std::{
	fmt::{self, Debug, Write},
	sync::{Arc, Mutex}
};

use crate::{
	build::{
		r#async::{BuildFuture, BuildFutureState},
		options::BuildOptions,
		BuildResult
	},
	error::Message,
	sys, util
};

pub struct ContextError(*mut sys::ContextResult);

impl ContextError {
	#[inline(always)]
	fn inner(&self) -> &sys::ContextResult {
		unsafe { &*self.0 }
	}

	pub fn messages(&self) -> &[Message<'_>] {
		unsafe { util::slice_from_raw_parts_or_empty(self.inner().messages.cast_const().cast::<Message>(), self.inner().messages_len) }
	}
}

impl Debug for ContextError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let messages = self.messages();
		if messages.len() == 1 {
			write!(f, "{}", messages[0])?;
		} else {
			for i in 0..messages.len() {
				write!(f, "{}. {}", i + 1, messages[i])?;
				if i != messages.len() - 1 {
					f.write_char('\n')?;
				}
			}
		}
		Ok(())
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

impl Debug for SharedContextHandle {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "<cgo:{}>", self.0)
	}
}

impl Drop for SharedContextHandle {
	fn drop(&mut self) {
		unsafe { sys::Zsb_Context_Destroy(self.0) };
	}
}

#[derive(Clone, Debug)]
pub struct Context {
	handle: Arc<SharedContextHandle>
}

impl Context {
	#[inline]
	pub fn new(options: &BuildOptions) -> Result<Self, ContextError> {
		let mut handle = 0;
		let res = unsafe { sys::Zsb_Context_Create(options.handle(), &mut handle) };
		if !res.is_null() {
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
