package main

// #include "common.h"
import "C"
import (
	"runtime"
	"sync"
	"sync/atomic"
	"unsafe"

	esbuild "github.com/evanw/esbuild/pkg/api"
)

var (
	// map[uint64]esbuild.BuildContext
	contextHandles       = sync.Map{}
	contextHandleAcc     atomic.Uint64
	contextResultPinners = sync.Map{} // map[*C.struct_ContextResult]*runtime.Pinner
	buildResultPinners   = sync.Map{} // map[*C.struct_BuildResult]*runtime.Pinner
)

//export Zsb_Context_Create
func Zsb_Context_Create(optionsHandle uint64, outHandle *uint64) *C.struct_ContextResult {
	v, ok := buildOptions.Load(optionsHandle)
	if !ok {
		panic("bad build options handle")
	}
	options := v.(*esbuild.BuildOptions)

	*outHandle = contextHandleAcc.Add(1)
	context, err := esbuild.Context(*options)
	if err != nil {
		numMessages := len(err.Errors)
		result := alloc(C.struct_ContextResult{})
		result.messages_len = C.size_t(numMessages)
		pinner := new(runtime.Pinner)
		buildResultPinners.Store(result, pinner)
		if result.messages_len > 0 {
			messagesSlice, messages := allocSlice(result.messages_len, C.struct_Message{})
			for i, msg := range err.Errors {
				messagesSlice[i] = serializeMessage(&msg, pinner)
			}
			result.messages = messages
		}
		return result
	}

	contextHandles.Store(*outHandle, context)
	return nil
}

//export Zsb_ContextResult_Destroy
func Zsb_ContextResult_Destroy(res *C.struct_ContextResult) {
	v, ok := contextResultPinners.LoadAndDelete(res)
	if !ok {
		panic("bad context result addr")
	}
	v.(*runtime.Pinner).Unpin()

	numMessages := int(res.messages_len)
	if numMessages > 0 {
		messagesArr := unsafe.Slice(res.messages, numMessages)
		for i := 0; i < numMessages; i++ {
			destroyMessage(&messagesArr[i])
		}
		free(res.messages)
	}
}

//export Zsb_Context_Build
func Zsb_Context_Build(handle uint64) *C.struct_BuildResult {
	v, ok := contextHandles.Load(handle)
	if !ok {
		panic("bad context handle")
	}
	goRes := v.(esbuild.BuildContext).Rebuild()
	pinner := new(runtime.Pinner)
	cRes := serializeBuildResult(&goRes, pinner)
	buildResultPinners.Store(cRes, pinner)
	return cRes
}

//export Zsb_Build
func Zsb_Build(optionsHandle uint64) *C.struct_BuildResult {
	v, ok := buildOptions.Load(optionsHandle)
	if !ok {
		panic("bad build options handle")
	}
	goRes := esbuild.Build(*v.(*esbuild.BuildOptions))
	pinner := new(runtime.Pinner)
	cRes := serializeBuildResult(&goRes, pinner)
	buildResultPinners.Store(cRes, pinner)
	return cRes
}

func buildContextAsyncInner(ctx esbuild.BuildContext, callback C.BuildAsyncCallback, data unsafe.Pointer) {
	res := ctx.Rebuild()
	pinner := new(runtime.Pinner)
	cRes := serializeBuildResult(&res, pinner)
	buildResultPinners.Store(cRes, pinner)
	C.Zsb_BuildAsyncCallback_Dispatch(callback, cRes, data)
}

func buildAsyncInner(options *esbuild.BuildOptions, callback C.BuildAsyncCallback, data unsafe.Pointer) {
	res := esbuild.Build(*options)
	pinner := new(runtime.Pinner)
	cRes := serializeBuildResult(&res, pinner)
	buildResultPinners.Store(cRes, pinner)
	C.Zsb_BuildAsyncCallback_Dispatch(callback, cRes, data)
}

//export Zsb_Context_BuildAsync
func Zsb_Context_BuildAsync(handle uint64, callback C.BuildAsyncCallback, data *C.void) {
	v, ok := contextHandles.Load(handle)
	if !ok {
		panic("bad context handle")
	}
	ctx := v.(esbuild.BuildContext)
	go buildContextAsyncInner(ctx, callback, unsafe.Pointer(data))
}

//export Zsb_BuildAsync
func Zsb_BuildAsync(optionsHandle uint64, callback C.BuildAsyncCallback, data *C.void) {
	v, ok := buildOptions.Load(optionsHandle)
	if !ok {
		panic("bad build options handle")
	}
	go buildAsyncInner(v.(*esbuild.BuildOptions), callback, unsafe.Pointer(data))
}

func serializeOutputFile(file *esbuild.OutputFile, pinner *runtime.Pinner) C.struct_OutputFile {
	out := C.struct_OutputFile{}
	out.path_len = C.size_t(len(file.Path))
	out.path = pinnedString(file.Path, pinner)
	out.hash_len = C.size_t(len(file.Hash))
	out.hash = pinnedString(file.Hash, pinner)
	out.contents_len = C.size_t(len(file.Contents))
	out.contents = (*C.char)(pinnedSlice(file.Contents, pinner))
	return out
}

func serializeBuildResult(goRes *esbuild.BuildResult, pinner *runtime.Pinner) *C.struct_BuildResult {
	cRes := alloc(C.struct_BuildResult{})

	numOutputFiles := len(goRes.OutputFiles)
	cRes.output_files_len = C.size_t(numOutputFiles)
	if cRes.output_files_len > 0 {
		outputFilesSlice, outputFiles := allocSlice(cRes.output_files_len, C.struct_OutputFile{})
		for i, file := range goRes.OutputFiles {
			outputFilesSlice[i] = serializeOutputFile(&file, pinner)
		}
		cRes.output_files = outputFiles
	}

	numErrors := len(goRes.Errors)
	cRes.errors_len = C.size_t(numErrors)
	if cRes.errors_len > 0 {
		errorsSlice, errors := allocSlice(cRes.errors_len, C.struct_Message{})
		for i, msg := range goRes.Errors {
			errorsSlice[i] = serializeMessage(&msg, pinner)
		}
		cRes.errors = errors
	}

	numWarnings := len(goRes.Warnings)
	cRes.warnings_len = C.size_t(numWarnings)
	if cRes.warnings_len > 0 {
		warningsSlice, warnings := allocSlice(cRes.warnings_len, C.struct_Message{})
		for i, msg := range goRes.Warnings {
			warningsSlice[i] = serializeMessage(&msg, pinner)
		}
		cRes.warnings = warnings
	}

	return cRes
}

//export Zsb_BuildResult_Destroy
func Zsb_BuildResult_Destroy(c *C.struct_BuildResult) {
	numErrors := int(c.errors_len)
	if numErrors > 0 {
		errors := unsafe.Slice(c.errors, numErrors)
		for _, err := range errors {
			destroyMessage(&err)
		}
		free(c.errors)
	}
	numWarnings := int(c.warnings_len)
	if numWarnings > 0 {
		warnings := unsafe.Slice(c.warnings, numWarnings)
		for _, warning := range warnings {
			destroyMessage(&warning)
		}
		free(c.warnings)
	}
	numOutputFiles := int(c.output_files_len)
	if numOutputFiles > 0 {
		outputFiles := unsafe.Slice(c.output_files, numOutputFiles)
		for _, outputFile := range outputFiles {
			destroyOutputFile(&outputFile)
		}
		free(c.output_files)
	}
	free(c)

	pinner, ok := buildResultPinners.LoadAndDelete(c)
	if !ok {
		panic("bad output file pinner addr")
	}
	pinner.(*runtime.Pinner).Unpin()
}

func destroyOutputFile(file *C.struct_OutputFile) {
	// C.free(unsafe.Pointer(file.path))
	// C.free(unsafe.Pointer(file.hash))
	// C.free(unsafe.Pointer(file.contents))
}

//export Zsb_Context_Cancel
func Zsb_Context_Cancel(handle uint64) {
	v, ok := contextHandles.Load(handle)
	if !ok {
		panic("bad context handle")
	}
	v.(esbuild.BuildContext).Cancel()
}

//export Zsb_Context_Destroy
func Zsb_Context_Destroy(handle uint64) {
	v, ok := contextHandles.LoadAndDelete(handle)
	if !ok {
		return
	}
	v.(esbuild.BuildContext).Dispose()
}

func main() {}
