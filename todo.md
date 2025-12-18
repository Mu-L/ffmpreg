io → container → codec → transform → codec → container → io

Todo

- Minimal WAV pipeline

  > End-to-end pipeline, audible audio.
  - [x] Create project and basic folders (core, containers/wav, codecs/pcm, cli)
  - [x] Implement Packet, Frame, Timebase
  - [x] Read WAV and produce Packets (containers/wav/read.rs)
  - [x] Write Packets back (containers/wav/write.rs)
  - [x] PCM passthrough codec (decode → encode)
  - [x] Connect pipeline: read → decode → encode → write
  - [x] Minimal CLI: ffmpreg -i input.wav -o output.wav
  - [x] Test with a simple WAV file

- Frame inspection / Media info

  > Show internal frame info, minimal ffprobe alternative.
  - [x] Add CLI option --show
  - [x] Iterate over Packets → Frames
  - [x] Display pts, sample count, channels, sample rate
  - [x] Test output with example WAV

- Basic transform

  > Apply simple operation on frames (e.g., gain)
  - [x] Create transforms/gain.rs
  - [x] Implement trait Transform<T>
  - [x] Integrate pipeline: read → decode → transform → encode → write
  - [x] CLI: ffmpreg --apply gain=2.0
  - [x] Test amplified audio

- Multi-file / batch

  > Process multiple files using the same pipeline
  - [x] CLI accepts multiple files or wildcard (folder/\*.wav)
  - [x] Iterate files → pipeline
  - [x] Create separate output for each file
  - [x] Test with 2-3 WAV files

- More containers

  > Add raw video support (Y4M)
  - [x] Create containers/y4m/read.rs and write.rs
  - [x] Parse Y4M header (width, height, framerate, colorspace, aspect ratio)
  - [x] Produce Packets/Frames
  - [x] Minimal pipeline: decode → encode → write
  - [x] CLI: ffmpreg -i input.y4m -o output.y4m
  - [x] Test with a Y4M file (lossless passthrough verified)

- More codecs

  > ADPCM, multi-channel PCM
  - [x] Add ADPCM codec
  - [x] Support multi-channel PCM
  - [x] Pipeline: decode → transform → encode → write
  - [x] Roundtrip tests for each codec

- Chained filters
   > Apply multiple transforms in sequence
   - [x] CLI: ffmpreg --apply gain=2.0 --apply normalize
   - [x] Create transforms/normalize.rs
   - [x] Pipeline applies filters in sequence
   - [x] Test audio with two chained filters

- Audio Containers (Complete)

   > Support multiple audio formats
   - [x] WAV format (containers/wav/read.rs, write.rs)
   - [x] FLAC format (containers/flac/read.rs, write.rs)
   - [x] OGG Vorbis (containers/ogg/read.rs, write.rs)
   - [x] MP3 format (containers/mp3/read.rs, write.rs)
   - [x] Roundtrip tests for all audio containers

- Video Containers (Complete)

   > Support video formats
   - [x] Y4M (already done above)
   - [x] AVI format (containers/avi/read.rs, write.rs)
   - [x] MP4 format (containers/mp4/read.rs, write.rs)
   - [x] Roundtrip tests for video containers

- Audio Transforms (Complete)

   > Effects and filters for audio
   - [x] Volume/Gain (transforms/volume.rs)
   - [x] Normalize (transforms/normalize.rs)
   - [x] RMS Limiter (transforms/rms_limiter.rs)
   - [x] Peak Limiter (transforms/peak_limiter.rs)
   - [x] Resample (transforms/resample.rs)
   - [x] Channel Mixer (transforms/channel_mixer.rs)
   - [x] EQ (transforms/eq.rs)
   - [x] Highpass Filter (transforms/highpass.rs)
   - [x] Lowpass Filter (transforms/lowpass.rs)
   - [x] Fade In/Out (transforms/fade.rs)
   - [x] Crossfade (transforms/fade.rs)

- Video Transforms (Complete)

   > Transformations for video frames
   - [x] Scale/Resize (transforms/video/scale.rs)
   - [x] Rotate (transforms/video/rotate.rs)
   - [x] Flip H/V (transforms/video/flip.rs)
   - [x] Crop (transforms/video/crop.rs)
   - [x] Pad (transforms/video/pad.rs)
   - [x] Brightness (transforms/video/brightness.rs)
   - [x] Contrast (transforms/video/contrast.rs)
   - [x] Blur (transforms/video/blur.rs)
   - [x] Framerate Converter (transforms/video/framerate.rs)

- Testing & Quality

   > Comprehensive test coverage
   - [x] Unit tests (217 passing)
   - [x] Transform tests (volume, normalize, filters, chain)
   - [x] Container tests (WAV, Y4M roundtrip)
   - [x] Codec tests (PCM, ADPCM)
   - [x] Pipeline integration tests
   - [x] CLI argument parsing tests
   - [x] Batch processing tests
   - [ ] Roundtrip validation for all containers
   - [ ] Error handling edge cases
   - [ ] Large file stress tests

- CLI Enhancements

   > Improve command-line interface
   - [x] Basic args: -i, -o, --apply, --show, --codec
   - [x] Transform chaining (multiple --apply flags)
   - [x] Batch processing (wildcards, directories)
   - [x] Format auto-detection
   - [ ] --dry-run (preview without writing)
   - [ ] --verbose (logging and statistics)
   - [ ] --force (overwrite existing files)
   - [ ] --quality / --bitrate flags
   - [ ] --metadata preserve|strip
   - [ ] Progress bars for batch operations

- Documentation

   > Code and user documentation
   - [ ] Rustdoc comments for public APIs
   - [ ] Transform trait documentation
   - [ ] Codec/container feature matrix
   - [ ] CLI examples and help text
   - [ ] README with examples
   - [ ] Architecture guide
   - [ ] Codec compatibility guide

- Color Space & Metadata

   > Advanced media features
   - [ ] YUV ↔ RGB conversion
   - [ ] Chroma subsampling (4:4:4 → 4:2:0)
   - [ ] ID3 tags for MP3 (read/write)
   - [ ] Vorbis comments for FLAC
   - [ ] MP4 iTunes metadata
   - [ ] WAV LIST INFO chunks

- Streaming & I/O

   > Enhanced input/output capabilities
   - [ ] Read from stdin (-i -)
   - [ ] Write to stdout (-o -)
   - [ ] Pipe support for Unix tools
   - [ ] Unbuffered mode for low-latency

- Performance & Optimization

   > Speed and memory improvements
   - [ ] Codec throughput benchmarks
   - [ ] Transform overhead profiling
   - [ ] SIMD optimizations for PCM
   - [ ] Buffer pooling/allocation reduction
   - [ ] Parallel transform execution
   - [ ] Stream-based processing (avoid full load)

- Advanced Features

   > Future extensions
   - [ ] Multi-stream support (audio + video)
   - [ ] Stream selection (--audio 0, --video 1)
   - [ ] More codecs (Opus, VP9, AV1, HEVC)
   - [ ] Effects library (Reverb, Compression, Distortion, etc.)
   - [ ] Multiband processing
   - [ ] Sidechain support
