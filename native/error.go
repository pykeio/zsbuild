package main

// #include "common.h"
import "C"
import (
	"runtime"
	"unsafe"

	esbuild "github.com/evanw/esbuild/pkg/api"
)

func serializeLocation(location *esbuild.Location, pinner *runtime.Pinner) *C.struct_Location {
	serialized := alloc(C.struct_Location{})
	if len(location.File) != 0 {
		serialized.file = pinnedString(location.File, pinner)
		serialized.file_len = C.size_t(len(location.File))
	}
	if len(location.Namespace) != 0 {
		serialized.namespace_ = pinnedString(location.Namespace, pinner)
		serialized.namespace_len = C.size_t(len(location.Namespace))
	}
	serialized.line = C.int64_t(location.Line)
	serialized.column = C.int64_t(location.Column)
	serialized.length = C.int64_t(location.Length)
	if len(location.LineText) != 0 {
		serialized.line_text = pinnedString(location.LineText, pinner)
		serialized.line_text_len = C.size_t(len(location.LineText))
	}
	if len(location.Suggestion) != 0 {
		serialized.suggestion = pinnedString(location.Suggestion, pinner)
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
	// free(location.file)
	// free(location.namespace_)
	// free(location.line_text)
	// free(location.suggestion)
	free(location)
}

func serializeNote(note *esbuild.Note, pinner *runtime.Pinner) C.struct_Note {
	serialized := C.struct_Note{}
	if len(note.Text) > 0 {
		serialized.text = pinnedString(note.Text, pinner)
		serialized.text_len = C.size_t(len(note.Text))
	}
	if note.Location != nil {
		serialized.location = serializeLocation(note.Location, pinner)
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
	// free(note.text)
	destroyLocation(note.location)
}

func serializeMessage(message *esbuild.Message, pinner *runtime.Pinner) C.struct_Message {
	serialized := C.struct_Message{}
	if len(message.ID) != 0 {
		serialized.id = pinnedString(message.ID, pinner)
		serialized.id_len = C.size_t(len(message.ID))
	}
	if len(message.PluginName) != 0 {
		serialized.plugin_name = pinnedString(message.PluginName, pinner)
		serialized.plugin_name_len = C.size_t(len(message.PluginName))
	}
	if len(message.Text) != 0 {
		serialized.text = pinnedString(message.Text, pinner)
		serialized.text_len = C.size_t(len(message.Text))
	}
	if message.Location != nil {
		serialized.location = serializeLocation(message.Location, pinner)
	}
	numNotes := len(message.Notes)
	serialized.notes_len = C.size_t(numNotes)
	if serialized.notes_len > 0 {
		notes, notesPtr := allocSlice(serialized.notes_len, C.struct_Note{})
		for i, note := range message.Notes {
			notes[i] = serializeNote(&note, pinner)
		}
		serialized.notes = notesPtr
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
		serializedNotes := unsafe.Slice(message.notes, message.notes_len)
		notesLen := int(message.notes_len)
		notes := make([]esbuild.Note, notesLen)
		for i := 0; i < notesLen; i++ {
			notes[i] = deserializeNote(&serializedNotes[i])
		}
		deserialized.Notes = notes
	}
	return deserialized
}

func destroyMessage(message *C.struct_Message) {
	// free(message.id)
	// free(message.plugin_name)
	// free(message.text)
	destroyLocation(message.location)
	numNotes := int(message.notes_len)
	if numNotes > 0 {
		notesArr := unsafe.Slice(message.notes, numNotes)
		for i := 0; i < numNotes; i++ {
			destroyNote(&notesArr[i])
		}
		free(message.notes)
	}
}
