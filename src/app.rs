use crate::lexer::Token;
use crate::parser::{ExprVisitor, Parser};
use crate::passes::{derivative::derivative, fold::FoldVisitor};
use logos::Logos;
use yew::prelude::*;

enum ItemKind {
    Input,
    ParsedAs,
    Derivative,
    Error,
}

struct Item {
    kind: ItemKind,
    text: String,
}

pub enum Msg {
    UpdateInput(String),
    AddItem,
    Noop,
}

pub struct App {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    input: String,
    items: Vec<Item>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            input: "".to_string(),
            items: Vec::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::UpdateInput(input) => {
                self.input = input;
                false
            }
            Msg::AddItem => {
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

                if !parser.errors().is_empty() {
                    self.items.extend(parser.errors().iter().map(|error| Item {
                        kind: ItemKind::Error,
                        text: error.clone(),
                    }));
                }

                let mut fold_visitor = FoldVisitor;
                fold_visitor.visit(&mut ast);
                self.items.push(Item {
                    kind: ItemKind::ParsedAs,
                    text: format!("{}", ast),
                });

                match derivative(&ast, "x") {
                    Ok(mut derivative) => {
                        fold_visitor.visit(&mut derivative);
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
                        ItemKind::Error => html! {
                            <p class="error">
                                {" Error: " }
                                <i class="error-msg">{ &item.text }</i>
                            </p>
                        }
                    })
                }

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
