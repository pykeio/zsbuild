use std::ffi::c_void;

use crate::sys;

#[no_mangle]
pub unsafe extern "C" fn Zsb_BuildAsyncCallback_Dispatch(callback: sys::BuildAsyncCallback, result: *mut sys::BuildResult, data: *mut c_void) {
	callback.unwrap()(result, data)
}

#[no_mangle]
pub unsafe extern "C" fn Zsb_PluginBuildCallback_Dispatch(callback: sys::PluginBuildCallback, handle: u64, data: *mut c_void) {
	callback.unwrap()(handle, data)
}

#[no_mangle]
pub unsafe extern "C" fn Zsb_PluginDestructor_Dispatch(callback: sys::PluginDestructor, data: *mut c_void) {
	callback.unwrap()(data)
}

#[no_mangle]
pub unsafe extern "C" fn Zsb_PluginCallbackOnStart_Dispatch(callback: sys::PluginCallbackOnStart, data: *mut c_void) {
	callback.unwrap()(data)
}
