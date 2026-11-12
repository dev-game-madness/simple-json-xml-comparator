fn main() {
    println!("cargo:rerun-if-changed=icon.svg");
    if std::env::var("CARGO_CFG_TARGET_OS").ok().as_deref() != Some("windows") {
        return;
    }

    let svg = include_str!("icon.svg");
    let sizes = [16u32, 24, 32, 48, 64, 256];
    let images: Vec<(u32, Vec<u8>)> = sizes.into_iter().map(|size| (size, render(svg, size))).collect();
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let ico_path = std::path::Path::new(&out_dir).join("icon.ico");
    std::fs::write(&ico_path, encode_ico(&images)).unwrap();

    let mut resource = winresource::WindowsResource::new();
    resource.set_icon(ico_path.to_str().unwrap());
    resource.compile().unwrap();
}

fn render(svg: &str, size: u32) -> Vec<u8> {
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default()).expect("icon.svg");
    let mut pixmap = tiny_skia::Pixmap::new(size, size).unwrap();
    let width = tree.size().width();
    let height = tree.size().height();
    let scale = size as f32 / width.max(height);
    let transform = tiny_skia::Transform::from_scale(scale, scale).post_translate(
        (size as f32 - width * scale) / 2.0,
        (size as f32 - height * scale) / 2.0,
    );
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    unpremultiply(pixmap.data())
}

fn unpremultiply(data: &[u8]) -> Vec<u8> {
    let mut pixels = data.to_vec();
    for pixel in pixels.chunks_exact_mut(4) {
        let alpha = pixel[3] as u32;
        if alpha == 0 {
            pixel[0] = 0;
            pixel[1] = 0;
            pixel[2] = 0;
        } else if alpha < 255 {
            pixel[0] = ((pixel[0] as u32 * 255) / alpha).min(255) as u8;
            pixel[1] = ((pixel[1] as u32 * 255) / alpha).min(255) as u8;
            pixel[2] = ((pixel[2] as u32 * 255) / alpha).min(255) as u8;
        }
    }
    pixels
}

fn encode_ico(images: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let mut entries = Vec::new();
    let mut blobs = Vec::new();
    let mut offset = 6 + 16 * images.len() as u32;
    for (size, rgba) in images {
        let blob = dib(*size, rgba);
        entries.push((*size, blob.len() as u32, offset));
        offset += blob.len() as u32;
        blobs.push(blob);
    }

    let mut file = Vec::new();
    file.extend_from_slice(&0u16.to_le_bytes());
    file.extend_from_slice(&1u16.to_le_bytes());
    file.extend_from_slice(&(images.len() as u16).to_le_bytes());
    for (size, length, image_offset) in entries {
        let dimension = if size >= 256 { 0 } else { size as u8 };
        file.push(dimension);
        file.push(dimension);
        file.push(0);
        file.push(0);
        file.extend_from_slice(&1u16.to_le_bytes());
        file.extend_from_slice(&32u16.to_le_bytes());
        file.extend_from_slice(&length.to_le_bytes());
        file.extend_from_slice(&image_offset.to_le_bytes());
    }
    for blob in blobs {
        file.extend_from_slice(&blob);
    }
    file
}

fn dib(size: u32, rgba: &[u8]) -> Vec<u8> {
    let mask_stride = ((size + 31) / 32) * 4;
    let mask_size = mask_stride * size;
    let xor_size = size * size * 4;
    let mut blob = Vec::new();
    blob.extend_from_slice(&40u32.to_le_bytes());
    blob.extend_from_slice(&(size as i32).to_le_bytes());
    blob.extend_from_slice(&((size as i32) * 2).to_le_bytes());
    blob.extend_from_slice(&1u16.to_le_bytes());
    blob.extend_from_slice(&32u16.to_le_bytes());
    blob.extend_from_slice(&0u32.to_le_bytes());
    blob.extend_from_slice(&(xor_size + mask_size).to_le_bytes());
    blob.extend_from_slice(&0i32.to_le_bytes());
    blob.extend_from_slice(&0i32.to_le_bytes());
    blob.extend_from_slice(&0u32.to_le_bytes());
    blob.extend_from_slice(&0u32.to_le_bytes());
    for y in 0..size {
        let source_y = size - 1 - y;
        for x in 0..size {
            let index = ((source_y * size + x) * 4) as usize;
            blob.push(rgba[index + 2]);
            blob.push(rgba[index + 1]);
            blob.push(rgba[index]);
            blob.push(rgba[index + 3]);
        }
    }
    blob.extend(std::iter::repeat(0).take(mask_size as usize));
    blob
}
