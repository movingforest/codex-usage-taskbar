use std::env;
use std::io;
use std::path::PathBuf;
use std::process::Command;

use winres::{VersionInfo, WindowsResource};

fn main() {
    let version = env!("CARGO_PKG_VERSION");

    // Embed the icon and richer PE version metadata into the executable.
    let mut res = WindowsResource::new();
    let numeric_version = pack_version(version);

    res.set_icon("src/icons/icon.ico")
        .set("FileVersion", version)
        .set("ProductVersion", version)
        .set_version_info(VersionInfo::FILEVERSION, numeric_version)
        .set_version_info(VersionInfo::PRODUCTVERSION, numeric_version);

    let result = if env::var_os("CODEX_WINRES_POWERSHELL").is_some() {
        compile_resources_without_gcc(&res)
    } else {
        res.compile()
    };
    result.expect("Failed to compile Windows resources");
}

/// Compile the generated RC file when MinGW's resource compiler is available
/// but a full GCC preprocessor is not. The generated file contains no C
/// includes or macros, so PowerShell can safely pass it through unchanged.
fn compile_resources_without_gcc(res: &WindowsResource) -> io::Result<()> {
    let output_dir = PathBuf::from(env::var("OUT_DIR").map_err(io::Error::other)?);
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").map_err(io::Error::other)?;
    let rc = output_dir.join("resource.rc");
    let object = output_dir.join("resource.o");
    let library = output_dir.join("libresource.a");
    let windres = env::var("WINDRES").unwrap_or_else(|_| "windres.exe".to_string());
    let ar = env::var("AR").unwrap_or_else(|_| "ar.exe".to_string());

    res.write_resource_file(&rc)?;
    let status = Command::new(windres)
        .current_dir(&manifest_dir)
        .arg("--preprocessor=powershell.exe")
        .arg("--preprocessor-arg=-NoProfile")
        .arg("--preprocessor-arg=-Command")
        .arg("--preprocessor-arg=& { Get-Content -Raw -LiteralPath $args[-1] }")
        .arg("--use-temp-file")
        .arg(format!("-I{manifest_dir}"))
        .arg(&rc)
        .arg(&object)
        .status()?;
    if !status.success() {
        return Err(io::Error::other("windres failed"));
    }

    let status = Command::new(ar)
        .arg("rsc")
        .arg(&library)
        .arg(&object)
        .status()?;
    if !status.success() {
        return Err(io::Error::other("ar failed"));
    }

    println!("cargo:rustc-link-search=native={}", output_dir.display());
    println!("cargo:rustc-link-lib=static=resource");
    Ok(())
}

fn pack_version(version: &str) -> u64 {
    let core = version.split('-').next().unwrap_or(version);
    let mut parts = core.split('.').map(|part| part.parse::<u64>().unwrap_or(0));

    let major = parts.next().unwrap_or(0).min(u16::MAX as u64);
    let minor = parts.next().unwrap_or(0).min(u16::MAX as u64);
    let patch = parts.next().unwrap_or(0).min(u16::MAX as u64);

    (major << 48) | (minor << 32) | (patch << 16)
}
