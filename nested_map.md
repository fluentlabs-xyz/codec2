# Nested Map

```rs
let mut original = HashMap::new();
original.insert(1, HashMap::from([(5, 6)]));
original.insert(2, HashMap::from([(7, 8)]));
```

## WASM

```sh
# Outer map
0000: 02 00 00 00   ||  000: 002 | # length
0004: 14 00 00 00   ||  004: 020 | # keys offset
0008: 08 00 00 00   ||  008: 008 | # keys length (in bytes)

000c: 1c 00 00 00   ||  012: 028 | # values offset
0010: 38 00 00 00   ||  016: 056 | # values length (in bytes)

0014: 01 00 00 00   ||  020: 001 | # key 1
0018: 02 00 00 00   ||  024: 002 | # key 2

# outer map values <---- all offsets are relative to the start of the outer map values (since 028)
# inner map 1 header
001c: 01 00 00 00   ||  028: 001 | # length
0020: 28 00 00 00   ||  032: 040 | # key offset
0024: 04 00 00 00   ||  036: 004 | # keys length (in bytes)

0028: 2c 00 00 00   ||  040: 044 | # values offset
002c: 04 00 00 00   ||  044: 004 | # values length (in bytes)

# inner map 2 header
0030: 01 00 00 00   ||  048: 001 | # length
0034: 30 00 00 00   ||  052: 048 | # key offset
0038: 04 00 00 00   ||  056: 004 | # length (in bytes)

003c: 34 00 00 00   ||  060: 052 | # values offset
0040: 04 00 00 00   ||  064: 004 | # values length (in bytes)

# inner map 1 data
0044: 05 00 00 00   ||  068: 005 | # key
0048: 06 00 00 00   ||  072: 006 | # value

# inner map 2 data
004c: 07 00 00 00   ||  076: 007 | # key
0050: 08 00 00 00   ||  080: 008 | # value
```

## SOL

For solidity all offsets are relative to the current position. For example, if the current position is 0x20, and the offset is 0x10, then the actual offset is 0x30.

```sh
# Outer map

0000: 00 00 00 20   ||  000: 032 | # offset
0020: 00 00 00 02   ||  032: 002 | # length
0040: 00 00 00 40   ||  064: 064 | # keys offset 064+064 = 128
0060: 00 00 00 80   ||  096: 128 | # values offset 96+128 = 224

# Outer map keys
0080: 00 00 00 02   ||  128: 002 | # keys length (elements)
00a0: 00 00 00 01   ||  160: 001 | # key 1
00c0: 00 00 00 02   ||  192: 002 | # key 2

# Outer map values
00e0: 00 00 00 02   ||  224: 002 | # values length
# Inner map 1 header
0100: 00 00 00 20   ||  256: 032 | # offset 256+32 = 288
0120: 00 00 00 01   ||  288: 001 | # length
0140: 00 00 00 c0   ||  320: 192 | # keys offset 320+192 = 512
0160: 00 00 00 e0   ||  352: 224 | # values offset 352+224 = 576

# Inner map 2 header
0180: 00 00 00 a0   ||  384: 032 | # offset  384+32 = 416
01a0: 00 00 00 01   ||  416: 001 | # length
01c0: 00 00 00 c0   ||  448: 192 | # keys offset 448+192 = 640
01e0: 00 00 00 e0   ||  480: 224 | # values offset 480+224 = 704

# Inner map 1 keys
0200: 00 00 00 01   ||  512: 001 | # length
0220: 00 00 00 05   ||  544: 005 | # key

# Inner map 1 values
0240: 00 00 00 01   ||  576: 001 | # length
0260: 00 00 00 06   ||  608: 006 | # value

# Inner map 2 keys
0280: 00 00 00 01   ||  640: 001 | # length
02a0: 00 00 00 07   ||  672: 007 | # key

# Inner map 1 values
02c0: 00 00 00 01   ||  704: 001 | # length
02e0: 00 00 00 08   ||  736: 008 | # value
```
