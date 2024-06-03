use std::{
	fmt::{Display, Formatter},
	marker::PhantomData
};

use crate::{sys, util};

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
		util::as_str_opt(self.inner().file, self.inner().file_len)
	}

	#[inline]
	#[must_use]
	pub fn namespace(&self) -> Option<&str> {
		util::as_str_opt(self.inner().namespace, self.inner().namespace_len)
	}

	#[inline]
	#[must_use]
	pub fn line_text(&self) -> &str {
		util::as_str_or_empty(self.inner().line_text, self.inner().line_text_len)
	}

	#[inline]
	#[must_use]
	pub fn suggestion(&self) -> Option<&str> {
		util::as_str_opt(self.inner().suggestion, self.inner().suggestion_len)
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
		util::as_str_or_empty(self.0.text, self.0.text_len)
	}

	pub fn location(&self) -> Location {
		Location(self.0.location, PhantomData)
	}
}

#[repr(transparent)]
pub struct Message<'s>(sys::Message, PhantomData<&'s ()>);

impl<'s> Message<'s> {
	pub fn id(&self) -> Option<&str> {
		util::as_str_opt(self.0.id, self.0.id_len)
	}

	pub fn plugin_name(&self) -> Option<&str> {
		util::as_str_opt(self.0.plugin_name, self.0.plugin_name_len)
	}

	pub fn text(&self) -> &str {
		util::as_str_or_empty(self.0.text, self.0.text_len)
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
			f.write_str(self.text())?;
		} else {
			f.write_str(self.text())?;
		}
		Ok(())
	}
}
