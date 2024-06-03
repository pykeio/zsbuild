use std::{ffi::c_char, slice, str};

pub unsafe fn as_str<'s>(ptr: *const c_char, len: usize) -> &'s str {
	unsafe { str::from_utf8_unchecked(slice::from_raw_parts(ptr.cast(), len)) }
}

pub unsafe fn as_str_opt<'s>(ptr: *const c_char, len: usize) -> Option<&'s str> {
	(!ptr.is_null()).then(|| as_str(ptr, len))
}

pub unsafe fn as_str_or_empty<'s>(ptr: *const c_char, len: usize) -> &'s str {
	if !ptr.is_null() { as_str(ptr, len) } else { "" }
}

pub unsafe fn slice_from_raw_parts_or_empty<'a, T>(data: *const T, len: usize) -> &'a [T] {
	if data.is_null() && len == 0 { &[] } else { std::slice::from_raw_parts(data, len) }
}
