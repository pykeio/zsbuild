package main

// #include "common.h"
import "C"
import (
	"runtime"
	"unsafe"
)

func alloc[T any](empty T) (ptr *T) {
	ptr = (*T)(C.malloc(C.size_t(unsafe.Sizeof(empty))))
	return ptr
}

func allocSlice[T any](numel C.size_t, empty T) (slice []T, ptr *T) {
	ptr = (*T)(C.malloc(numel * C.size_t(unsafe.Sizeof(empty))))
	return unsafe.Slice(ptr, int(numel)), ptr
}

func free[T any](ptr *T) {
	C.free(unsafe.Pointer(ptr))
}

func pinnedString(str string, pinner *runtime.Pinner) *C.char {
	contents_ptr := unsafe.Pointer(unsafe.StringData(str))
	pinner.Pin(&contents_ptr)
	return (*C.char)(contents_ptr)
}

func pinnedSlice[T any](slice []T, pinner *runtime.Pinner) unsafe.Pointer {
	contents_ptr := unsafe.Pointer(unsafe.SliceData(slice))
	pinner.Pin(&contents_ptr)
	return contents_ptr
}
