use super::hello_schemes::*;
use cef_resource_handlers::*;
use maud::{Markup, Render, html};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum WebPage {
    Home,
    Hello,
    NotFound(String),
}

impl WebPage {
    fn from_path(path: &str) -> Self {
        match path {
            "home" => Self::Home,
            "hello" => Self::Hello,
            _ => Self::NotFound(path.to_owned()),
        }
    }

    pub(crate) fn url(&self) -> String {
        let path = match self {
            Self::Home => "home",
            Self::Hello => "hello",
            Self::NotFound(path) => path,
        };

        format!("{APP_SCHEME}://{path}")
    }
}

pub(crate) enum HelloWebPageHandler {
    Home,
    Hello { name: String },
    Error { description: String },
}

impl WebPageHandler for HelloWebPageHandler {
    fn from_request(request_info: &RequestInfo) -> Result<Self, ResourceHandlerError> {
        let page = WebPage::from_path(&request_info.path);

        let handler = match page {
            WebPage::Home => Self::Home,
            WebPage::Hello => match request_info.post_data.get("name") {
                Some(name) => Self::Hello {
                    name: name.to_owned(),
                },
                None => Self::Error {
                    description: "Invalid post data received.".into(),
                },
            },
            WebPage::NotFound(page) => Self::Error {
                description: format!("Page not found: {page}."),
            },
        };

        Ok(handler)
    }

    fn render(&self) -> String {
        let (title, contents) = match self {
            Self::Home => ("Home", home_page()),
            Self::Hello { name } => ("Hello", hello_page(name)),
            Self::Error { description } => ("Error", error_page(description)),
        };

        master_page(title, &contents)
    }
}

fn home_page() -> Markup {
    html! {
        h1 { "Home" }

        form action=(WebPage::Hello.url()) method="post" {
            div class="mb-3" {
                label for="name" class="form-label" { "Name" }
                input type="text" class="form-control" id="name" name="name" placeholder="Enter your name";
            }
            button type="submit" class="btn btn-primary" { "Submit" }
        }
    }
}

fn hello_page(name: &str) -> Markup {
    let name = name.trim();
    let name = if name.is_empty() { "Anonymous" } else { name };

    let title = format!("Hello {name}!");

    html! {
        h1 { (title) }
        (back_button())
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
                div class="container mt-2" {
                    (contents)
                }

                script src=(bootstrap_js) {}
            }
        }
    };

    page.into_string()
}
