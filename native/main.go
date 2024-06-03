package main

// #include "common.h"
import "C"
import (
	esbuild "github.com/evanw/esbuild/pkg/api"
	"sync"
	"sync/atomic"
	"unsafe"
)

var (
	// map[uint64]esbuild.BuildContext
	contextHandles   = sync.Map{}
	contextHandleAcc atomic.Uint64
)

//export Zsb_Context_Create
func Zsb_Context_Create(optionsHandle uint64, outHandle *uint64) C.struct_ContextResult {
	v, ok := buildOptions.Load(optionsHandle)
	if !ok {
		panic("bad build options handle")
	}
	options := v.(*esbuild.BuildOptions)

	*outHandle = contextHandleAcc.Add(1)
	context, err := esbuild.Context(*options)
	if err != nil {
		numMessages := len(err.Errors)
		result := C.struct_ContextResult{is_err: true}
		result.messages_len = C.size_t(numMessages)
		if result.messages_len > 0 {
			messages := C.malloc(result.messages_len * C.size_t(unsafe.Sizeof(C.struct_Message{})))
			messagesArr := (*[1 << 28]C.struct_Message)(messages)[:numMessages:numMessages]
			for i, msg := range err.Errors {
				messagesArr[i] = serializeMessage(&msg)
			}
			result.messages = (*C.struct_Message)(messages)
		}
		return result
	}

	contextHandles.Store(*outHandle, context)
	return C.struct_ContextResult{is_err: false}
}

//export Zsb_Context_Build
func Zsb_Context_Build(handle uint64) *C.struct_BuildResult {
	v, ok := contextHandles.Load(handle)
	if !ok {
		panic("bad context handle")
	}
	goRes := v.(esbuild.BuildContext).Rebuild()
	return serializeBuildResult(&goRes)
}

//export Zsb_Build
func Zsb_Build(optionsHandle uint64) *C.struct_BuildResult {
	v, ok := buildOptions.Load(optionsHandle)
	if !ok {
		panic("bad build options handle")
	}
	goRes := esbuild.Build(*v.(*esbuild.BuildOptions))
	return serializeBuildResult(&goRes)
}

func buildContextAsyncInner(ctx esbuild.BuildContext, callback C.BuildAsyncCallback, data unsafe.Pointer) {
	res := ctx.Rebuild()
	C.Zsb_BuildAsyncCallback_Dispatch(callback, serializeBuildResult(&res), data)
}

func buildAsyncInner(options *esbuild.BuildOptions, callback C.BuildAsyncCallback, data unsafe.Pointer) {
	res := esbuild.Build(*options)
	C.Zsb_BuildAsyncCallback_Dispatch(callback, serializeBuildResult(&res), data)
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

func serializeOutputFile(file *esbuild.OutputFile) C.struct_OutputFile {
	out := C.struct_OutputFile{}
	out.path_len = C.size_t(len(file.Path))
	out.path = C.CString(file.Path)
	out.hash_len = C.size_t(len(file.Hash))
	out.hash = C.CString(file.Hash)
	out.contents_len = C.size_t(len(file.Contents))
	out.contents = (*C.char)(C.CBytes(file.Contents))
	return out
}

func serializeBuildResult(goRes *esbuild.BuildResult) *C.struct_BuildResult {
	cResPtr := C.malloc(C.size_t(unsafe.Sizeof(C.struct_BuildResult{})))
	cRes := (*C.struct_BuildResult)(cResPtr)

	numOutputFiles := len(goRes.OutputFiles)
	cRes.output_files_len = C.size_t(numOutputFiles)
	if cRes.output_files_len > 0 {
		outputFiles := C.malloc(cRes.output_files_len * C.size_t(unsafe.Sizeof(C.struct_OutputFile{})))
		outputFilesArr := (*[1 << 28]C.struct_OutputFile)(outputFiles)[:numOutputFiles:numOutputFiles]
		for i, file := range goRes.OutputFiles {
			outputFilesArr[i] = serializeOutputFile(&file)
		}
		cRes.output_files = (*C.struct_OutputFile)(outputFiles)
	}

	numErrors := len(goRes.Errors)
	cRes.errors_len = C.size_t(numErrors)
	if cRes.errors_len > 0 {
		errors := C.malloc(cRes.errors_len * C.size_t(unsafe.Sizeof(C.struct_Message{})))
		errorsArr := (*[1 << 28]C.struct_Message)(errors)[:numErrors:numErrors]
		for i, msg := range goRes.Errors {
			errorsArr[i] = serializeMessage(&msg)
		}
		cRes.errors = (*C.struct_Message)(errors)
	}

	numWarnings := len(goRes.Warnings)
	cRes.warnings_len = C.size_t(numWarnings)
	if cRes.warnings_len > 0 {
		warnings := C.malloc(cRes.warnings_len * C.size_t(unsafe.Sizeof(C.struct_Message{})))
		warningsArr := (*[1 << 28]C.struct_Message)(warnings)[:numWarnings:numWarnings]
		for i, msg := range goRes.Warnings {
			warningsArr[i] = serializeMessage(&msg)
		}
		cRes.warnings = (*C.struct_Message)(warnings)
	}

	return cRes
}

//export Zsb_BuildResult_Destroy
func Zsb_BuildResult_Destroy(c *C.struct_BuildResult) {
	numErrors := int(c.errors_len)
	if numErrors > 0 {
		errorsArr := (*[1 << 28]C.struct_Message)(unsafe.Pointer(c.errors))[:numErrors:numErrors]
		for i := 0; i < numErrors; i++ {
			destroyMessage(&errorsArr[i])
		}
		C.free(unsafe.Pointer(c.errors))
	}
	numWarnings := int(c.warnings_len)
	if numWarnings > 0 {
		warningsArr := (*[1 << 28]C.struct_Message)(unsafe.Pointer(c.warnings))[:numWarnings:numWarnings]
		for i := 0; i < numWarnings; i++ {
			destroyMessage(&warningsArr[i])
		}
		C.free(unsafe.Pointer(c.warnings))
	}
	numOutputFiles := int(c.output_files_len)
	if numOutputFiles > 0 {
		outputFilesArr := (*[1 << 28]C.struct_OutputFile)(unsafe.Pointer(c.output_files))[:numOutputFiles:numOutputFiles]
		for i := 0; i < numOutputFiles; i++ {
			destroyOutputFile(&outputFilesArr[i])
		}
		C.free(unsafe.Pointer(c.output_files))
	}
	C.free(unsafe.Pointer(c))
}

func destroyOutputFile(file *C.struct_OutputFile) {
	C.free(unsafe.Pointer(file.path))
	C.free(unsafe.Pointer(file.hash))
	C.free(unsafe.Pointer(file.contents))
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
	v, ok := contextHandles.Load(handle)
	if !ok {
		return
	}
	v.(esbuild.BuildContext).Dispose()
	contextHandles.Delete(handle)
}

func main() {}
