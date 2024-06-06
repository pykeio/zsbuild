package main

// #include "common.h"
import "C"
import (
	"sync"
	"sync/atomic"
	"unsafe"

	esbuild "github.com/evanw/esbuild/pkg/api"
)

type PluginDescriptor struct {
	callback   C.PluginBuildCallback
	data       *C.void
	destructor C.PluginDestructor
	destroyed  bool
	plugin     esbuild.Plugin
}

type PluginBuilder struct {
	descriptor *PluginDescriptor
	build      esbuild.PluginBuild
}

var (
	// map[uint64]PluginDescriptor
	pluginDescriptorHandles = sync.Map{}
	pluginDescriptorAcc     atomic.Uint64
	// map[uint64]PluginBuilder
	pluginBuildHandles = sync.Map{}
	pluginBuildAcc     atomic.Uint64
)

func (b *PluginDescriptor) Callback(build esbuild.PluginBuild) {
	buildHandle := pluginBuildAcc.Add(1)

	pluginBuildHandles.Store(buildHandle, PluginBuilder{descriptor: b, build: build})

	C.Zsb_PluginBuildCallback_Dispatch(b.callback, C.uint64_t(buildHandle), unsafe.Pointer(b.data))

	build.OnDispose(func() {
		if !b.destroyed {
			C.Zsb_PluginDestructor_Dispatch(b.destructor, unsafe.Pointer(b.data))
			b.destroyed = true
		}
	})

	pluginBuildHandles.Delete(buildHandle)
}

//export Zsb_PluginBuilder_OnStart
func Zsb_PluginBuilder_OnStart(handle uint64, cb C.PluginCallbackOnStart, data *C.void) {
	v, ok := pluginBuildHandles.Load(handle)
	if !ok {
		panic("bad plugin build handle")
	}
	build := v.(PluginBuilder)
	build.build.OnStart(func() (esbuild.OnStartResult, error) {
		if !build.descriptor.destroyed {
			cRes := C.Zsb_PluginCallbackOnStart_Dispatch(cb, unsafe.Pointer(data))
			if cRes == nil {
				return esbuild.OnStartResult{}, nil
			}

			res := esbuild.OnStartResult{}
			if cRes.errors_len > 0 {
				serializedErrors := unsafe.Slice(cRes.errors, cRes.errors_len)
				errorsLen := int(cRes.errors_len)
				errors := make([]esbuild.Message, errorsLen)
				for i := 0; i < errorsLen; i++ {
					errors[i] = deserializeMessage(&serializedErrors[i])
				}
				res.Errors = errors
			}
			if cRes.warnings_len > 0 {
				serializedWarnings := unsafe.Slice(cRes.warnings, cRes.warnings_len)
				warningsLen := int(cRes.warnings_len)
				warnings := make([]esbuild.Message, warningsLen)
				for i := 0; i < warningsLen; i++ {
					warnings[i] = deserializeMessage(&serializedWarnings[i])
				}
				res.Warnings = warnings
			}
			C.Zsb_PluginOnStartResult_Destroy(cRes)
			return res, nil
		}
		return esbuild.OnStartResult{}, nil
	})
}

//export Zsb_Plugin_Create
func Zsb_Plugin_Create(name *C.char, nameLen C.size_t, callback C.PluginBuildCallback, data *C.void, destructor C.PluginDestructor) uint64 {
	outHandle := pluginDescriptorAcc.Add(1)
	builder := PluginDescriptor{callback: callback, data: data, destructor: destructor, destroyed: false}
	builder.plugin = esbuild.Plugin{Name: C.GoStringN(name, C.int(nameLen)), Setup: builder.Callback}
	pluginDescriptorHandles.Store(outHandle, builder)
	return outHandle
}

//export Zsb_Plugin_Destroy
func Zsb_Plugin_Destroy(handle uint64) {
	_, ok := pluginDescriptorHandles.LoadAndDelete(handle)
	if !ok {
		panic("bad plugin descriptor handle")
	}

	// In this function, we will simply remove the descriptor from the global map so it can be GC'd on the Go side once
	// it is disposed. The `OnDispose()` callback we register will handle cleaning up the Rust side when the plugin is
	// truly no longer in use.
}
