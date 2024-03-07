# tracing-replay

Replay `tracing` recordings.

## Overview

The `tracing-replay` crate is the matching pair of the [`tracing-rec`] crate.

`tracing-rec` provides a [`tracing-subscriber`] layer which can record event and span traces
into a serialized format.

`tracing-replay` can then take the serialized format and replay it into the current
[`tracing`] dispatcher.

## Usage

The recorded traces from a file at a provided path will be replayed into the current
[`Dispatch`].

```rust
let mut replay = tracing_replay::Replay::new();
let result = replay.replay_file(recording_path);

println!("{:?}", result);
assert!(result.is_ok());
```

## Supported Rust Versions

`tracing-replay` is built against the latest stable release. The minimum supported version is
1.76. The current version of `tracing-replay` is not guaranteed to build on Rust versions
earlier than the minimum supported version.

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/hds/tracing-rec-replay/blob/main/LICENSE

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion
in `tracing-replay` by you, shall be licensed as MIT, without any additional terms or
conditions.

[`Dispatch`]: https://docs.rs/tracing/latest/tracing/dispatcher/struct.Dispatch.html
[`tracing-rec`]: ../tracing-rec/
[`tracing-subscriber`]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/