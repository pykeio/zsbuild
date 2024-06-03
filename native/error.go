package main

// #include "common.h"
import "C"
import (
	esbuild "github.com/evanw/esbuild/pkg/api"
	"unsafe"
)

func serializeLocation(location *esbuild.Location) *C.struct_Location {
	ptr := C.malloc(C.size_t(unsafe.Sizeof(C.struct_Location{})))
	serialized := (*C.struct_Location)(ptr)
	if len(location.File) != 0 {
		serialized.file = C.CString(location.File)
		serialized.file_len = C.size_t(len(location.File))
	}
	if len(location.Namespace) != 0 {
		serialized.namespace = C.CString(location.Namespace)
		serialized.namespace_len = C.size_t(len(location.Namespace))
	}
	serialized.line = C.int64_t(location.Line)
	serialized.column = C.int64_t(location.Column)
	serialized.length = C.int64_t(location.Length)
	if len(location.LineText) != 0 {
		serialized.line_text = C.CString(location.LineText)
		serialized.line_text_len = C.size_t(len(location.LineText))
	}
	if len(location.Suggestion) != 0 {
		serialized.suggestion = C.CString(location.Suggestion)
		serialized.suggestion_len = C.size_t(len(location.Suggestion))
	}
	return serialized
}

func deserializeLocation(location *C.struct_Location) esbuild.Location {
	deserialized := esbuild.Location{}
	if location.file_len != 0 && location.file != nil {
		deserialized.File = C.GoStringN(location.file, C.int(location.file_len))
	}
	deserialized.Line = int(location.line)
	deserialized.Column = int(location.column)
	deserialized.Length = int(location.length)
	if location.line_text_len != 0 && location.line_text != nil {
		deserialized.LineText = C.GoStringN(location.line_text, C.int(location.line_text_len))
	}
	if location.suggestion_len != 0 && location.suggestion != nil {
		deserialized.Suggestion = C.GoStringN(location.suggestion, C.int(location.suggestion_len))
	}
	return deserialized
}

func destroyLocation(location *C.struct_Location) {
	if location == nil {
		return
	}
	C.free(unsafe.Pointer(location.file))
	C.free(unsafe.Pointer(location.namespace))
	C.free(unsafe.Pointer(location.line_text))
	C.free(unsafe.Pointer(location.suggestion))
	C.free(unsafe.Pointer(location))
}

func serializeNote(note *esbuild.Note) C.struct_Note {
	serialized := C.struct_Note{}
	if len(note.Text) > 0 {
		serialized.text = C.CString(note.Text)
		serialized.text_len = C.size_t(len(note.Text))
	}
	if note.Location != nil {
		serialized.location = serializeLocation(note.Location)
	}
	return serialized
}

func deserializeNote(note *C.struct_Note) esbuild.Note {
	deserialized := esbuild.Note{}
	if note.text_len > 0 && note.text != nil {
		deserialized.Text = C.GoStringN(note.text, C.int(note.text_len))
	}
	if note.location != nil {
		location := deserializeLocation(note.location)
		deserialized.Location = &location
	}
	return deserialized
}

func destroyNote(note *C.struct_Note) {
	C.free(unsafe.Pointer(note.text))
	destroyLocation(note.location)
}

func serializeMessage(message *esbuild.Message) C.struct_Message {
	serialized := C.struct_Message{}
	if len(message.ID) != 0 {
		serialized.id = C.CString(message.ID)
		serialized.id_len = C.size_t(len(message.ID))
	}
	if len(message.PluginName) != 0 {
		serialized.plugin_name = C.CString(message.PluginName)
		serialized.plugin_name_len = C.size_t(len(message.PluginName))
	}
	if len(message.Text) != 0 {
		serialized.text = C.CString(message.Text)
		serialized.text_len = C.size_t(len(message.Text))
	}
	if message.Location != nil {
		serialized.location = serializeLocation(message.Location)
	}
	numNotes := len(message.Notes)
	serialized.notes_len = C.size_t(numNotes)
	if serialized.notes_len > 0 {
		notes := C.malloc(serialized.notes_len * C.size_t(unsafe.Sizeof(C.struct_Note{})))
		notesArr := (*[1 << 28]C.struct_Note)(notes)[:numNotes:numNotes]
		for i, note := range message.Notes {
			notesArr[i] = serializeNote(&note)
		}
		serialized.notes = (*C.struct_Note)(notes)
	}
	return serialized
}

func deserializeMessage(message *C.struct_Message) esbuild.Message {
	deserialized := esbuild.Message{}
	if message.id_len > 0 && message.id != nil {
		deserialized.ID = C.GoStringN(message.id, C.int(message.id_len))
	}
	if message.plugin_name_len > 0 && message.plugin_name != nil {
		deserialized.PluginName = C.GoStringN(message.plugin_name, C.int(message.plugin_name_len))
	}
	if message.text_len > 0 && message.text != nil {
		deserialized.Text = C.GoStringN(message.text, C.int(message.text_len))
	}
	if message.location != nil {
		location := deserializeLocation(message.location)
		deserialized.Location = &location
	}
	if message.notes_len > 0 {
		serializedNotes := (*[1 << 28]C.struct_Note)(unsafe.Pointer(message.notes))[:message.notes_len:message.notes_len]
		notesLen := int(message.notes_len)
		notes := make([]esbuild.Note, notesLen)
		for i := 0; i < notesLen; i++ {
			notes[i] = deserializeNote(&serializedNotes[i])
		}
	}
	return deserialized
}

func destroyMessage(message *C.struct_Message) {
	C.free(unsafe.Pointer(message.id))
	C.free(unsafe.Pointer(message.plugin_name))
	C.free(unsafe.Pointer(message.text))
	C.free(unsafe.Pointer(message.notes))
	destroyLocation(message.location)
	numNotes := int(message.notes_len)
	if numNotes > 0 {
		notesArr := (*[1 << 28]C.struct_Note)(unsafe.Pointer(message.notes))[:numNotes:numNotes]
		for i := 0; i < numNotes; i++ {
			destroyNote(&notesArr[i])
		}
		C.free(unsafe.Pointer(message.notes))
	}
}

//export Zsb_ContextResult_Destroy
func Zsb_ContextResult_Destroy(res C.struct_ContextResult) {
	numMessages := int(res.messages_len)
	if numMessages > 0 {
		messagesArr := (*[1 << 28]C.struct_Message)(unsafe.Pointer(res.messages))[:numMessages:numMessages]
		for i := 0; i < numMessages; i++ {
			destroyMessage(&messagesArr[i])
		}
		C.free(unsafe.Pointer(res.messages))
	}
}
