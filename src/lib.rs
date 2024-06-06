mod build;
mod context;
mod dispatch;
mod error;
mod plugin;
#[doc(hidden)]
pub mod sys;
mod util;

pub use self::{
	build::{r#async::BuildFuture, build, options::BuildOptions, BuildResult},
	context::{Context, ContextError},
	error::{Location, Message, Note},
	plugin::{IntoPluginDescriptor, Plugin, PluginBuilder, PluginDescriptor}
};

#[cfg(all(target_family = "windows", target_arch = "x86_64", target_env = "msvc"))]
#[link_section = ".CRT$XCU"]
pub static _ZSB_GORUNTIME_INIT: unsafe extern "C" fn() = {
	// Manually call Go runtime initialization.
	// Go is supposed to place this in the `.ctors` section of the archive, which then invokes it whenever a new thread is
	// constructed. However, somewhere along the way, MSVC/LLVM ignores the constructor, requiring us to manually initialize
	// the runtime using MSVC's CRT equivalent of `.ctors`.
	// ref: https://github.com/golang/go/issues/42190
	extern "C" {
		fn _rt0_amd64_windows_lib();
	}
	_rt0_amd64_windows_lib
};

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use super::{
		error::{MessageBuilder, NoteBuilder},
		plugin::OnStartResult,
		*
	};

	pub struct TestPlugin {
		pub the_number: Arc<usize>
	}

	impl TestPlugin {
		pub fn new(the_number: usize) -> Self {
			Self { the_number: Arc::new(the_number) }
		}
	}

	impl Plugin for TestPlugin {
		fn name(&self) -> &str {
			"test"
		}

		fn build(&self, builder: &mut PluginBuilder) {
			let num = Arc::clone(&self.the_number);
			builder.on_start(move || {
				if *num != 42 {
					OnStartResult::error(MessageBuilder::new("test error").with_note(NoteBuilder::new("test note")))
				} else {
					OnStartResult::ok()
				}
			})
		}
	}

	#[test]
	fn test() {
		let Ok(ctx) = Context::new(
			&BuildOptions::new()
				.entry_point("test/main.js", "out.js")
				.plugin(TestPlugin::new(42))
				.bundle(true)
		) else {
			panic!("error creating context");
		};
		let res = ctx.build();
		if res.is_error() {
			panic!("{}", &res.errors()[0]);
		}
	}

	#[test]
	fn test_plugin_on_start_error() {
		let Ok(ctx) = Context::new(
			&BuildOptions::new()
				.entry_point("test/main.js", "out.js")
				.plugin(TestPlugin::new(7216))
				.bundle(true)
		) else {
			panic!("error creating context");
		};
		let res = ctx.build();
		assert!(res.is_error());
		assert_eq!(res.errors()[0].text(), "test error");
	}

	#[test]
	fn test_bad() {
		let context = Context::new(&BuildOptions::new().entry_point("test/not_exist.js", "out.js").bundle(true)).unwrap();

		let res = context.build();
		assert!(res.is_error());
		let errors = res.errors();
		assert_eq!(errors.len(), 1);
		assert_eq!(errors[0].text(), "Could not resolve \"test/not_exist.js\"");
	}

	#[tokio::test]
	async fn test_async() {
		let Ok(ctx) = Context::new(
			&BuildOptions::new()
				.entry_point("test/main.js", "out.js")
				.plugin(TestPlugin::new(42))
				.bundle(true)
		) else {
			panic!("error creating context");
		};
		let res = ctx.build_async().await;
		if res.is_error() {
			panic!("{}", &res.errors()[0]);
		}
	}
}
