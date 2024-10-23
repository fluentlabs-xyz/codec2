#![no_std]
extern crate alloc;

use bytes::BytesMut;
use codec2::Codec;
use wee_alloc::WeeAlloc;

#[global_allocator]
static ALLOC: WeeAlloc = WeeAlloc::INIT;

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn main() {
    let val: u32 = 12345;
    let mut buf = BytesMut::new();
    codec2::encoder::SolidityABI::encode(&val, &mut buf, 0).unwrap();
    let _encoded = buf.freeze();
}
