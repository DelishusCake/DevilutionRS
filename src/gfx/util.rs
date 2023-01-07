/// NOTE: These two functions use old-school pointer arithmetic and unchecked de-referencing
/// to achieve the best blit performance possible.

/// Blit a RGB paletted image into an RGBA output buffer
pub unsafe fn blit_with_palette(
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

    let mut dst_row = dst[(min_x*bpp + min_y*dst_pitch)..].as_mut_ptr();
    let mut src_row = src[(height-1)*src_pitch..].as_ptr();
    for _y in min_y..max_y {
        let mut dst_pixel = dst_row;
        let mut src_pixel = src_row;
        for _x in min_x..max_x {
            let value = *src_pixel as usize;

            *dst_pixel.add(0) = *pallete.get_unchecked(value*3 + 0);
            *dst_pixel.add(1) = *pallete.get_unchecked(value*3 + 1);
            *dst_pixel.add(2) = *pallete.get_unchecked(value*3 + 2);
            *dst_pixel.add(3) = 0xFF;
            
            dst_pixel = dst_pixel.add(bpp);
            src_pixel = src_pixel.add(1);
        }
        dst_row = dst_row.add(dst_pitch);
        src_row = src_row.sub(src_pitch);
    }
}

/// Blit a RGB paletted image into an RGBA output buffer, with a transparency index
pub unsafe fn blit_with_palette_and_transparency(
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

    let mut dst_row = dst[(min_x*bpp + min_y*dst_pitch)..].as_mut_ptr();
    let mut src_row = src[(height-1)*src_pitch..].as_ptr();
    for _y in min_y..max_y {
        let mut dst_pixel = dst_row;
        let mut src_pixel = src_row;
        for _x in min_x..max_x {
            let value = *src_pixel as usize;
            let mask = if value == transparency as usize {
                0x00
            } else {
                0xFF
            };

            *dst_pixel.add(0) = mask & palette.get_unchecked(value*3 + 0);
            *dst_pixel.add(1) = mask & palette.get_unchecked(value*3 + 1);
            *dst_pixel.add(2) = mask & palette.get_unchecked(value*3 + 2);
            *dst_pixel.add(3) = mask & 0xFF;
            
            dst_pixel = dst_pixel.add(bpp);
            src_pixel = src_pixel.add(1);
        }
        dst_row = dst_row.add(dst_pitch);
        src_row = src_row.sub(src_pitch);
    }
}
