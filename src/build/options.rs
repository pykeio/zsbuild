use crate::{plugin::IntoPluginDescriptor, sys};

pub struct BuildOptions(u64);

impl BuildOptions {
	#[inline]
	#[must_use]
	pub fn new() -> BuildOptions {
		BuildOptions(unsafe { sys::Zsb_BuildOptions_Create() })
	}

	pub fn entry_point(self, in_path: impl AsRef<str>, out_path: impl AsRef<str>) -> Self {
		let in_path = in_path.as_ref();
		let out_path = out_path.as_ref();
		if unsafe { sys::Zsb_BuildOptions_AppendEntryPoint(self.0, in_path.as_ptr() as *mut _, in_path.len(), out_path.as_ptr() as *mut _, out_path.len()) }
			!= 0
		{
			panic!("");
		}
		self
	}

	pub fn plugin<P: IntoPluginDescriptor>(self, plugin: P) -> Self {
		let descriptor = plugin.into_descriptor();
		if unsafe { sys::Zsb_BuildOptions_AddPlugin(self.0, descriptor.handle) } != 0 {
			panic!("");
		}
		self
	}

	pub fn bundle(self, enable: bool) -> Self {
		unsafe { sys::Zsb_BuildOptions_Bundle(self.0, enable.into()) };
		self
	}

	pub(crate) fn handle(&self) -> u64 {
		self.0
	}
}

impl Default for BuildOptions {
	fn default() -> Self {
		BuildOptions(unsafe { sys::Zsb_BuildOptions_Create() })
	}
}

impl Drop for BuildOptions {
	fn drop(&mut self) {
		tracing::trace!("Dropping BuildOptions");
		unsafe { sys::Zsb_BuildOptions_Destroy(self.0) };
	}
}
