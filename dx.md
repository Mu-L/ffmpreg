````
Library DX

- Minimal WAV pipeline
 > Read WAV → decode → encode → write
```rust
use ffmpreg::containers::wav;
use ffmpreg::codecs::pcm;
use ffmpreg::core::{Packet, Frame};

let packets = wav::read("input.wav")?;
let frames: Vec<Frame<i16>> = packets.iter().map(|p| pcm::decode(p)).collect();
let out_packets: Vec<Packet> = frames.iter().map(|f| pcm::encode(f)).collect();
wav::write("output.wav", &out_packets)?;
````

- Frame inspection

> Access frame details

```rust
for frame in frames.iter() {
    println!("pts={}, samples={}, channels={}, rate={}",
        frame.pts, frame.sample_count, frame.channels, frame.sample_rate);
}
```

- Basic transform

> Apply gain to frames

```rust
use ffmpreg::transforms::gain::Gain;

let gain = Gain::new(2.0);
let transformed: Vec<Frame<i16>> = frames.iter().map(|f| gain.process(f)).collect();
```

- Multi-file / batch

> Process multiple WAV files

```rust
for file in std::fs::read_dir("folder/")? {
    let input = file.path();
    let packets = wav::read(&input)?;
    let frames: Vec<_> = packets.iter().map(|p| pcm::decode(p)).collect();
    let out_packets: Vec<_> = frames.iter().map(|f| pcm::encode(f)).collect();
    let output = format!("out/{}", input.file_name().unwrap().to_string_lossy());
    wav::write(&output, &out_packets)?;
}

// or
use ffmpreg::high::batch;

batch("in/*.wav", "out/")
    .parallel(4)
    .process(|task| {
        let packets = wav::read(task.input)?;

        let frames = pcm::decode_packets(&packets)?;
        let out_packets = pcm::encode_frames(&frames)?;

        wav::write(task.output, &out_packets)?;
        Ok(())
    })
    .run()?;


```

- Chained transforms

> Apply multiple transforms in sequence

```rust
use ffmpreg::transforms::{Normalize, Volume};

let volume = Volume::new(2.0);
let normalize = Normalize::new();

let frames: Vec<Frame> = frames.iter()
  .map(|frame| volume.apply(frame))
  .map(|frame| normalize.apply(frame)).collect();
```
