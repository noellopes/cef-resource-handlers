use cef_resource_handlers::*;
use common::schemes::APP_SCHEME;
use common::web_page::{back_button, error_page, master_page};
use maud::{Markup, html};

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
    type Context = ();

    fn from_request(
        request_info: &RequestInfo,
        _context: &Self::Context,
    ) -> Result<Self, ResourceHandlerError> {
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
