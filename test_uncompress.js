// This is a short Node.js script which can be used to uncompress files
// compressed with Snappy framed compression.
//
// Usage:
//   npm install snappy-stream
//   node snappy_uncompress.js < foo.txt.sz > foo.txt

var snappyStream = require('snappy-stream'),
    compressStream = snappyStream.createCompressStream();

compressStream.pipe(process.stdout);
process.stdin.pipe(compressStream);
