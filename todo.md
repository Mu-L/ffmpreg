### IO / Pipeline

- [x] `Packet` structure
- [x] `Frame` structure
- [x] `Timebase` structure
- [x] Create pipeline connector: read → decode → encode → write
- [x] Test pipeline with simple WAV file
- [x] Support batch processing (multiple files, wildcards)

### Containers (Audio)

- [x] WAV read/write
- [x] FLAC read/write
- [x] MP3 read/write (decode first)
- [x] OGG Vorbis read/write
- [x] Roundtrip validation for core audio containers
- [ ] Support stdin/stdout for audio
- [ ] Auto-detect audio format

### Containers (Video)

- [x] Y4M read/write
- [x] AVI read/write
- [x] MP4 read/write
- [ ] MKV read/write
- [ ] WebM read/write
- [ ] Roundtrip validation for core video containers
- [ ] Support multiple streams (audio + video + subtitles)
- [ ] Stream selection via CLI

### Codecs (Audio)

- [x] PCM decode + encode
- [x] ADPCM decode + encode
- [x] FLAC decode + encode
- [x] MP3 Layer3 decode
- [x] G.711 µ-law & A-law utils
- [ ] Opus decode + encode
- [ ] AAC decode
- [ ] WMA decode
- [ ] AC3/E-AC3 decode
- [ ] DTS decode
- [x] Skip ID3v2 when reading
- [ ] MP3 encoding
- [ ] VBR support
- [ ] Full ID3v2 read/write
- [ ] Gapless playback info
- [ ] Error recovery for corrupted frames

### Codecs (Video)

- [x] Raw Video/YUV decode + encode
- [ ] H.264/AVC decode
- [ ] VP9 decode
- [ ] AV1 decode
- [ ] HEVC/H.265 decode

### Streams

- [ ] multiple streams per container (audio, video, subtitles)
- [ ] Stream selection via CLI e.g (`--audio 0`, `--video 1`, `--subtitle 0`)
- [ ] Demux streams into individual Packets
- [ ] Synchronize streams (audio/video PTS/DTS)
- [ ] Real-time decoding of selected streams
- [ ] Switch streams dynamically during playback
- [ ] Stream iteration API for pipeline
- [ ] Unit tests for multi-stream containers
- [ ] Network streaming: HTTP, RTSP, HLS, DASH
- [ ] Real-time buffering and jitter compensation for network streams

### Subtitles

- [ ] Support SRT subtitles
- [ ] Support ASS/SSA subtitles
- [ ] Support embedded subtitles in containers (MKV, MP4)
- [ ] Subtitle track selection and switching
- [ ] Subtitle rendering API for pipeline integration

### Transforms (Audio)

- [x] Volume/Gain
- [x] Normalize
- [x] Fade In/Out / Crossfade
- [x] Channel Mixer
- [x] Resample
- [ ] EQ
- [x] Support chaining multiple audio transforms

### Transforms (Video)

- [x] Scale/Resize
- [x] Rotate
- [x] Crop / Pad
- [x] Brightness / Contrast
- [x] Framerate Converter

### CLI

- [x] Basic args: -i, -o, --apply, --show, --codec
- [x] Transform chaining via multiple `--apply` flags
- [x] Batch processing (wildcards, directories)
- [x] Format auto-detection
- [x] Show frame info (`--show`, `--json`, `--stream`, `--frames`)
- [ ] --dry-run
- [ ] --verbose
- [ ] --force overwrite
- [ ] --quality / --bitrate flags
- [ ] --metadata preserve/strip
- [ ] Progress bar
- [x] stdin/stdout support (-i -, -o -)

### Testing & Quality

- [x] Unit tests for core structures
- [x] Tests for audio transforms
- [x] Tests for video transforms
- [x] Tests for audio containers
- [x] Tests for video containers
- [x] Tests for codecs
- [x] Pipeline integration tests
- [x] CLI argument parsing tests
- [ ] Roundtrip validation for all containers/codecs
- [ ] Edge case error handling
- [ ] Large file stress testing

### Advanced / Optional

- [ ] YUV ↔ RGB conversion
- [ ] Chroma subsampling
- [ ] Streaming / pipe / unbuffered mode
- [ ] Parallel transform execution
- [ ] Multi-stream selection dynamic
- [ ] Advanced audio effects: Compressor / Expander / Noise Gate
- [ ] Advanced audio: Delay / Reverb / Chorus / Phaser
- [ ] Advanced audio: Multiband / Sidechain / Saturation / Distortion
- [ ] Full metadata support: ID3v2, Vorbis comments, MP4/iTunes, WAV LIST INFO

