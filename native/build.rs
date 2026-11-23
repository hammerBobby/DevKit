fn main() {
    #[cfg(windows)]
    let _ = embed_resource::compile("assets/devkit.rc", embed_resource::NONE);
}
