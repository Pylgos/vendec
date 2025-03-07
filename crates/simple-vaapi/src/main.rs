use anyhow::Result;
use vendec_libva as va;

fn draw_color_bar(width: u32, height: u32, frame: u32) -> ::image::RgbaImage {
    let mut image = ::image::RgbaImage::new(width, height);
    let bar_width = width / 7;
    let colors = [
        [192, 192, 192, 255], // White
        [192, 192, 0, 255],   // Yellow
        [0, 192, 192, 255],   // Cyan
        [0, 192, 0, 255],     // Green
        [192, 0, 192, 255],   // Magenta
        [192, 0, 0, 255],     // Red
        [0, 0, 192, 255],     // Blue
    ];

    for (i, color) in colors.iter().enumerate() {
        let start_x = ((i as u32 * bar_width) + frame) % width;
        for x in 0..bar_width {
            for y in 0..height {
                let pixel_x = (start_x + x) % width;
                image.put_pixel(pixel_x, y, ::image::Rgba(*color));
            }
        }
    }
    image
}

fn upload_image(surface: &va::Surface, img: &image::RgbaImage) {
    let image = surface.derive_image().unwrap();
    let mut mapped = image.buffer().map().unwrap();
    let format = image.format();
    if image.format().fourcc == va::Fourcc::try_from("NV12").unwrap() {
        let luma = &mut mapped[image.offsets()[0] as usize..];
    }
}

fn main() -> Result<()> {
    let width = 1920;
    let height = 1080;

    let library = va::Library::load()?;
    let display = va::Display::enumerate(library).next().unwrap();
    let config = display.get_config_attributes(None, va::Entrypoint::VideoProc)?;
    println!("{:#?}", config);
    let config = va::Config::new(
        display.clone(),
        None,
        va::Entrypoint::VideoProc,
        &va::ConfigAttributes::default(),
    )?;
    let surface_attrs = config.query_surface_attributes()?;
    println!("{:#?}", surface_attrs);
    let in_surface = va::Surface::new(
        display.clone(),
        va::RtFormat::YUV420,
        width,
        height,
        Some(va::Fourcc::try_from("NV12").unwrap()),
        va::UsageHint::GENERIC,
    )?;
    let out_surface = va::Surface::new(
        display.clone(),
        va::RtFormat::RGB32,
        width,
        height,
        Some(va::Fourcc::try_from("RGB32").unwrap()),
        va::UsageHint::GENERIC,
    )?;
    let context = va::Context::new(
        config,
        width,
        height,
        va::ContextFlags::PROGRESSIVE,
        vec![out_surface.clone()],
    );
    let in_surface_image = in_surface.derive_image()?;
    let out_surface_image = out_surface.derive_image()?;
    println!("{:#?}", in_surface_image);
    println!("{:#?}", out_surface_image);

    Ok(())
}
