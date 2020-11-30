use crate::lexer::Token;
use crate::parser::{ExprVisitor, Parser};
use crate::passes::derivative::derivative;
use crate::transformations::{prettify::Prettify, simplify::Simplify};
use logos::Logos;
use yew::prelude::*;

enum ItemKind {
    Input,
    ParsedAs,
    Derivative,
    DebugMsg,
    Error,
}

struct Item {
    kind: ItemKind,
    text: String,
}

pub enum Msg {
    UpdateInput(String),
    AddItem,
    ToggleDbg,
    Noop,
}

pub struct App {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    input: String,
    items: Vec<Item>,
    debug_mode: bool,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            input: "".to_string(),
            items: Vec::new(),
            debug_mode: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::UpdateInput(input) => {
                self.input = input;
                false
            }
            Msg::AddItem => {
                let mut start: f64 = web_sys::window().unwrap().performance().unwrap().now();
                let initial_start = start;

                self.items.push(Item {
                    kind: ItemKind::Input,
                    text: self.input.clone(),
                });

                // compute folded expression and derivative
                let mut tokens = Token::lexer(&self.input);
                let mut tokens2 = tokens.clone();
                if tokens2.next().is_none() {
                    self.items.push(Item {
                        kind: ItemKind::Error,
                        text: "no input found, skipping".to_string(),
                    });
                    return true;
                }

                let mut parser = Parser::from(&mut tokens);
                let mut ast = parser.parse();

                if self.debug_mode {
                    let now = web_sys::window().unwrap().performance().unwrap().now();
                    self.items.push(Item {
                        kind: ItemKind::DebugMsg,
                        text: format!("Parsed input - took {}ms", now - start),
                    });
                    start = now;
                }

                if !parser.errors().is_empty() {
                    self.items.extend(parser.errors().iter().map(|error| Item {
                        kind: ItemKind::Error,
                        text: error.clone(),
                    }));
                }

                Simplify.visit(&mut ast);
                if self.debug_mode {
                    let now = web_sys::window().unwrap().performance().unwrap().now();
                    self.items.push(Item {
                        kind: ItemKind::DebugMsg,
                        text: format!("Simplify input - took {}ms", now - start),
                    });
                    start = now;
                }

                // do not prettify expr used for derivative
                let mut ast2 = ast.clone();
                Prettify.visit(&mut ast2);
                Simplify.visit(&mut ast2);

                if self.debug_mode {
                    let now = web_sys::window().unwrap().performance().unwrap().now();
                    self.items.push(Item {
                        kind: ItemKind::DebugMsg,
                        text: format!("Prettify input - took {}ms", now - start),
                    });
                    start = now;
                }

                self.items.push(Item {
                    kind: ItemKind::ParsedAs,
                    text: format!("{}", ast2),
                });

                match derivative(&ast, "x") {
                    Ok(mut derivative) => {
                        if self.debug_mode {
                            let now = web_sys::window().unwrap().performance().unwrap().now();
                            self.items.push(Item {
                                kind: ItemKind::DebugMsg,
                                text: format!("Compute derivative - took {}ms", now - start),
                            });
                            start = now;
                        }

                        Simplify.visit(&mut derivative);
                        Prettify.visit(&mut derivative);
                        Simplify.visit(&mut derivative);

                        if self.debug_mode {
                            let now = web_sys::window().unwrap().performance().unwrap().now();
                            self.items.push(Item {
                                kind: ItemKind::DebugMsg,
                                text: format!(
                                    "Simplify and prettify derivative - took {}ms",
                                    now - start
                                ),
                            });
                        }

                        self.items.push(Item {
                            kind: ItemKind::Derivative,
                            text: format!("{}", derivative),
                        });
                    }
                    Err(err) => self.items.push(Item {
                        kind: ItemKind::Error,
                        text: err,
                    }),
                }

                if self.debug_mode {
                    let now = web_sys::window().unwrap().performance().unwrap().now();
                    self.items.push(Item {
                        kind: ItemKind::DebugMsg,
                        text: format!("Total time elapsed - {}ms", now - initial_start),
                    });
                }

                self.input = "".to_string();

                // wrap scroll to bottom in Callback to call after list is rendered
                // FIXME: call in next update
                let scroll_to_bottom = self.link.callback_once(|()| {
                    web_sys::window().unwrap().scroll_to_with_x_and_y(
                        0.0,
                        web_sys::window()
                            .unwrap()
                            .document()
                            .unwrap()
                            .body()
                            .unwrap()
                            .scroll_height() as f64,
                    );
                    Msg::Noop
                });
                scroll_to_bottom.emit(());

                true
            }
            Msg::ToggleDbg => {
                self.debug_mode = !self.debug_mode;
                true
            }
            Msg::Noop => false,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                { self.view_header() }
                <div class="output-area">
                    {
                        for self.items.iter().map(|item| match item.kind {
                            ItemKind::Input => html! {
                                <p class="input">
                                    <i class="sub">{ "> " }</i>
                                    { &item.text }
                                    </p>
                            },
                            ItemKind::ParsedAs => html! {
                                <p class="parsed-as">
                                    <i class="sub">{ "f(x) = " }</i>
                                    { &item.text }
                                </p>
                            },
                            ItemKind::Derivative => html! {
                                <p class="derivative">
                                    <i class="sub">{ "df/dx = " }</i>
                                    { &item.text }
                                </p>
                            },
                            ItemKind::DebugMsg => html! {
                                <p class="debug-msg">
                                    <i class="sub">{ "[DEBUG]: " }</i>
                                    { &item.text }
                                </p>
                            },
                            ItemKind::Error => html! {
                                <p class="error">
                                    {" Error: " }
                                    <i class="error-msg">{ &item.text }</i>
                                </p>
                            }
                        })
                    }
                </div>

                <input
                    type="text"
                    placeholder="Enter expression here, e.g. 2 * x ^ 2"
                    value=self.input
                    onkeyup=self.link.callback(|ev: KeyboardEvent| {
                        if ev.code() == "Enter" {
                            Msg::AddItem
                        } else { Msg::Noop }
                    })
                    oninput=self.link.callback(|ev: InputData| Msg::UpdateInput(ev.value))
                />
            </div>
        }
    }
}

impl App {
    fn view_header(&self) -> Html {
        html! {
            <header>
                { "Derivative calculator - Source: " }
                <a href="https://github.com/lukechu10/derivative-calculator">{ "lukechu10/derivative-calculator" }</a>

                <i class="debug-mode-toggle" onclick=self.link.callback(|_| Msg::ToggleDbg)>
                    { format!("Debug mode {}", if self.debug_mode { "on" } else { "off" }) }
                </i>
            </header>
        }
    }
}
