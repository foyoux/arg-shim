use std::io;

fn main() -> io::Result<()> {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        // 显式设置 Windows 资源信息
        res.set("CompanyName", "foyou");
        res.set("FileDescription", "arg-shim - A flexible CLI argument transformer");
        res.set("LegalCopyright", "Copyright (c) 2025 foyou");
        res.set("Comments", "https://github.com/foyoux/arg-shim");
        res.compile()?;
    }
    Ok(())
}
