#[rustversion::nightly]
const NIGHTLY: bool = true;

#[rustversion::not(nightly)]
const NIGHTLY: bool = false;

fn main() {
    println!("cargo:rustc-check-cfg=cfg(nightly)");
    if NIGHTLY {
        println!("cargo:rustc-cfg=nightly");
    }

    // Get target information
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    // Conditionally compile assembly using cc for Windows ARM64 MSVC
    if target_arch == "aarch64" && target_os == "windows" && target_env == "msvc" {
        // Enable the asm-backend feature which pulls in the cc dependency
        println!("cargo:rustc-cfg=feature=\"asm-backend\"");

        // Use cfg! to conditionally execute cc build only if the feature is enabled
        // This requires the user to build with --features asm-backend or have it in default
        // Alternatively, we can directly call cc::Build if we don't need the feature toggle.
        // Let's directly call it for simplicity here, assuming cc is available when needed.
        #[cfg(feature = "asm-backend")] // This check might be redundant if we always compile
        {
            cc::Build::new()
                .file("src/detail/asm/aarch64_windows.asm")
                .compile("asm_helpers"); // Compile into a static library named libasm_helpers.a (or .lib)
        }

        // If not using the feature toggle, the code would be:
        /*
        match cc::Build::new()
            .file("src/detail/asm/aarch64_windows.asm")
            .try_compile("asm_helpers") // Use try_compile for better error handling
        {
            Ok(_) => println!("Successfully compiled aarch64_windows.asm"),
            Err(e) => panic!("Failed to compile aarch64_windows.asm: {}", e),
        }
        */
    }

    // Add logic for other targets if they also need cc build in the future
    // else if target_arch == "..." && target_os == "..." { ... }
}
