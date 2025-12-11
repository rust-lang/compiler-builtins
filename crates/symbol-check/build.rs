//! Compile test sources to object files.

use std::env;

fn main() {
    let compiler = cc::Build::new().get_compiler();

    println!(
        "cargo::rustc-env=OBJ_TARGET={}",
        env::var("TARGET").unwrap()
    );

    let objs = cc::Build::new()
        .file("tests/no_gnu_stack.S")
        .compile_intermediates();
    let [obj] = objs.as_slice() else {
        panic!(">1 output")
    };
    println!("cargo::rustc-env=NO_GNU_STACK_OBJ={}", obj.display());

    if !compiler.is_like_gnu() {
        println!("cargo::warning=Can't run execstack test; non-GNU compiler");
        return;
    }

    let objs = cc::Build::new()
        .file("tests/has_exe_stack.c")
        .compile_intermediates();
    let [obj] = objs.as_slice() else {
        panic!(">1 output")
    };
    println!("cargo::rustc-env=HAS_EXE_STACK_OBJ={}", obj.display());
}
