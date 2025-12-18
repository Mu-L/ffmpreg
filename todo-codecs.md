# Codecs TODO - Implementação Mínima (Zero/Baixa Dependência)

## Status Atual

### ✅ Implementados (Funcionais)

| Codec | Tipo | Encoder | Decoder | Notas |
|-------|------|---------|---------|-------|
| PCM (s16le) | Audio | ✅ | ✅ | Passthrough, sem compressão |
| ADPCM (IMA) | Audio | ✅ | ✅ | Compressão 4:1, implementação completa |
| ADPCM (MS) | Audio | ✅ | ✅ | MS ADPCM variant |
| G.711 µ-law | Audio | ✅ | ✅ | Compressão 2:1, telefonia |
| G.711 A-law | Audio | ✅ | ✅ | Compressão 2:1, telefonia |
| FLAC | Audio | ✅ | ✅ | Fixed prediction encoder, full decoder |
| MP3 | Audio | ❌ | ✅ | Decoder pure Rust implementation |
| Vorbis | Audio | ❌ | ✅ | Decoder via lewton |
| RawVideo | Video | ✅ | ✅ | Passthrough YUV, sem compressão |

### ❌ Não Implementados (Apenas Container/Demux)

| Codec | Tipo | Container | Problema |
|-------|------|-----------|----------|
| AAC | Audio | ✅ MP4/M4A | Sem decoder/encoder real |
| H.264/AVC | Video | ✅ MP4/AVI | Sem decoder/encoder real |

---

## Prioridade 1: Audio Codecs (Zero Dependência)

### 1.1 µ-law / A-law (G.711) ✅
- **Status**: Implementado
- **Complexidade**: Muito baixa
- **Dependências**: Nenhuma
- **Compressão**: 2:1 (16-bit → 8-bit)
- **Uso**: Telefonia, WAV

```
codecs/g711/
├── mod.rs      # Lookup tables ✅
├── ulaw.rs     # µ-law encode/decode ✅
└── alaw.rs     # A-law encode/decode ✅
```

### 1.2 IMA ADPCM (Melhorias) ✅
- **Status**: Implementado com MS variant
- **TODO**: 
  - [x] Suporte a block alignment (WAV ADPCM)
  - [x] MS ADPCM variant
  - [ ] Validação contra referências

### 1.3 FLAC Decoder ✅
- **Status**: Implementado
- **Complexidade**: Média-Alta
- **Dependências**: Nenhuma (pure Rust)
- **Componentes**:
  - [x] Frame header parsing
  - [x] Subframe types (constant, verbatim, fixed, lpc)
  - [x] Rice coding
  - [ ] CRC validation
  - [x] Decorrelation (stereo)

```
codecs/flac/
├── mod.rs         ✅
├── decode.rs      # FlacDecoder ✅
├── frame.rs       # Frame/subframe parsing ✅
├── rice.rs        # Rice entropy coding ✅
└── lpc.rs         # LPC prediction ✅
```

### 1.4 FLAC Encoder ✅
- **Status**: Implementado (Fixed prediction)
- **Complexidade**: Alta
- **Dependências**: Nenhuma
- **Componentes**:
  - [x] Frame construction
  - [x] LPC analysis (Levinson-Durbin)
  - [x] Rice parameter estimation
  - [ ] MD5 computation

---

## Prioridade 2: Audio Codecs (Mínima Dependência)

### 2.1 MP3 Decoder ✅
- **Status**: Implementado (Pure Rust, zero dependências)
- **Complexidade**: Alta
- **Dependência**: Nenhuma

```
codecs/mp3/
├── mod.rs           ✅
├── bits.rs          ✅ Bitstream reader
├── decode.rs        ✅ Main decoder wrapper
├── header.rs        ✅ Frame header parsing
├── huffman.rs       ✅ Huffman decoding
├── layer3.rs        ✅ Layer III decoding
├── sideinfo.rs      ✅ Side info parsing
├── synth.rs         ✅ Synthesis filterbank & IMDCT
└── tables.rs        ✅ Lookup tables (Huffman, scalefactors, etc)
```

### 2.2 MP3 Encoder
- **Complexidade**: Muito Alta
- **Opções**:
  1. **lame** bindings (qualidade profissional)
  2. **shine** (encoder simples, ~2000 linhas C)
- **Recomendação**: Implementar encoder CBR simples

### 2.3 Vorbis Decoder ✅
- **Status**: Implementado via `lewton` crate
- **Complexidade**: Muito Alta
- **Dependência**: `lewton = "0.10"`

```
codecs/vorbis/
├── mod.rs         ✅
└── decode.rs      # Wrapper lewton ✅
```

### 2.4 Opus Decoder/Encoder
- **Complexidade**: Extrema
- **Recomendação**: `opus` crate (bindings libopus)
- **Alternativa futura**: Pure Rust em desenvolvimento

---

## Prioridade 3: Video Codecs

### 3.1 Motion JPEG (MJPEG)
- **Complexidade**: Média
- **Dependências**: Nenhuma ou `jpeg-decoder`
- **Uso**: AVI, câmeras
- **Componentes**:
  - [ ] JPEG baseline decoder
  - [ ] JPEG encoder
  - [ ] YUV ↔ RGB conversion

```
codecs/mjpeg/
├── mod.rs
├── decode.rs
├── encode.rs
├── dct.rs         # DCT/IDCT
├── huffman.rs     # Huffman tables
└── quantize.rs    # Quantization
```

### 3.2 H.264 Decoder (Baseline apenas)
- **Complexidade**: Extrema
- **Dependências**: Recomendado `openh264` ou `dav1d` pattern
- **Tamanho**: ~20000+ linhas
- **NÃO recomendado** para implementação pure Rust inicial

### 3.3 VP8/VP9 Decoder
- **Complexidade**: Muito Alta
- **Recomendação**: Usar crates existentes se necessário

---

## Ordem de Implementação Sugerida

```
Fase 1 (Zero Dep, ~1 semana): ✅ COMPLETO
├── [x] PCM s16le (done)
├── [x] ADPCM IMA (done)  
├── [x] G.711 µ-law
├── [x] G.711 A-law
└── [x] ADPCM MS variant

Fase 2 (Zero Dep, ~2-3 semanas): ✅ COMPLETO
├── [x] FLAC decoder
└── [x] FLAC encoder (básico)

Fase 3 (Mínima Dep, ~1-2 semanas): ✅ COMPLETO
├── [x] MP3 decoder (minimp3)
└── [x] Vorbis decoder (lewton)

Fase 4 (Opcional):
├── [ ] MP3 encoder (shine-like)
├── [ ] MJPEG decoder
├── [ ] AAC decoder (fdk-aac ou pure)
└── [ ] Opus decoder/encoder
```

---

## Estrutura Proposta

```
src/codecs/
├── mod.rs
├── pcm/           ✅ Completo
├── adpcm/         ✅ Completo (IMA + MS variant)
├── rawvideo/      ✅ Completo
├── g711/          ✅ Completo
│   ├── mod.rs
│   ├── ulaw.rs
│   └── alaw.rs
├── flac/          ✅ Completo
│   ├── mod.rs
│   ├── decode.rs
│   ├── encode.rs
│   ├── frame.rs
│   ├── rice.rs
│   └── lpc.rs
├── mp3/           ✅ Decoder (minimp3)
│   ├── mod.rs
│   └── decode.rs
└── vorbis/        ✅ Decoder (lewton)
    ├── mod.rs
    └── decode.rs
```

---

## Conversões que Funcionam

Implementados:

| Codec | Conversões |
|-------|------------|
| G.711 | WAV ↔ WAV (µ-law/A-law), telefonia ✅ |
| FLAC | FLAC → WAV, WAV → FLAC ✅ |
| MP3 | MP3 → WAV ✅ |
| Vorbis | OGG → WAV ✅ |

Pendentes:

| Codec | Conversões |
|-------|------------|
| MP3 encoder | WAV → MP3 |
| AAC decoder | AAC → WAV |
| Opus | OGG Opus ↔ WAV |

---

## Notas de Implementação

### Performance
- Usar lookup tables sempre que possível
- Evitar alocações em hot paths
- SIMD opcional via `std::arch` para DCT/FFT

### Testing
- Roundtrip tests: encode → decode → compare
- Bit-exact tests contra referências (ffmpeg output)
- Fuzzing para parsers

### Referências
- FLAC: https://xiph.org/flac/format.html
- MP3: ISO 11172-3, ISO 13818-3
- ADPCM: https://wiki.multimedia.cx/index.php/IMA_ADPCM
- G.711: ITU-T G.711
