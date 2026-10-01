fn main() {
    // 让产物可以被其他程序按 rpath / soname 加载，而不是写死编译机上的绝对路径。
    if std::env::var("CARGO_CFG_TARGET_OS").ok().as_deref() == Some("macos") {
        println!("cargo::rustc-link-arg-cdylib=-Wl,-install_name,@rpath/libhwinfo.dylib");
    }
    if std::env::var("CARGO_CFG_TARGET_OS").ok().as_deref() == Some("linux") {
        println!("cargo::rustc-link-arg-cdylib=-Wl,-soname,libhwinfo.so");
    }
}
