use std::{
	fmt::{Display, Formatter},
	marker::PhantomData,
	ptr
};

use crate::{
	sys,
	util::{self, IntoFFI}
};

#[repr(transparent)]
pub struct Location<'s>(*mut sys::Location, PhantomData<&'s ()>);

impl<'s> Location<'s> {
	#[inline(always)]
	fn inner(&self) -> &sys::Location {
		unsafe { &*self.0 }
	}

	#[inline]
	#[must_use]
	pub fn file(&self) -> Option<&str> {
		unsafe { util::as_str_opt(self.inner().file, self.inner().file_len) }
	}

	#[inline]
	#[must_use]
	pub fn namespace(&self) -> Option<&str> {
		unsafe { util::as_str_opt(self.inner().namespace_, self.inner().namespace_len) }
	}

	#[inline]
	#[must_use]
	pub fn line_text(&self) -> &str {
		unsafe { util::as_str_or_empty(self.inner().line_text, self.inner().line_text_len) }
	}

	#[inline]
	#[must_use]
	pub fn suggestion(&self) -> Option<&str> {
		unsafe { util::as_str_opt(self.inner().suggestion, self.inner().suggestion_len) }
	}

	#[inline]
	pub fn line(&self) -> usize {
		self.inner().line as _
	}

	#[inline]
	pub fn column(&self) -> usize {
		self.inner().column as _
	}

	#[inline]
	#[allow(clippy::len_without_is_empty)]
	pub fn len(&self) -> usize {
		self.inner().length as _
	}
}

#[repr(transparent)]
pub struct Note<'s>(sys::Note, PhantomData<&'s ()>);

impl<'s> Note<'s> {
	pub fn text(&self) -> &str {
		unsafe { util::as_str_or_empty(self.0.text, self.0.text_len) }
	}

	pub fn location(&self) -> Location {
		Location(self.0.location, PhantomData)
	}
}

#[repr(transparent)]
pub struct Message<'s>(sys::Message, PhantomData<&'s ()>);

impl<'s> Message<'s> {
	pub fn id(&self) -> Option<&str> {
		unsafe { util::as_str_opt(self.0.id, self.0.id_len) }
	}

	pub fn plugin_name(&self) -> Option<&str> {
		unsafe { util::as_str_opt(self.0.plugin_name, self.0.plugin_name_len) }
	}

	pub fn text(&self) -> &str {
		unsafe { util::as_str_or_empty(self.0.text, self.0.text_len) }
	}

	pub fn location(&self) -> Option<Location> {
		(!self.0.location.is_null()).then_some(Location(self.0.location, PhantomData))
	}

	pub fn notes(&self) -> &[Note<'s>] {
		unsafe { util::slice_from_raw_parts_or_empty(self.0.notes.cast_const().cast::<Note>(), self.0.notes_len) }
	}
}

impl<'s> Display for Message<'s> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		if let Some(location) = self.location() {
			f.write_str(location.file().unwrap())?;
			f.write_str(":")?;
			write!(f, "{}", location.line())?;
			f.write_str(":")?;
			write!(f, "{}", location.column())?;
			f.write_str(": ")?;
		}
		f.write_str(self.text())?;
		let notes = self.notes();
		if !notes.is_empty() {
			for (i, note) in notes.iter().enumerate() {
				f.write_str(" (note")?;
				if notes.len() != 1 {
					write!(f, " {}", i + 1)?;
				}
				f.write_str(": ")?;
				f.write_str(note.text())?;
				f.write_str(")")?;
			}
		}
		Ok(())
	}
}

#[derive(Default, Debug, Clone)]
pub struct LocationBuilder {
	file: String,
	namespace: Option<String>,
	line: i64,
	column: i64,
	len: i64,
	line_text: Option<String>,
	suggestion: Option<String>
}

impl LocationBuilder {
	pub fn new(file: impl ToString, line: i64, column: i64, len: i64) -> Self {
		Self {
			file: file.to_string(),
			line,
			column,
			len,
			..Default::default()
		}
	}

	pub fn with_namespace(mut self, namespace: impl ToString) -> Self {
		self.namespace = Some(namespace.to_string());
		self
	}

	pub fn with_line_text(mut self, line_text: impl ToString) -> Self {
		self.line_text = Some(line_text.to_string());
		self
	}

	pub fn with_suggestion(mut self, suggestion: impl ToString) -> Self {
		self.suggestion = Some(suggestion.to_string());
		self
	}
}

impl IntoFFI for LocationBuilder {
	type FFIType = *mut sys::Location;

	fn into_ffi(self) -> Self::FFIType {
		let mut location = sys::Location {
			line: self.line,
			column: self.column,
			length: self.len,
			file: ptr::null_mut(),
			file_len: 0,
			line_text: ptr::null_mut(),
			line_text_len: 0,
			namespace_: ptr::null_mut(),
			namespace_len: 0,
			suggestion: ptr::null_mut(),
			suggestion_len: 0
		};
		(location.file_len, location.file) = util::decompose_string(self.file);
		if let Some(namespace) = self.namespace {
			(location.namespace_len, location.namespace_) = util::decompose_string(namespace);
		}
		if let Some(line_text) = self.line_text {
			(location.line_text_len, location.line_text) = util::decompose_string(line_text);
		}
		if let Some(suggestion) = self.suggestion {
			(location.suggestion_len, location.suggestion) = util::decompose_string(suggestion);
		}
		Box::into_raw(Box::new(location))
	}

	unsafe fn drop_ffi(location: Self::FFIType) {
		let location = &mut *location;
		util::drop_decomposed_string(location.file, location.file_len);
		if !location.namespace_.is_null() {
			util::drop_decomposed_string(location.namespace_, location.namespace_len);
		}
		if !location.line_text.is_null() {
			util::drop_decomposed_string(location.line_text, location.line_text_len);
		}
		if !location.suggestion.is_null() {
			util::drop_decomposed_string(location.suggestion, location.suggestion_len);
		}
		drop(Box::from_raw(location as *mut sys::Location));
	}
}

#[derive(Default, Debug, Clone)]
pub struct NoteBuilder {
	text: String,
	location: Option<LocationBuilder>
}

impl NoteBuilder {
	pub fn new(text: impl ToString) -> Self {
		Self {
			text: text.to_string(),
			location: None
		}
	}

	pub fn at(mut self, location: LocationBuilder) -> Self {
		self.location = Some(location);
		self
	}
}

impl IntoFFI for NoteBuilder {
	type FFIType = sys::Note;

	fn into_ffi(self) -> Self::FFIType {
		let text = self.text.into_boxed_str();
		sys::Note {
			text_len: text.len(),
			text: Box::into_raw(text).cast::<i8>().cast_const(),
			location: self.location.map(LocationBuilder::into_ffi).unwrap_or_else(ptr::null_mut)
		}
	}

	unsafe fn drop_ffi(note: Self::FFIType) {
		util::drop_decomposed_string(note.text, note.text_len);
		if !note.location.is_null() {
			LocationBuilder::drop_ffi(note.location);
		}
	}
}

#[derive(Default, Debug, Clone)]
pub struct MessageBuilder {
	id: Option<String>,
	plugin_name: Option<String>,
	text: String,
	location: Option<LocationBuilder>,
	notes: Vec<NoteBuilder>
}

impl MessageBuilder {
	pub fn new(text: impl ToString) -> Self {
		Self {
			text: text.to_string(),
			..Default::default()
		}
	}

	pub fn with_note(mut self, note: NoteBuilder) -> Self {
		self.notes.push(note);
		self
	}

	pub fn with_id(mut self, id: impl ToString) -> Self {
		self.id = Some(id.to_string());
		self
	}

	pub fn with_plugin_name(mut self, plugin: impl ToString) -> Self {
		self.plugin_name = Some(plugin.to_string());
		self
	}
}

impl IntoFFI for MessageBuilder {
	type FFIType = sys::Message;

	fn into_ffi(self) -> Self::FFIType {
		let mut message = sys::Message {
			id: ptr::null(),
			id_len: 0,
			location: self.location.map(LocationBuilder::into_ffi).unwrap_or_else(ptr::null_mut),
			notes: ptr::null_mut(),
			notes_len: 0,
			plugin_name: ptr::null(),
			plugin_name_len: 0,
			text: ptr::null(),
			text_len: 0
		};
		(message.text_len, message.text) = util::decompose_string(self.text);
		if !self.notes.is_empty() {
			(message.notes_len, message.notes) = util::decompose_vec(self.notes.into_iter().map(NoteBuilder::into_ffi).collect());
		}
		message
	}

	unsafe fn drop_ffi(message: Self::FFIType) {
		util::drop_decomposed_string(message.text, message.text_len);
		if message.notes_len > 0 {
			for note in util::recompose_vec(message.notes, message.notes_len) {
				NoteBuilder::drop_ffi(note);
			}
		}
	}
}
