use build_target;
use build_target::{Arch, Os};
use std::env;
use std::path::Path;

fn android_host_dir() -> &'static str {
    let host = env::var("HOST").unwrap();
    if host.contains("aarch64") || host.contains("arm") || host.contains("thumb") {
        panic!("AArch64/ARM/Thumb hosts are unimplemented.");
    }
    return match host.as_str() {
        "x86_64-pc-windows-gnu" | "x86_64-pc-windows-msvc" => "windows-x86_64",
        "x86_64-apple-darwin" => "darwin-x86_64",
        _ => "linux-x86_64"
    };
}

fn main() {
    match build_target::target_os().unwrap() {
        Os::Android => {
            let archstr = match build_target::target_arch().unwrap() {
                Arch::AARCH64 => "aarch64",
                Arch::ARM => "arm",
                Arch::X86 => "i686",
                Arch::X86_64 => "x86_64",
                _ => panic!("Unknown arch!")
            };
            let clang_archstr = match archstr {
                "i686" => "i386",
                _ => archstr
            };
            let abi_suffix = match archstr {
                "arm" => "eabi",
                _ => ""
            };
            let ndk = env::var("ANDROID_NDK_HOME").unwrap();
            let clang_ver = "14.0.6";
            let ndk_api_ver = "33";
            let toolchain_dir = Path::new(&ndk).join("toolchains").join("llvm").join("prebuilt").join(android_host_dir());
            let libgcc_path = Path::new(&toolchain_dir).join("lib64").join("clang").join(clang_ver).join("lib").join("linux").join(clang_archstr);
            let libandroid_path = Path::new(&toolchain_dir).join("sysroot").join("usr").join("lib").join(format!("{}-linux-android{}", archstr, abi_suffix)).join(ndk_api_ver);
            println!("cargo:rustc-flags=-L{} -L{}", libgcc_path.as_os_str().to_str().unwrap(), libandroid_path.as_os_str().to_str().unwrap());
        },
        Os::iOs => {
            panic!("iOS build unsupported...");
        },
        _ => {
            panic!("Unknown OS!");
        }
    }
}
