use std::vec;

use crate::{
    encoder::{Align32, Alignment, BigEndian, Encoder, Endian, LittleEndian},
    vec::SolidityVecEncoding,
};

use alloy_primitives::Bytes;
use bytes::{buf, Buf, BufMut, BytesMut};
use hashbrown::{HashMap, HashSet};

use alloy_sol_types::{sol_data::*, SolType, SolValue};

// type MyU32 = u32;
