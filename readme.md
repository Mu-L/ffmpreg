`ffmpreg` is a rust-native multimedia toolkit that decodes, transforms, and
encodes audio and video without requiring FFmpeg. It provides both a
command-line interface and a library API, allowing you to process media files
directly from the terminal or integrate media processing into Rust applications.
The project aims to deliver a safe, deterministic, and modular media processing
engine with a focus on explicit pipelines and developer control.

## Installation

To install the command-line tool, run:

```bash
cargo install ffmpreg
```

This downloads and compiles `ffmpreg`, placing the binary in your Cargo bin
directory.

To use `ffmpreg` as a library in your Rust project, add it to your `Cargo.toml`
dependencies:

```toml
[dependencies]
ffmpreg = "0.1"
```

## Getting Started

The simplest operation is transcoding a file from one format to another. For
example, reading a WAV file and writing it back:

```bash
ffmpreg -i input.wav -o output.wav
```

You can apply transforms to modify the media during processing. Transforms
execute in the order they appear and can perform operations such as gain
adjustment or normalization:

```bash
ffmpreg -i input.wav -o output.wav --apply gain=2.0 --apply normalize
```

To inspect a media file without transcoding, use the `--show` flag. This prints
frame-level metadata, similar to `ffprobe`, including timing, sample count,
channel configuration, and sample rate:

```bash
ffmpreg -i input.wav --show
```

Output example:

```
Frame 0: pts=0, samples=1024, channels=2, rate=44100
Frame 1: pts=1024, samples=1024, channels=2, rate=44100
```

Batch processing is supported using glob patterns. Each file is processed
independently, enabling parallel execution:

```bash
ffmpreg -i "folder/*.wav" -o out/
```

## Library

The library exposes the same primitives used internally by the CLI. A pipeline
reads packets from a container, decodes them into frames, optionally applies
transforms, encodes the frames, and writes them to an output container.

Example WAV transcoding pipeline:

```rust
use ffmpreg::container::{WavReader, WavWriter};
use ffmpreg::codecs::{PcmDecoder, PcmEncoder};
use ffmpreg::core::{Decoder, Encoder, Demuxer, Muxer, Timebase};
use std::fs::File;

fn main() -> std::io::Result<()> {
    let input = File::open("input.wav")?;
    let mut reader = WavReader::new(input)?;
    let format = reader.format();

    let output = File::create("output.wav")?;
    let mut writer = WavWriter::new(output, format)?;

    let mut decoder = PcmDecoder::new(format);
    let mut encoder = PcmEncoder::new(Timebase::new(1, format.sample_rate));

    while let Some(packet) = reader.read_packet()? {
        if let Some(frame) = decoder.decode(packet)? {
            if let Some(out_packet) = encoder.encode(frame)? {
                writer.write_packet(out_packet)?;
            }
        }
    }

    writer.finalize()?;
    Ok(())
}
```

## Core Concepts

Media processing revolves around two main data types: `Packet` and `Frame`. A
`Packet` represents encoded data in a container, while a `Frame` represents
decoded data ready for processing or playback. Data flows predictably: readers
demux containers into packets, decoders convert packets into frames, transforms
modify frames, encoders convert frames back into packets, and writers mux
packets into output files.

Example `Frame` structure:

```rust
pub struct Frame {
    pub data: Vec<u8>,
    pub pts: i64,
    pub timebase: Timebase,
    pub sample_rate: u32,
    pub channels: u8,
    pub nb_samples: usize,
}
```

Example `Packet` structure:

```rust
pub struct Packet {
    pub data: Vec<u8>,
    pub pts: i64,
    pub dts: i64,
    pub timebase: Timebase,
    pub stream_index: usize,
}
```

## Transforms

Transforms modify frames in the pipeline. Built-in transforms include `Gain` for
multiplying sample amplitudes and `Normalize` for scaling samples to a target
peak.

Example of applying transforms:

```rust
use ffmpreg::transform::{Gain, Normalize};
use ffmpreg::core::Transform;

let mut gain = Gain::new(2.0);
let mut normalize = Normalize::new(0.95);

let frame = decoder.decode(packet)?.unwrap();
let frame = gain.apply(frame)?;
let frame = normalize.apply(frame)?;
```

Custom transforms can be implemented by the `Transform` trait:

```rust
pub struct Invert;

impl Transform for Invert {
    fn apply(&mut self, mut frame: Frame) -> Result<Frame> {
        for i in (0..frame.data.len()).step_by(2) {
            let sample = i16::from_le_bytes([frame.data[i], frame.data[i + 1]]);
            let inverted = (-sample).to_le_bytes();
            frame.data[i] = inverted[0];
            frame.data[i + 1] = inverted[1];
        }
        Ok(frame)
    }

    fn name(&self) -> &'static str {
        "invert"
    }
}
```

CLI transforms are specified as `name=value` pairs:

```bash
ffmpreg -i input.wav -o output.wav --apply gain=1.5 --apply normalize=0.9
```

## Formats

- **WAV**: uncompressed audio, suitable for lossless pipelines.
- **Y4M**: raw video frames with a text-based header.

Codecs:

- **PCM**: uncompressed 16-bit audio, multi-channel support.
- **ADPCM**: adaptive differential audio compression.
- **Raw video**: passes through YUV frame data unchanged.

Video processing follows the same pipeline as audio:

```rust
use ffmpreg::container::{Y4mReader, Y4mWriter};
use ffmpreg::codecs::{RawVideoDecoder, RawVideoEncoder};

let input = File::open("input.y4m")?;
let mut reader = Y4mReader::new(input)?;
let output = File::create("output.y4m")?;
let mut writer = Y4mWriter::new(output, reader.format().clone())?;
```

Format metadata is accessible via the reader:

```rust
let format = reader.format();
println!("Resolution: {}x{}", format.width, format.height);
println!("Frame rate: {}/{}", format.framerate_num, format.framerate_den);
```

## CLI Reference

- `-i`: input file or glob pattern.
- `-o`: output file or directory.
- `--show`: inspection mode, prints frame metadata.
- `--apply`: add transform to pipeline (multiple allowed).
- `--codec`: select output codec (default matches input).

## Goals

`ffmpreg` aims to be a safe, reliable, and maintainable Rust-native alternative
to FFmpeg. The project emphasizes deterministic and explicit pipelines, clear
developer control, and minimal runtime surprises. All development prioritizes
safety by avoiding `unsafe` code wherever possible, minimizing `unwrap`
usage, reducing panics, and keeping external dependencies to a minimum. The
goal is to provide a high-performance media processing engine that is easy to
understand, integrate, and extend while maintaining strong Rust safety
guarantees.

---

Simples: AVI, FLV, OGV

Média: MOV, MP4 (inicialmente H264/AAC)

Alta: MKV, MXF, TS

Simples: WAV

Média: MP3, FLAC, AAC, ALAC

Alta: OPUS, OGG com Vorbis/Opus

## 1. Containers de Áudio (somente áudio)

| Container | Codecs típicos                | Complexidade | Uso                             |
| --------- | ----------------------------- | ------------ | ------------------------------- |
| mp3       | mp3                           | Simples      | Muito usado                     |
| wav       | pcm_s16le, pcm_s24le, pcm_f32 | Simples      | Muito usado                     |
| flac      | flac                          | Simples      | Muito usado                     |
| m4a       | aac, alac                     | Média        | Muito usado (iTunes, streaming) |
| aac       | aac                           | Média        | Muito usado (streaming)         |
| opus      | opus                          | Média        | Médio                           |
| alac      | alac                          | Média        | Médio                           |
| ogg       | vorbis, opus                  | Média        | Médio                           |

---

## 2. Containers de Vídeo Simples

| Container | Video Codecs | Audio Codecs | Legend   | Complexidade | Uso         |
| --------- | ------------ | ------------ | -------- | ------------ | ----------- |
| avi       | mpeg4, h264  | mp3, aac     | Nenhum   | Simples      | Muito usado |
| mov       | h264, h265   | aac, mp3     | mov_text | Simples      | Muito usado |
| flv       | h264, vp6    | mp3          | Nenhum   | Simples      | Médio       |
| ogv       | theora       | vorbis       | Nenhum   | Simples      | Médio       |

---

## 3. Containers de Vídeo Média/Alta Complexidade

| Container | Video Codecs         | Audio Codecs           | Legend        | Complexidade | Uso         |
| --------- | -------------------- | ---------------------- | ------------- | ------------ | ----------- |
| mp4       | h264, h265           | aac, mp3               | mov_text      | Média        | Muito usado |
| mkv       | h264, h265, vp9, av1 | aac, vorbis, opus, mp3 | srt, ass, vtt | Alta         | Muito usado |
| webm      | vp8, vp9, av1        | vorbis, opus           | vtt           | Média        | Médio       |
| mxf       | mpeg2, h264, h265    | pcm_s16le, aac         | Nenhum        | Alta         | Médio       |
| ts        | h264, h265, mpeg2    | aac, mp2               | Nenhum        | Média        | Médio       |

---

### Ordem sugerida de implementação

1. **Áudio simples**: mp3, wav, flac
2. **Áudio média complexidade**: m4a, aac, opus, alac, ogg
3. **Vídeo simples**: avi, mov, flv, ogv
4. **Vídeo média/alta complexidade**: mp4, mkv, webm, mxf, ts

## 1. Containers de Áudio

| Container | Codecs prioridade             |
| --------- | ----------------------------- |
| mp3       | mp3                           |
| wav       | pcm_s16le, pcm_s24le, pcm_f32 |
| flac      | flac                          |
| m4a       | aac, alac                     |
| aac       | aac                           |
| opus      | opus                          |
| alac      | alac                          |
| ogg       | vorbis, opus                  |

---

## 2. Containers de Vídeo Simples

| Container | Video       | Audio    | Legend   |
| --------- | ----------- | -------- | -------- |
| avi       | mpeg4, h264 | mp3, aac | nenhum   |
| mov       | h264, h265  | aac, mp3 | mov_text |
| flv       | h264, vp6   | mp3      | nenhum   |
| ogv       | theora      | vorbis   | nenhum   |

---

## 3. Containers de Vídeo Média/Alta Complexidade

| Container | Video                | Audio                  | Legend        |
| --------- | -------------------- | ---------------------- | ------------- |
| mp4       | h264, h265           | aac, mp3               | mov_text      |
| mkv       | h264, h265, vp9, av1 | aac, vorbis, opus, mp3 | srt, ass, vtt |
| webm      | vp8, vp9, av1        | vorbis, opus           | vtt           |
| mxf       | mpeg2, h264, h265    | pcm_s16le, aac         | nenhum        |
| ts        | h264, h265, mpeg2    | aac, mp2               | nenhum        |

---

### Prioridade de implementação

1. **Áudio simples** (mp3, wav, flac) → foco nos codecs principais (`mp3`, PCM, `flac`).
2. **Áudio média complexidade** (m4a, aac, opus, alac, ogg) → foco nos codecs listados primeiro.
3. **Vídeo simples** (avi, mov, flv, ogv) → priorizar codecs mais comuns (`h264`, `mpeg4`, `aac`).
4. **Vídeo média/alta complexidade** (mp4, mkv, webm, mxf, ts) → implementar `h264` e `aac` primeiro, depois adicionar VP9, AV1, Opus, etc.

Exemplos coerentes com a CLI que definimos (flags explícitas, sem strings complexas).

---

## WAV (áudio puro)

**Converter WAV → WAV ajustando volume**

```
ffmpreg -i input.wav -o out.wav --audio volume=0.5
```

**Converter WAV → AAC (codec explícito)**

```
ffmpreg -i input.wav -o out.aac --audio codec=aac volume=0.8
```

**Alterar formato PCM**

```
ffmpreg -i input.wav -o out.wav --audio format=pcm_s24le
```

---

## MKV (container multimídia)

**Escalar vídeo + ajustar volume de todas as faixas de áudio**

```
ffmpreg -i input.mkv -o out.mkv \
  --video scale=1280x720 \
  --audio volume=0.7
```

**Ajustar volume apenas da faixa de áudio 1**

```
ffmpreg -i input.mkv -o out.mkv \
  --audio track=1 volume=0.5
```

**Selecionar codec de saída**

```
ffmpreg -i input.mkv -o out.mkv \
  --video codec=h264 \
  --audio codec=aac
```

---

## MKV → WAV (extração de áudio)

**Extrair áudio de todas as faixas**

```
ffmpreg -i input.mkv -o out.wav --audio
```

**Extrair apenas uma faixa**

```
ffmpreg -i input.mkv -o out.wav --audio track=2
```

---

## Áudio sem vídeo (ex: MP3, AAC)

**AAC → WAV**

```
ffmpreg -i input.aac -o out.wav
```

**MP3 → AAC com volume**

```
ffmpreg -i input.mp3 -o out.aac --audio volume=1.2
```

---

## Legendas (MKV)

**Traduzir legenda**

```
ffmpreg -i input.mkv -o out.mkv \
  --subtitle lang=en translate=pt-br
```

**Erro esperado (exemplo)**

```
error: multiple subtitle tracks with same language (pt-br)
```
