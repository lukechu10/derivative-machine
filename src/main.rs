#![recursion_limit = "512"]
#![feature(box_patterns)]
#![feature(or_patterns)]

use std::error::Error;

mod lexer;
mod parser;
mod passes;
mod app;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn main() -> Result<(), Box<dyn Error>> {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<app::App>();

    Ok(())
}
