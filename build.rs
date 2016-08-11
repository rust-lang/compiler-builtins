use std::path::Path;
use std::env;

fn main() {
    let target_dir = Path::new(env!("OUT_DIR")).parent().unwrap().parent().unwrap().parent().unwrap().parent().unwrap();
    let mut deps_dir = target_dir.to_owned();
    let host = env::var_os("HOST").unwrap();
    let target = env::var_os("TARGET").unwrap();

    if target != host {
        deps_dir.push(target);
    }
    deps_dir.push(env::var_os("PROFILE").unwrap());
    deps_dir.push("deps");

    println!("cargo:rustc-link-search=native={}", deps_dir.display());
    println!("cargo:rustc-link-lib=static=rlibc");
}
