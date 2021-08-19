mod app;

fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    sycamore::render(|| sycamore::template! { app::App() });
}
