#ifdef _MSC_VER
#ifdef __cplusplus
extern "C" {
#endif

	void _rt0_amd64_windows_lib();

    // Manually call Go runtime initialization.
    // Go is supposed to place this in the `.ctors` section of the archive, which then invokes it whenever a new thread is constructed.
    // However, somewhere along the way, MSVC/LLVM ignores the constructor, requiring us to manually initialize the runtime using MSVC's
    // CRT equivalent of `.ctors`.
    // ref: https://github.com/golang/go/issues/42190
	__pragma(section(".CRT$XCU", read));
	__declspec(allocate(".CRT$XCU")) void (*init1)() = _rt0_amd64_windows_lib;
	__pragma(comment(linker, "/include:init1"));

#ifdef __cplusplus
}
#endif
#endif
