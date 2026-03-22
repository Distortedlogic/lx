pub const SAMPLE_RATE: u32 = 16000;
pub const CHANNELS: u16 = 1;
pub const BITS_PER_SAMPLE: u16 = 16;

pub fn wrap_pcm_as_wav(raw: &[u8], sample_rate: u32, channels: u16, bits_per_sample: u16) -> Vec<u8> {
  let data_size = raw.len() as u32;
  let chunk_size = 36 + data_size;
  let byte_rate = sample_rate * u32::from(channels) * u32::from(bits_per_sample) / 8;
  let block_align = channels * bits_per_sample / 8;

  let mut wav = Vec::with_capacity(44 + raw.len());
  wav.extend_from_slice(b"RIFF");
  wav.extend_from_slice(&chunk_size.to_le_bytes());
  wav.extend_from_slice(b"WAVE");
  wav.extend_from_slice(b"fmt ");
  wav.extend_from_slice(&16u32.to_le_bytes());
  wav.extend_from_slice(&1u16.to_le_bytes());
  wav.extend_from_slice(&channels.to_le_bytes());
  wav.extend_from_slice(&sample_rate.to_le_bytes());
  wav.extend_from_slice(&byte_rate.to_le_bytes());
  wav.extend_from_slice(&block_align.to_le_bytes());
  wav.extend_from_slice(&bits_per_sample.to_le_bytes());
  wav.extend_from_slice(b"data");
  wav.extend_from_slice(&data_size.to_le_bytes());
  wav.extend_from_slice(raw);
  wav
}

struct WavHeader {
  sample_rate: u32,
  channels: u16,
  bits_per_sample: u16,
  data_offset: usize,
}

fn parse_wav_header(wav: &[u8]) -> Option<WavHeader> {
  if wav.len() < 44 || &wav[0..4] != b"RIFF" || &wav[8..12] != b"WAVE" {
    return None;
  }
  let channels = u16::from_le_bytes([wav[22], wav[23]]);
  let sample_rate = u32::from_le_bytes([wav[24], wav[25], wav[26], wav[27]]);
  let bits_per_sample = u16::from_le_bytes([wav[34], wav[35]]);

  let mut offset = 12;
  while offset + 8 <= wav.len() {
    let id = &wav[offset..offset + 4];
    let size = u32::from_le_bytes([wav[offset + 4], wav[offset + 5], wav[offset + 6], wav[offset + 7]]) as usize;
    if id == b"data" {
      return Some(WavHeader { sample_rate, channels, bits_per_sample, data_offset: offset + 8 });
    }
    offset += 8 + size;
  }
  None
}

pub fn chunk_wav(wav: &[u8], max_chunk_bytes: usize) -> Vec<Vec<u8>> {
  let Some(header) = parse_wav_header(wav) else {
    return vec![wav.to_vec()];
  };
  let data = &wav[header.data_offset..];
  let max_data_per_chunk = max_chunk_bytes.saturating_sub(44);
  if max_data_per_chunk == 0 {
    return vec![wav.to_vec()];
  }
  data.chunks(max_data_per_chunk).map(|slice| wrap_pcm_as_wav(slice, header.sample_rate, header.channels, header.bits_per_sample)).collect()
}
