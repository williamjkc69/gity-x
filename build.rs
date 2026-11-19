fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "gity x");
        res.set("FileDescription", "gity x — Fast Git GUI in Rust");
        let _ = res.compile();
    }
}
