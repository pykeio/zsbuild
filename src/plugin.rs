use std::ffi::c_void;

use tracing::Level;

use crate::sys;

type PluginDescriptorBuildCallback = dyn FnMut(&mut PluginBuilder) + Send;
struct PluginCallbacks {
	on_start: Vec<*mut Box<dyn FnMut() + Send + Sync>>
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

pub struct PluginBuilder<'d> {
	handle: u64,
	callbacks: &'d mut PluginCallbacks
}

impl<'d> PluginBuilder<'d> {
	pub fn on_start<F: FnMut() + Send + Sync + 'static>(&mut self, callback: F) {
		let callback = Box::into_raw(Box::new(Box::new(callback) as Box<dyn FnMut() + Send + Sync>));
		self.callbacks.on_start.push(callback);
		unsafe { sys::Zsb_PluginBuilder_OnStart(self.handle, Some(Self::on_start_cb), callback as *mut _) }
		tracing::trace!("Registered on_start callback @ {:?}", callback);
	}

	extern "C" fn on_start_cb(callback: *mut c_void) {
		unsafe { (*callback.cast::<Box<dyn FnMut() + Send + Sync>>())() }
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
