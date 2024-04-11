use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();

    // These platforms do not have f128 symbols available in their system libraries, so
    // skip related tests.
    if target.starts_with("arm-")
        || target.contains("apple-darwin")
        || target.contains("windows-msvc")
    {
        println!("cargo:warning=skipping `f128` tests; system does not have relevant symbols");
        println!("cargo:rustc-cfg=feature=\"no-sys-f128\"");
    }
}
