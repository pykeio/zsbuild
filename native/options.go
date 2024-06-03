package main

// #include "common.h"
import "C"
import (
	esbuild "github.com/evanw/esbuild/pkg/api"
	"sync"
	"sync/atomic"
)

var (
	// map[uint64]*esbuild.BuildOptions
	buildOptions    = sync.Map{}
	buildOptionsAcc atomic.Uint64
)

//export Zsb_BuildOptions_Create
func Zsb_BuildOptions_Create() (handle uint64) {
	handle = buildOptionsAcc.Add(1)
	buildOptions.Store(handle, &esbuild.BuildOptions{LogLevel: esbuild.LogLevelSilent})
	return handle
}

//export Zsb_BuildOptions_Bundle
func Zsb_BuildOptions_Bundle(handle uint64, enable bool) uint16 {
	v, ok := buildOptions.Load(handle)
	if !ok {
		panic("bad build options handle")
	}
	v.(*esbuild.BuildOptions).Bundle = enable
	return 0
}

//export Zsb_BuildOptions_AppendEntryPoint
func Zsb_BuildOptions_AppendEntryPoint(handle uint64, input *C.char, inputLen C.size_t, output *C.char, outputLen C.size_t) uint16 {
	v, ok := buildOptions.Load(handle)
	if !ok {
		panic("bad build options handle")
	}
	options := v.(*esbuild.BuildOptions)
	options.EntryPointsAdvanced = append(options.EntryPointsAdvanced, esbuild.EntryPoint{
		InputPath:  C.GoStringN(input, C.int(inputLen)),
		OutputPath: C.GoStringN(output, C.int(outputLen)),
	})
	return 0
}

//export Zsb_BuildOptions_AddPlugin
func Zsb_BuildOptions_AddPlugin(handle uint64, pluginHandle uint64) uint64 {
	v, ok := buildOptions.Load(handle)
	if !ok {
		panic("bad build options handle")
	}
	options := v.(*esbuild.BuildOptions)
	plugin, ok := pluginDescriptorHandles.Load(pluginHandle)
	if !ok {
		panic("bad plugin handle")
	}
	options.Plugins = append(options.Plugins, plugin.(PluginDescriptor).plugin)
	return 0
}

//export Zsb_BuildOptions_Destroy
func Zsb_BuildOptions_Destroy(handle uint64) {
	buildOptions.Delete(handle)
}
