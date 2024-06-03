use std::{env, path::PathBuf};

fn main() {
	let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
	let native_out_path = manifest.join("native").join("out");
	println!("cargo:rustc-link-search={}", native_out_path.to_string_lossy());
	println!("cargo:rustc-link-lib=static=zsb");

	let target_family = env::var("CARGO_CFG_TARGET_FAMILY").unwrap();
	let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
	let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
	if target_family == "windows" {
		// println!("cargo:rustc-link-lib=static=win32rt_patch");
		if target_env == "msvc" {
			println!("cargo:rustc-link-lib=legacy_stdio_definitions");
		}
	} else {
		match &*target_os {
			"macos" | "tvos" | "ios" => {
				println!("cargo:rustc-link-lib=framework=CoreFoundation");
			}
			_ => {}
		}
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
