use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to tell rustc to link the system nvpair of zfs
    // shared library.
    println!("cargo:rustc-link-lib=uutil");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        .allowlist_var(r#"(\w*uu\w*)"#)
        .allowlist_type(r#"(\w*uu\w*)"#)
        .allowlist_function(r#"(\w*uu\w*)"#)
        .clang_args(vec!["-I/usr/include/libzfs", "-I/usr/include/libspl"])
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: (true),
        })
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("uutil.rs"))
        .expect("Couldn't write bindings!");
}