fn main() {
    // 确保库和二进制程序的链接正确
    println!("cargo:rerun-if-changed=src/lib.rs");
}
