use std::io;

fn main() -> io::Result<()> {
    if cfg!(target_os = "windows") {
        let res = winres::WindowsResource::new();
        res.compile()?;
    }
    Ok(())
}