use std::io::{BufWriter, Write};

use zerocopy::{
    Immutable,
    IntoBytes,
    little_endian::{U16, U32},
};

const CHANNELS: u16 = 1;
pub const SAMPLES_PER_SECOND: u32 = 44100;
const BITS_PER_SAMPLE: u16 = 16;

pub const AVG_BYTES_PER_SECOND: u32 =
    CHANNELS as u32
        * SAMPLES_PER_SECOND
        * (BITS_PER_SAMPLE / 8) as u32;

#[derive(IntoBytes, Immutable)]
#[repr(u16)]
enum WaveFormatCategory {
    Pcm = 0x0001u16.to_le(),
}

#[derive(IntoBytes, Immutable)]
#[repr(C, packed)]
struct FormatChunkCommon<FSF> {
    format_tag: WaveFormatCategory,
    channels: U16,
    samples_per_sec: U32,
    avg_bytes_per_sec: U32,
    block_align: U16,
    format_specific: FSF,
}

#[derive(IntoBytes, Immutable)]
#[repr(C, packed)]
struct FormatChunkPcm {
    bits_per_sample: U16,
}

pub fn create_wav_buffer(
    path: &str,
    total_samples: usize
) -> Result<BufWriter<std::fs::File>, std::io::Error> {
    let sample_data_len =
        (total_samples * std::mem::size_of::<i16>())
            as u32;

    let format = FormatChunkCommon {
        format_tag: WaveFormatCategory::Pcm,
        channels: CHANNELS.into(),
        samples_per_sec: SAMPLES_PER_SECOND.into(),
        avg_bytes_per_sec: AVG_BYTES_PER_SECOND.into(),
        block_align: (CHANNELS * BITS_PER_SAMPLE / 8).into(),
        format_specific: FormatChunkPcm {
            bits_per_sample: BITS_PER_SAMPLE.into(),
        },
    };

    let file = std::fs::File::create(path)?;
    let mut out = BufWriter::new(file);

    out.write_all(b"RIFF")?;

    out.write_all(
        &(sample_data_len
            + 3 * 4
            + std::mem::size_of_val(&format) as u32)
            .to_le_bytes(),
    )?;

    out.write_all(b"WAVE")?;

    write_chunk(
        b"fmt ",
        format,
        &mut out,
    )?;

    out.write_all(b"data")?;
    out.write_all(
        &sample_data_len.to_le_bytes()
    )?;

    Ok(out)
}

fn write_chunk<T: IntoBytes + Immutable, W: Write>(
    fourcc: &[u8; 4],
    t: T,
    mut out: W,
) -> Result<(), std::io::Error> {
    out.write_all(fourcc)?;
    out.write_all(
        &(std::mem::size_of::<T>() as u32).to_le_bytes()
    )?;
    t.write_to_io(&mut out)?;
    Ok(())
}