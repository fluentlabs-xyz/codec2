# FLuentbase codec v2

## Codec macro

Codec macro allows you to derrive a codec for a struct.

### Solidity

- при кодировании структуры, у которой хотя бы одно из полей динамическое, нам нужно кодировать структуру следующим образом

| Index | Field          | Description                                     |
| ----- | -------------- | ----------------------------------------------- |
| 0     | uint256        | offset                                          |
| 1     | static fields  | all static fields, the same order as in struct  |
| 2     | dynamic fields | all dynamic fields, the same order as in struct |

для солидити смещение до данных всегда будет на 32 байта меньше для динамических структур - это происходит из-за того, что оффсет учитывается от начала данных, а не от смещения. Таким образом, чтобы получить правильное смещение, нам нужно добавить 32 байта к оффсету.

let test_struct_sol = TestStructSol {
    bool_val: true,
    u8_val: 42,
    uint_val: (1000, 1_000_000, 1_000_000_000),
    int_val: (-1000, -1_000_000, -1_000_000_000),
    u256_val: U256::from(12345),
    address_val: Address::repeat_byte(0xAA),
    bytes_val: Bytes::from(vec![1, 2, 3, 4, 5]),
    vec_val: vec![10, 20, 30],
};

000      : 00 00 00 20   ||  032 |

032 000  : 00 00 00 01   ||  001 |
064 032  : 00 00 00 2a   ||  042 |
096 064  : 00 00 03 e8   ||  1000 |
128 096  : 00 0f 42 40   ||  1000000 |
160 128  : 3b 9a ca 00   ||  1000000000 |
192 160  : ff ff fc 18   ||  4294966296 |
224 192  : ff f0 bd c0   ||  4293967296 |
256 224  : c4 65 36 00   ||  3294967296 |
288 256  : 00 00 30 39   ||  12345 |
320 288  : aa aa aa aa   ||  2863311530 |
352 320  : 00 00 01 80   ||  384 |
384 352  : 00 00 01 c0   ||  448 |
416 384  : 00 00 00 05   ||  005 |
448 416  : 00 00 00 00   ||  000 |
480 448  : 00 00 00 03   ||  003 |
512 480  : 00 00 00 0a   ||  010 |
544 512  : 00 00 00 14   ||  020 |
576 544  : 00 00 00 1e   ||  030 |

let original: Vec<Vec<u32>> = vec![vec![1, 2, 3], vec![4, 5]];
solidity

000 000  : 00 00 00 20   ||  032 |
032 000  : 00 00 00 02   ||  002 |

000 000  : 00 00 00 40   ||  064 |
032 032  : 00 00 00 c0   ||  192 |
064 064  : 00 00 00 03   ||  003 |
096 096  : 00 00 00 01   ||  001 |
128 128  : 00 00 00 02   ||  002 |
160 160  : 00 00 00 03   ||  003 |
192 192  : 00 00 00 02   ||  002 |
224 224  : 00 00 00 04   ||  004 |
256 256  : 00 00 00 05   ||  005 |






0000 | 0000: 00 00 00 20   ||  000: 032 |
-----------------------------------------

0020 | 0000: 00 00 00 01   ||  032: 001 |
0040 | 0020: 00 00 00 2a   ||  064: 042 |
0060 | 0040: 00 00 03 e8   ||  096: 1000 |
0080 | 0060: 00 0f 42 40   ||  128: 1000000 |
00a0 | 0080: 3b 9a ca 00   ||  160: 1000000000 |
00c0 | 00a0: ff ff fc 18   ||  192: 4294966296 |
00e0 | 00c0: ff f0 bd c0   ||  224: 4293967296 |
0100 | 00e0: c4 65 36 00   ||  256: 3294967296 |
0120 | 0100: 00 00 30 39   ||  288: 12345 |
0140 | 0120: aa aa aa aa   ||  320: 2863311530 |

-----------------------------------------
0160 | 0140: 00 00 01 80   ||  352: 384 |  header =  (384 + 32 , 5)
0180 | 0160: 00 00 01 c0   ||  384: 448 |  header =  (448 + 32 , 3)
01a0 | 0180: 00 00 00 05   ||  416: 005 |
01c0 | 01a0: 00 00 00 00   ||  448: 000 |
01e0 | 01c0: 00 00 00 03   ||  480: 003 |
0200 | 01e0: 00 00 00 0a   ||  512: 010 |
0220 | 0200: 00 00 00 14   ||  544: 020 |
0240 | 0220: 00 00 00 1e   ||  576: 030 |
