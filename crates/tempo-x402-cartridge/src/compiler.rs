//! Cartridge compiler — wraps `cargo build --target wasm32-wasip1`.

use std::path::{Path, PathBuf};

use crate::error::CartridgeError;

/// Maximum compilation time.
const COMPILE_TIMEOUT_SECS: u64 = 120;

/// Compile a cartridge from its source directory.
///
/// Source must have a valid Cargo.toml at `source_dir/Cargo.toml`.
/// Output `.wasm` binary goes to `output_dir/{name}.wasm`.
///
/// Returns the path to the compiled WASM binary.
pub async fn compile_cartridge(
    source_dir: &Path,
    output_dir: &Path,
) -> Result<PathBuf, CartridgeError> {
    let cargo_toml = source_dir.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Err(CartridgeError::CompilationFailed(format!(
            "no Cargo.toml at {}",
            source_dir.display()
        )));
    }

    // Ensure output directory exists
    tokio::fs::create_dir_all(output_dir).await?;

    // Ensure wasm32-wasip1 target is installed (might not be at runtime)
    let target_check = tokio::process::Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .await;
    let has_wasip1 = target_check
        .as_ref()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("wasm32-wasip1"))
        .unwrap_or(false);
    if !has_wasip1 {
        tracing::info!("Installing wasm32-wasip1 target for cartridge compilation");
        let _ = tokio::process::Command::new("rustup")
            .args(["target", "add", "wasm32-wasip1"])
            .output()
            .await;
    }

    // Use /tmp for build target to avoid bloating persistent volume
    let target_dir = format!(
        "/tmp/cartridge-build-{}",
        source_dir.file_name().unwrap_or_default().to_string_lossy()
    );

    // Build with wasm32-wasip1 target
    let output = tokio::time::timeout(
        std::time::Duration::from_secs(COMPILE_TIMEOUT_SECS),
        tokio::process::Command::new("cargo")
            .args([
                "build",
                "--target",
                "wasm32-wasip1",
                "--release",
                "--manifest-path",
            ])
            .arg(cargo_toml.to_string_lossy().as_ref())
            .env("CARGO_TARGET_DIR", &target_dir)
            .env("RUSTUP_HOME", "/usr/local/rustup")
            .env("CARGO_HOME", "/usr/local/cargo")
            .output(),
    )
    .await
    .map_err(|_| {
        CartridgeError::CompilationFailed(format!(
            "compilation timed out after {COMPILE_TIMEOUT_SECS}s"
        ))
    })?
    .map_err(|e| CartridgeError::CompilationFailed(format!("cargo failed to start: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Truncate long error output
        let truncated = if stderr.len() > 4096 {
            format!("{}...(truncated)", &stderr[..4096])
        } else {
            stderr.to_string()
        };
        return Err(CartridgeError::CompilationFailed(truncated));
    }

    // Find the compiled .wasm binary in the target directory
    let release_dir_path = format!("{}/wasm32-wasip1/release", target_dir);
    let pattern = format!("{}/*.wasm", release_dir_path);

    let release_dir = std::path::PathBuf::from(&release_dir_path);
    let mut wasm_path = None;
    if let Ok(mut entries) = tokio::fs::read_dir(&release_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().map(|e| e == "wasm").unwrap_or(false)
                && !path.to_string_lossy().contains(".d")
            {
                // Copy to output_dir/{name}.wasm
                let name = path.file_name().unwrap();
                let dest = output_dir.join(name);
                tokio::fs::copy(&path, &dest).await?;
                wasm_path = Some(dest);
                break;
            }
        }
    }

    // Clean up build directory to save disk space
    let _ = tokio::fs::remove_dir_all(&target_dir).await;

    wasm_path.ok_or_else(|| {
        CartridgeError::CompilationFailed(format!("no .wasm binary found in {}", pattern))
    })
}

/// Generate the default Cargo.toml for a new cartridge.
/// NOTE: No dependencies needed — the host ABI uses raw extern "C" FFI.
pub fn default_cargo_toml(slug: &str) -> String {
    format!(
        r#"[package]
name = "{slug}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

# NO DEPENDENCIES NEEDED — the x402 host ABI uses extern "C" functions.
# Do NOT add x402_sdk or any external crate — cartridges are self-contained.
[dependencies]

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
"#
    )
}

/// Generate the default lib.rs template for a new cartridge.
///
/// This is the simplest possible cartridge that compiles and returns a response.
/// The agent fills in the actual logic.
pub fn default_lib_rs(slug: &str) -> String {
    let template = r#"//! __SLUG__ — x402 WASM cartridge
//!
//! This cartridge handles HTTP requests via the x402 host ABI.
//! The host calls `x402_handle` with a JSON request.
//! Call `x402_response` to set the HTTP response.

// Import host functions from the x402 namespace.
// The #[link] attribute ensures WASM imports come from "x402" module, not "env".
#[link(wasm_import_module = "x402")]
extern "C" {
    fn response(status: i32, body_ptr: *const u8, body_len: i32, ct_ptr: *const u8, ct_len: i32);
    fn log(level: i32, msg_ptr: *const u8, msg_len: i32);
    fn kv_get(key_ptr: *const u8, key_len: i32) -> i64;
    fn kv_set(key_ptr: *const u8, key_len: i32, val_ptr: *const u8, val_len: i32) -> i32;
    fn payment_info() -> i64;
}

/// Helper: send a response back to the host.
fn respond(status: i32, body: &str, content_type: &str) {
    unsafe {
        response(
            status,
            body.as_ptr(),
            body.len() as i32,
            content_type.as_ptr(),
            content_type.len() as i32,
        );
    }
}

/// Helper: log a message to the host.
fn host_log(level: i32, msg: &str) {
    unsafe { log(level, msg.as_ptr(), msg.len() as i32); }
}

/// Entry point: handle an HTTP request.
///
/// `request_ptr` points to a JSON string in memory:
/// {"method": "GET", "path": "/", "body": "", "headers": {}}
#[no_mangle]
pub extern "C" fn x402_handle(request_ptr: *const u8, request_len: i32) {
    host_log(1, "__SLUG__ cartridge invoked");

    // Read the request JSON from memory
    let request_bytes = unsafe {
        core::slice::from_raw_parts(request_ptr, request_len as usize)
    };
    let _request = core::str::from_utf8(request_bytes).unwrap_or("{}");

    // TODO: implement your cartridge logic here

    let body = "{\"message\": \"Hello from __SLUG__!\", \"status\": \"ok\"}";
    respond(200, body, "application/json");
}

/// Optional: allocator for host-to-guest memory transfers.
#[no_mangle]
pub extern "C" fn x402_alloc(size: i32) -> *mut u8 {
    let layout = core::alloc::Layout::from_size_align(size as usize, 1).unwrap();
    unsafe { std::alloc::alloc(layout) }
}
"#;
    template.replace("__SLUG__", slug)
}
