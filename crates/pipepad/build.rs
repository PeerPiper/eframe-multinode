//! Build script to build the RDX template at compile time.
//! RDX is just Rhai and Html text, so this makes sense to do so.
//! This saves the runtime from size and compute.
//! Also, it enables us to check the Rhai to ensure it compiles,
//! at Rust compile time.
//!
//! This build script uses the html crate to programmatically build
//! the html portion of RDX with type safety. This means no typos!
//!
//! Then this html is injected into the Rhai by using
//!
//! ```ignore
//! let html = "<p>some html text</p>"
//! let rhai = format!(r#" the {html} "#);
//! ````
#![recursion_limit = "512"]

/// Our app code
#[path = "build/app.rs"]
mod app;

use std::env;
use std::fs;
use std::path::PathBuf;

use rhai::Engine;
use rhai::ParseError;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap_or_default();
    let rhai_filename = env::var("CARGO_PKG_NAME").unwrap() + ".rhai";
    let dest_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("src")
        .join(&rhai_filename);

    let rhai_verified = assert_rhai_compiles();

    // also write it to out_dir
    fs::write(
        PathBuf::from(out_dir).join(&rhai_filename),
        rhai_verified.clone(),
    )
    .expect("Failed to write to file");
    // put a copy in the src folder
    fs::write(
        &dest_path,
        format!(
            "/* 
AUTOGNERATED FILE DO NOT EDIT!

Type-safe source generated by build.rs
*/\n\n{}",
            rhai_verified
        ),
    )
    .expect("Failed to write to file");
    println!("cargo:rerun-if-changed=build.rs");
    // rerun if anything in the ./build folder changes
    println!("cargo:rerun-if-changed=build");
}

/// Compile the Rhai in advamce, give a Rustc compile error if it fails.
fn assert_rhai_compiles() -> String {
    let script = app::gen_script();

    let engine = Engine::new();
    match engine.compile(&script) {
        Ok(_) => (),
        Err(e) => {
            let ParseError(err_msg, position) = e;
            let line = position.line().unwrap_or(0);
            let column = position.position().unwrap_or(0);
            let error_msg = format!(
                "Rhai script compilation error at line {}, column {}: {}",
                line, column, err_msg
            );

            // If compilation fails, emit a compile-time error
            println!(
                "cargo:warning=Rhai script compilation error: {:?}",
                error_msg
            );
            panic!("Rhai script compilation failed");
        }
    }

    script
}
