use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-search=/usr/lib/x86_64-linux-gnu/");
    println!("cargo:rustc-link-lib=marisa");

    let bindings = bindgen::Builder::default()
        .impl_debug(true)
        .size_t_is_usize(true)
        .generate_cstr(true)
        .header("marisa_wrapper.hpp")
        .clang_arg("-std=c++17")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate  bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("marisa_bindings.rs"))
        .expect("Couldn't write bindings!");
}
