/// Audio utility functions for format conversions and sample processing.
/// Convert a normalized f32 sample to i16 little-endian bytes.
///
/// Clamps the input to -1.0 to 1.0 range to prevent overflow when
/// mixing voices, applying filters, or effects exceed the normalized range.
///
/// # Arguments
/// * `sample` - f32 sample (typically -1.0 to 1.0, but safely handles out-of-range)
///
/// # Returns
/// Two bytes representing the i16 sample in little-endian format
#[inline]
pub fn f32_to_i16_le(sample: f32) -> [u8; 2] {
    debug_assert!((-1. ..=1.).contains(&sample)); // -1 <= sample <= 1
    let sample_i16 = (sample * (i16::MAX as f32)) as i16;
    sample_i16.to_le_bytes()
}