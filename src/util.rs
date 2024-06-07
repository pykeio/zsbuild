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

pub fn decompose_string(s: String) -> (usize, *const i8) {
	let s = s.into_boxed_str();
	// SAFETY: casting a `*mut str` to `*mut i8` rightfully seems psychopathic, but remember that Box<[T]> (which
	// Box<str> basically is) is actually a "fat" pointer. Only the data of the slice (in this case the string data)
	// is being pointed to; the length is actually stored *alongside* the pointer (`size_of::<Box<[T]>>() == 16`)
	// instead of being directed to by it.  Thus, it's okay to cast the raw pointer to the slice's element type.
	(s.len(), Box::into_raw(s).cast::<i8>().cast_const())
}

pub fn decompose_vec<T>(s: Vec<T>) -> (usize, *mut T) {
	let s = s.into_boxed_slice();
	(s.len(), Box::into_raw(s).cast::<T>())
}

pub unsafe fn drop_decomposed_string(ptr: *const i8, len: usize) {
	drop(Box::from_raw(slice::from_raw_parts_mut(ptr.cast_mut(), len)))
}

pub unsafe fn recompose_vec<T>(ptr: *const T, len: usize) -> Vec<T> {
	Box::from_raw(slice::from_raw_parts_mut(ptr.cast_mut(), len)).into_vec()
}

pub trait IntoFFI {
	type FFIType;

	fn into_ffi(self) -> Self::FFIType;
	unsafe fn drop_ffi(ty: Self::FFIType);
}
