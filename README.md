(**Work in progress:** Good enough for basic use, but may panic on corrupt
input, and `SnappyFramedEncoder` assumes you normally `write` large
buffers.  APIs may change before 1.0.)

[API documentation][doc]

Rust support for [Snappy framed compression][framed], a streaming version
of Google's high-speed [Snappy][snappy] compression scheme.  We provide
standard implementations of `Read` and `Write` for working with these
streams.

Benchmarked at 807 MB/s with CRC32C checksums disabled:

```
test read::benches::decompress_speed ... bench:    972211 ns/iter (+/- 213429) = 807 MB/s
```

To use, add the following to the `[dependencies]` section of your
`Cargo.toml` file:

```
snappy_framed = "*"
```

And import it as follows at the top-level of your crate:

```
extern crate snappy_framed;
```

You may also be interested in [`snzip`][snzip], a command-line tool for
compressing and decompressing the [Snappy framed][framed] format.

### Benchmarking notes

Using a nightly build of Rust, it's possible to profile and optimize the
decompressor:

```sh
cargo bench --features unstable
valgrind --tool=callgrind target/release/snappy_framed-* \
  --bench decompress_speed
kcachegrind callgrind.out.*
```

Note that, by default, Rust and LLVM are extremely aggressive about
inlining, which makes the profile hard to read.  You may need to add
`#[inline(never)]` to various functions to make them show up separately on
the report.

//! [doc]: http://emk.github.io/snappy_framed-rs/snappy_framed/index.html
//! [snappy]: http://code.google.com/p/snappy/
//! [framed]: http://code.google.com/p/snappy/source/browse/trunk/framing_format.txt
//! [snzip]: https://github.com/kubo/snzip
