use std::{marker::PhantomData, slice, str::Utf8Error};

use crate::{error::Message, sys, util};

pub mod r#async;
pub mod options;

#[repr(transparent)]
pub struct OutputFile<'s>(sys::OutputFile, PhantomData<&'s ()>);

impl<'s> OutputFile<'s> {
	pub fn contents(&self) -> &[u8] {
		unsafe { slice::from_raw_parts(self.0.contents.cast(), self.0.contents_len) }
	}

	pub fn contents_str(&self) -> Result<&str, Utf8Error> {
		std::str::from_utf8(
			(!self.0.contents.is_null())
				.then(|| unsafe { slice::from_raw_parts(self.0.contents.cast::<u8>(), self.0.contents_len) })
				.unwrap_or(&[])
		)
	}
}

pub struct BuildResult(*mut sys::BuildResult);

unsafe impl Send for BuildResult {}

impl BuildResult {
	pub(crate) fn new(res: *mut sys::BuildResult) -> Self {
		Self(res)
	}

	#[inline(always)]
	fn inner(&self) -> &sys::BuildResult {
		unsafe { &*self.0 }
	}

	pub fn is_error(&self) -> bool {
		self.inner().errors_len > 0
	}

	pub fn errors(&self) -> &[Message<'_>] {
		unsafe { util::slice_from_raw_parts_or_empty(self.inner().errors.cast_const().cast::<Message>(), self.inner().errors_len) }
	}

	pub fn warnings(&self) -> &[Message<'_>] {
		unsafe { util::slice_from_raw_parts_or_empty(self.inner().warnings.cast_const().cast::<Message>(), self.inner().warnings_len) }
	}

	pub fn outputs(&self) -> &[OutputFile<'_>] {
		unsafe { util::slice_from_raw_parts_or_empty(self.inner().output_files.cast_const().cast::<OutputFile>(), self.inner().output_files_len) }
	}
}

pub fn build(options: &self::options::BuildOptions) -> BuildResult {
	BuildResult::new(unsafe { sys::Zsb_Build(options.handle()) })
}

impl Drop for BuildResult {
	fn drop(&mut self) {
		tracing::trace!("Dropping BuildResult");
		unsafe { sys::Zsb_BuildResult_Destroy(self.0) };
	}
}
