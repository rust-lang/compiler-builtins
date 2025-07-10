fn main() {
    let intermediates = cc::Build::new()
        .file("has_wx.c")
        .try_compile_intermediates();
    if let Ok(list) = intermediates {
        let [obj] = list.as_slice() else { panic!() };
        println!("cargo::rustc-env=HAS_WX_OBJ={}", obj.display());
    }
}
