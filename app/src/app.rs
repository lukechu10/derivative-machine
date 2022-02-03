use derivative_calculator::{
    lexer::Token,
    parser::{ExprVisitor, Parser},
    transformations::{derivative::derivative, prettify::Prettify, simplify::Simplify},
};
use logos::Logos;
use sycamore::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, KeyboardEvent};

#[derive(PartialEq, Eq, Clone, Copy)]
enum ItemKind {
    Input,
    ParsedAs,
    Derivative,
    DebugMsg,
    Error,
}

#[derive(PartialEq, Eq, Clone)]
struct Item {
    kind: ItemKind,
    text: String,
}

#[component]
fn Header<'a, G: Html>(ctx: ScopeRef<'a>, debug_mode: &'a Signal<bool>) -> View<G> {
    view! { ctx,
        header {
            "Derivative machine - Source: "
            a(href="https://github.com/lukechu10/derivative-machine") {
                "lukechu10/derivative-machine"
            }

            i(
                class="debug-mode-toggle",
                on:click=|_| debug_mode.set(!*debug_mode.get()),
            ) {
                "Debug mode "
                (if *debug_mode.get() { "on" } else { "off" })
            }
        }
    }
}

#[component]
fn ItemView<G: Html>(ctx: ScopeRef, item: Item) -> View<G> {
    match item.kind {
        ItemKind::Input => view! { ctx,
            p(class="input") {
                i(class="sub") { "> " } (item.text)
            }
        },
        ItemKind::ParsedAs => view! { ctx,
            p(class="parsed-as") {
                i(class="sub") { "f(x)  = " } (item.text)
            }
        },
        ItemKind::Derivative => view! { ctx,
            p(class="derivative") {
                i(class="sub") { "f'(x) = " } (item.text)
            }
        },
        ItemKind::DebugMsg => view! { ctx,
            p(class="debug-msg") {
                i(class="sub") { "[DEBUG]: " } (item.text)
            }
        },
        ItemKind::Error => view! { ctx,
            p(class="error") {
                i(class="error-msg") { "[ERROR]: " (item.text) }
            }
        },
    }
}

fn add_item(items: &Signal<Vec<Item>>, input: &str, debug_mode: bool) {
    let push_item = |item: Item| {
        let mut tmp = items.get().as_ref().clone();
        tmp.push(item);
        items.set(tmp);
    };

    let mut start: f64 = web_sys::window().unwrap().performance().unwrap().now();
    let initial_start = start;

    push_item(Item {
        kind: ItemKind::Input,
        text: input.to_string(),
    });

    // compute folded expression and derivative
    let mut tokens = Token::lexer(input);
    let mut tokens2 = tokens.clone();
    if tokens2.next().is_none() {
        push_item(Item {
            kind: ItemKind::Error,
            text: "no input found, skipping".to_string(),
        });
        return;
    }

    let mut parser = Parser::from(&mut tokens);
    let mut ast = parser.parse();

    if debug_mode {
        let now = web_sys::window().unwrap().performance().unwrap().now();
        push_item(Item {
            kind: ItemKind::DebugMsg,
            text: format!("Parsed input - took {}ms", now - start),
        });
        start = now;
    }

    if !parser.errors().is_empty() {
        for item in parser.errors().iter().map(|error| Item {
            kind: ItemKind::Error,
            text: error.clone(),
        }) {
            push_item(item);
        }
    }

    Simplify.visit(&mut ast);
    if debug_mode {
        let now = web_sys::window().unwrap().performance().unwrap().now();
        push_item(Item {
            kind: ItemKind::DebugMsg,
            text: format!("Simplify input - took {}ms", now - start),
        });
        start = now;
    }

    // do not prettify expr used for derivative
    let mut ast2 = ast.clone();
    Prettify.visit(&mut ast2);
    Simplify.visit(&mut ast2);

    if debug_mode {
        let now = web_sys::window().unwrap().performance().unwrap().now();
        push_item(Item {
            kind: ItemKind::DebugMsg,
            text: format!("Prettify input - took {}ms", now - start),
        });
        start = now;
    }

    push_item(Item {
        kind: ItemKind::ParsedAs,
        text: format!("{}", ast2),
    });

    let mut derivative = derivative(&ast);
    if debug_mode {
        let now = web_sys::window().unwrap().performance().unwrap().now();
        push_item(Item {
            kind: ItemKind::DebugMsg,
            text: format!("Compute derivative - took {}ms", now - start),
        });
        start = now;
    }

    Simplify.visit(&mut derivative);
    Prettify.visit(&mut derivative);
    Simplify.visit(&mut derivative);

    if debug_mode {
        let now = web_sys::window().unwrap().performance().unwrap().now();
        push_item(Item {
            kind: ItemKind::DebugMsg,
            text: format!("Simplify and prettify derivative - took {}ms", now - start),
        });
    }

    push_item(Item {
        kind: ItemKind::Derivative,
        text: format!("{}", derivative),
    });

    if debug_mode {
        let now = web_sys::window().unwrap().performance().unwrap().now();
        push_item(Item {
            kind: ItemKind::DebugMsg,
            text: format!("Total time elapsed - {}ms", now - initial_start),
        });
    }

    // // wrap scroll to bottom in Callback to call after list is rendered
    // // FIXME: call in next update
    // let scroll_to_bottom = self.link.callback_once(|()| {
    //     web_sys::window().unwrap().scroll_to_with_x_and_y(
    //         0.0,
    //         web_sys::window()
    //             .unwrap()
    //             .document()
    //             .unwrap()
    //             .body()
    //             .unwrap()
    //             .scroll_height() as f64,
    //     );
    //     Msg::Noop
    // });
    // scroll_to_bottom.emit(());
}

#[component]
pub fn App<G: Html>(ctx: ScopeRef) -> View<G> {
    log::info!("started");

    let items = ctx.create_signal(Vec::<Item>::new());
    let input = ctx.create_signal(String::new());
    let debug_mode = ctx.create_signal(false);

    let keyup = |ev: Event| {
        let ev = ev.unchecked_into::<KeyboardEvent>();
        if ev.code() == "Enter" {
            // Add new item
            add_item(items, &input.get(), *debug_mode.get());
            // Reset input
            input.set(String::new());
        }
    };

    view! { ctx,
        div {
            Header(debug_mode)
            div(class="output-area") {
                Indexed {
                    iterable: items,
                    view: |ctx, item| view! { ctx, ItemView(item) }
                }
            }
            input(
                type="text",
                placeholder="Enter expression here, e.g. 2 * x ^ 2",
                bind:value=input,
                on:keyup=keyup,
            )
        }
    }
}
