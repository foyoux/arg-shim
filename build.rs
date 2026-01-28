use std::io;

fn main() -> io::Result<()> {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        // winres automatically uses package name, version, description from Cargo.toml
        res.compile()?;
    }
    Ok(())
}
