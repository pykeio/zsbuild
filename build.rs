use std::{env, path::PathBuf};

fn main() {
	let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
	let native_out_path = manifest.join("native").join("out");
	println!("cargo:rustc-link-search={}", native_out_path.to_string_lossy());
	println!("cargo:rustc-link-lib=static=zsb");

	let target = env::var("CARGO_CFG_TARGET_FAMILY").unwrap();
	if target == "windows" {
		// println!("cargo:rustc-link-lib=static=win32rt_patch");
	}

	#[cfg(feature = "bindgen")]
	{
		use bindgen::builder;

		let bindings = builder()
			.allowlist_type("Zsb_.+")
			.allowlist_function("Zsb_.+")
			.allowlist_recursively(true)
			.layout_tests(false)
			.clang_arg(format!("-I{}", manifest.join("native").to_string_lossy()))
			.header(native_out_path.join("zsb.h").to_string_lossy())
			.generate()
			.unwrap();

		bindings.write_to_file(manifest.join("src").join("sys.rs")).unwrap();
	}
}
