extern crate console_error_panic_hook;
extern crate nixpkgs_fmt;
extern crate wasm_bindgen;
extern crate wee_alloc;

use wasm_bindgen::prelude::*;
use std::panic;

// Use the smaller `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    Ok(())
}

#[wasm_bindgen]
pub fn reformat_string(text: &str) -> String {
    nixpkgs_fmt::reformat_string(text)
}
