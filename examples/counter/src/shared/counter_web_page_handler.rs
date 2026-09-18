use cef_resource_handlers::*;
use common::schemes::{APP_SCHEME, LOCAL_FILE_SCHEME};
use maud::{Markup, Render, html};
use std::sync::{
    Arc,
    atomic::{AtomicI32, Ordering},
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum WebPage {
    Home,
    Add,
    Reset,
    NotFound(String),
}

impl WebPage {
    fn from_path(path: &str) -> Self {
        match path {
            "home" => Self::Home,
            "add" => Self::Add,
            "reset" => Self::Reset,
            _ => Self::NotFound(path.to_owned()),
        }
    }

    pub(crate) fn url(&self) -> String {
        let path = match self {
            Self::Home => "home",
            Self::Add => "add",
            Self::Reset => "reset",
            Self::NotFound(path) => path,
        };

        format!("{APP_SCHEME}://{path}")
    }
}

pub(crate) enum CounterWebPageHandler {
    Counter { value: i32 },
    Error { description: String },
}

impl WebPageHandler for CounterWebPageHandler {
    type Context = Arc<AtomicI32>;

    fn from_request(
        request_info: &RequestInfo,
        context: &Self::Context,
    ) -> Result<Self, ResourceHandlerError> {
        let page = WebPage::from_path(&request_info.path);

        let handler = match page {
            WebPage::Home => Self::Counter {
                value: context.load(Ordering::Relaxed),
            },
            WebPage::Add => match request_info
                .query
                .get("value")
                .and_then(|value| value.parse().ok())
            {
                Some(amount) => {
                    let previous_value = context.fetch_add(amount, Ordering::Relaxed);
                    let value = previous_value + amount;
                    Self::Counter { value }
                }
                None => Self::Error {
                    description: "Invalid query data received.".into(),
                },
            },
            WebPage::Reset => {
                context.store(0, Ordering::Relaxed);
                Self::Counter { value: 0 }
            }
            WebPage::NotFound(page) => Self::Error {
                description: format!("Page not found: {page}."),
            },
        };

        Ok(handler)
    }

    fn render(&self) -> String {
        let (title, contents) = match self {
            Self::Counter { value } => ("Counter", counter_page(*value)),
            Self::Error { description } => ("Error", error_page(description)),
        };

        master_page(title, &contents)
    }
}

fn counter_page(value: i32) -> Markup {
    let increment_url = format!("{}?value=1", WebPage::Add.url());
    let decrement_url = format!("{}?value=-1", WebPage::Add.url());

    html! {
        div class="d-flex flex-column align-items-center" {
            h1 class="display-1" style="font-size: 8rem;" { (value) }

            div class="d-flex gap-2" {
                a href=(increment_url) class="btn btn-primary" { "Add 1" }
                a href=(decrement_url) class="btn btn-secondary" { "Subtract 1" }
                a href=(WebPage::Reset.url()) class="btn btn-danger" { "Reset" }
            }
        }
    }
}

fn back_button() -> Markup {
    html! {
        button type="button" class="btn btn-primary" onclick="window.history.back();" { "Back" }
    }
}

fn error_page(description: &str) -> Markup {
    html! {
        h1 { "An error occurred" }
        p { (description) }
        (back_button())
    }
}

fn master_page(title: &str, contents: impl Render) -> String {
    let bootstrap_css = format!("{LOCAL_FILE_SCHEME}://bootstrap.min.css");
    let bootstrap_js = format!("{LOCAL_FILE_SCHEME}://bootstrap.min.js");

    let page = html! {
        (maud::DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width,initial-scale=1";
                title { (title) }
                link href=(bootstrap_css) rel="stylesheet";
            }
            body {
                div class="container min-vh-100 d-flex justify-content-center" {
                    (contents)
                }

                script src=(bootstrap_js) {}
            }
        }
    };

    page.into_string()
}
