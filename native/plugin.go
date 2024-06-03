package main

// #include "common.h"
import "C"
import (
	esbuild "github.com/evanw/esbuild/pkg/api"
	"sync"
	"sync/atomic"
	"unsafe"
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
			C.Zsb_PluginCallbackOnStart_Dispatch(cb, unsafe.Pointer(data))
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
	v, ok := pluginDescriptorHandles.Load(handle)
	if !ok {
		panic("bad plugin descriptor handle")
	}
	builder := v.(PluginDescriptor)
	if !builder.destroyed {
		C.Zsb_PluginDestructor_Dispatch(builder.destructor, unsafe.Pointer(builder.data))
		builder.destroyed = true
	}
	pluginDescriptorHandles.Delete(handle)
}
