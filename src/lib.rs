use std::panic;
use wasm_bindgen::prelude::*;

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
pub fn reformat_string(text: &str, format: &str) -> String {
    let out = nixpkgs_fmt::reformat_string(text);

    if format == "diff" {
        let first_text = text.lines().collect::<Vec<&str>>();
        let second_text = out.lines().collect::<Vec<&str>>();
        let diff = difflib::unified_diff(&first_text, &second_text, "Input", "Output", "", "", 3);
        if diff.len() == 0 {
            return String::from("No changes found");
        } else {
            return [diff[0..3].join(""), diff[3..].join("\n")].join("");
        }
    } else {
        return out;
    }
}
