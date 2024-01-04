use itertools::enumerate;

pub fn copy_nv12(
    source: &[u8],
    stride: usize,
    encoder_line_size: usize,
    encoder_num_lines: usize,
    destination: &mut [u8],
) {
    // fast path
    if stride == encoder_line_size {
        destination.copy_from_slice(source);
        return;
    }

    for (r, row) in enumerate(source.chunks(stride)) {
        destination[r * encoder_line_size..r * encoder_line_size + encoder_num_lines]
            .copy_from_slice(&row[..encoder_num_lines])
    }
}
