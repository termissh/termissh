use std::error::Error;
use std::fs::File;
use std::path::Path;

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR is missing");
    let icon_path = Path::new(&out_dir).join("termissh.ico");

    generate_ico("src/icons/mini-icon.png", &icon_path)
        .expect("failed to generate icon from src/icons/mini-icon.png");

    let mut res = winres::WindowsResource::new();
    res.set_icon(icon_path.to_str().expect("icon path is not valid UTF-8"));
    res.compile().expect("failed to embed Windows icon resource");
}

fn generate_ico(png_path: &str, ico_path: &Path) -> Result<(), Box<dyn Error>> {
    let image = image::open(png_path)?.into_rgba8();
    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

    for &size in &[16u32, 32, 48, 256] {
        let resized = image::imageops::resize(
            &image,
            size,
            size,
            image::imageops::FilterType::Lanczos3,
        );
        let icon_image = ico::IconImage::from_rgba_data(size, size, resized.into_raw());
        let icon_entry = ico::IconDirEntry::encode(&icon_image)?;
        icon_dir.add_entry(icon_entry);
    }

    let mut file = File::create(ico_path)?;
    icon_dir.write(&mut file)?;
    Ok(())
}
