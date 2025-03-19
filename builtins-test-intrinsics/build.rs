mod builtins_configure {
    include!("../configure.rs");
}

fn main() {
    println!("cargo::rerun-if-changed=../configure.rs");

    let target = builtins_configure::Target::from_env();
    builtins_configure::configure_f16_f128(&target);
    builtins_configure::configure_aliases(&target);

    if target.os == "windows" {
        // Needed for using the `mainCRTStartup` entrypoint
        println!("cargo::rustc-link-arg=/subsystem:console");
    }
}
