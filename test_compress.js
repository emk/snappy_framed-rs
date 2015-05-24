// This is a short Node.js script which can be used to generate test
// files compressed with Snappy framed compression.
//
// Usage:
//   npm install snappy-stream
//   node snappy_compress.js < foo.txt > foo.txt.sz

var snappyStream = require('snappy-stream'),
    compressStream = snappyStream.createCompressStream();

compressStream.pipe(process.stdout);
process.stdin.pipe(compressStream);
