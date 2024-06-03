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

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use super::*;

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
				println!("the number: {num}");
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
