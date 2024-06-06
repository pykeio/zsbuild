use std::{ffi::c_void, ptr, slice};

use tracing::Level;

use crate::{error::MessageBuilder, sys};

pub trait OnStartCallback: FnMut() -> OnStartResult + Send + Sync + 'static {}
impl<F: FnMut() -> OnStartResult + Send + Sync + 'static> OnStartCallback for F {}

type PluginDescriptorBuildCallback = dyn FnMut(&mut PluginBuilder) + Send;
struct PluginCallbacks {
	on_start: Vec<*mut Box<dyn OnStartCallback>>
}

impl Drop for PluginCallbacks {
	fn drop(&mut self) {
		let span = tracing::span!(Level::TRACE, "PluginCallbacks::drop");
		let _enter = span.enter();

		for cb in self.on_start.drain(..) {
			tracing::trace!("Dropping on_start callback @ {:?}", cb);
			drop(unsafe { Box::from_raw(cb) });
		}
	}
}

pub struct PluginDescriptor {
	pub(crate) handle: u64
}

#[derive(Default, Debug, Clone)]
pub struct OnStartResult {
	errors: Vec<MessageBuilder>,
	warnings: Vec<MessageBuilder>
}

impl OnStartResult {
	pub fn ok() -> Self {
		OnStartResult::default()
	}

	pub fn error(message: MessageBuilder) -> Self {
		OnStartResult {
			errors: vec![message],
			..Default::default()
		}
	}

	pub fn with_error(mut self, message: MessageBuilder) -> Self {
		self.errors.push(message);
		self
	}

	pub fn with_warning(mut self, message: MessageBuilder) -> Self {
		self.warnings.push(message);
		self
	}

	pub(crate) fn serialize(self) -> *mut sys::PluginOnStartResult {
		let mut res = sys::PluginOnStartResult {
			errors: ptr::null_mut(),
			errors_len: 0,
			warnings: ptr::null_mut(),
			warnings_len: 0
		};
		if !self.errors.is_empty() {
			let errors = self.errors.into_iter().map(|c| c.serialize()).collect::<Vec<_>>().into_boxed_slice();
			(res.errors_len, res.errors) = (errors.len(), Box::into_raw(errors).cast::<sys::Message>());
		}
		if !self.warnings.is_empty() {
			let warnings = self.warnings.into_iter().map(|c| c.serialize()).collect::<Vec<_>>().into_boxed_slice();
			(res.warnings_len, res.warnings) = (warnings.len(), Box::into_raw(warnings).cast::<sys::Message>());
		}
		Box::into_raw(Box::new(res))
	}

	pub(crate) unsafe fn consume(res: *mut sys::PluginOnStartResult) {
		let res = &mut *res;
		if res.errors_len > 0 {
			let errors = Box::from_raw(slice::from_raw_parts_mut(res.errors, res.errors_len));
			for message in errors.into_vec() {
				MessageBuilder::consume(message);
			}
		}
		if res.warnings_len > 0 {
			let warnings = Box::from_raw(slice::from_raw_parts_mut(res.warnings, res.warnings_len));
			for message in warnings.into_vec() {
				MessageBuilder::consume(message);
			}
		}
		drop(Box::from_raw(res))
	}
}

pub struct PluginBuilder<'d> {
	handle: u64,
	callbacks: &'d mut PluginCallbacks
}

impl<'d> PluginBuilder<'d> {
	pub fn on_start<F: OnStartCallback>(&mut self, callback: F) {
		let callback = Box::into_raw(Box::new(Box::new(callback) as Box<dyn OnStartCallback>));
		self.callbacks.on_start.push(callback);
		unsafe { sys::Zsb_PluginBuilder_OnStart(self.handle, Some(Self::on_start_cb), callback as *mut _) }
		tracing::trace!("Registered on_start callback @ {:?}", callback);
	}

	extern "C" fn on_start_cb(callback: *mut c_void) -> *mut sys::PluginOnStartResult {
		let result = unsafe { (*callback.cast::<Box<dyn OnStartCallback>>())() };
		result.serialize()
	}
}

impl PluginDescriptor {
	pub fn new(name: &str, builder: Box<PluginDescriptorBuildCallback>) -> PluginDescriptor {
		let callbacks = PluginCallbacks { on_start: Vec::new() };
		let data = Box::into_raw(Box::new((builder, callbacks)));
		let handle = unsafe {
			sys::Zsb_Plugin_Create(
				name.as_ptr() as *mut _,
				name.len(),
				Some(Self::plugin_builder_callback),
				data as *mut _,
				Some(Self::plugin_builder_destructor)
			)
		};
		PluginDescriptor { handle }
	}

	extern "C" fn plugin_builder_callback(handle: u64, data: *mut c_void) {
		let data = unsafe { &mut *data.cast::<(Box<PluginDescriptorBuildCallback>, PluginCallbacks)>() };
		let mut builder = PluginBuilder { handle, callbacks: &mut data.1 };
		(data.0)(&mut builder)
	}

	extern "C" fn plugin_builder_destructor(data: *mut c_void) {
		tracing::trace!("Plugin descriptor destructor called; dropping callbacks");
		let data = unsafe { Box::from_raw(data.cast::<(Box<PluginDescriptorBuildCallback>, PluginCallbacks)>()) };
		drop(data);
	}
}

impl Drop for PluginDescriptor {
	fn drop(&mut self) {
		// NOTE: this is only marking the plugin descriptor for Go GC. this does not destroy callbacks & associated data;
		// that happens when the plugin is disposed by esbuild itself.
		unsafe { sys::Zsb_Plugin_Destroy(self.handle) };
	}
}

pub trait Plugin {
	fn name(&self) -> &str;
	fn build(&self, builder: &mut PluginBuilder);
}

pub trait IntoPluginDescriptor {
	fn into_descriptor(self) -> PluginDescriptor;
}

impl<P: Plugin + Send + Sync + 'static> IntoPluginDescriptor for P {
	fn into_descriptor(self) -> PluginDescriptor {
		let name = self.name().to_string();
		PluginDescriptor::new(&name, Box::new(move |builder| self.build(builder)))
	}
}
