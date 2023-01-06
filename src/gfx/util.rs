/// Blit a RGB paletted image into an RGBA output buffer
pub fn blit_with_palette(
    width: usize, height: usize,
    pallete: &[u8; 256*3],
    src: &[u8], dst: &mut [u8])
{
    let bpp = 4;
    let src_pitch = width;
    let dst_pitch = width*bpp;

    let min_y = 0usize;
    let max_y = height;

    let min_x = 0usize;
    let max_x = width;

    let mut dst_row = min_x*bpp + min_y*dst_pitch;
    let mut src_row = (height-1)*src_pitch;
    for _y in min_y..max_y {
        let mut dst_pixel = dst_row;
        let mut src_pixel = src_row;
        for _x in min_x..max_x {
            let value = src[src_pixel] as usize;

            dst[dst_pixel + 0] = pallete[value*3 + 0];
            dst[dst_pixel + 1] = pallete[value*3 + 1];
            dst[dst_pixel + 2] = pallete[value*3 + 2];
            dst[dst_pixel + 3] = 0xFF;
            
            dst_pixel += bpp;
            src_pixel += 1;
        }
        dst_row += dst_pitch;
        src_row = usize::max(src_row.wrapping_sub(src_pitch), 0);
    }
}

/// Blit a RGB paletted image into an RGBA output buffer, with a transparency index
pub fn blit_with_palette_and_transparency(
    width: usize, height: usize,
    palette: &[u8; 256*3], transparency: u8,
    src: &[u8], dst: &mut [u8])
{
    let bpp = 4;
    let src_pitch = width;
    let dst_pitch = width*bpp;

    let min_y = 0usize;
    let max_y = height;

    let min_x = 0usize;
    let max_x = width;

    let mut dst_row = min_x*bpp + min_y*dst_pitch;
    let mut src_row = (height-1)*src_pitch;
    for _y in min_y..max_y {
        let mut dst_pixel = dst_row;
        let mut src_pixel = src_row;
        for _x in min_x..max_x {
            let value = src[src_pixel] as usize;
            let alpha = if value == transparency as usize {
                0x00
            } else {
                0xFF
            };

            dst[dst_pixel + 0] = palette[value*3 + 0];
            dst[dst_pixel + 1] = palette[value*3 + 1];
            dst[dst_pixel + 2] = palette[value*3 + 2];
            dst[dst_pixel + 3] = alpha;
            
            dst_pixel += bpp;
            src_pixel += 1;
        }
        dst_row += dst_pitch;
        src_row = usize::max(src_row.wrapping_sub(src_pitch), 0);
    }
}
