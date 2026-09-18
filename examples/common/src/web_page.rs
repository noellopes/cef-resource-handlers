use crate::schemes::LOCAL_FILE_SCHEME;
use maud::{Markup, Render, html};

pub fn back_button() -> Markup {
    html! {
        button type="button" class="btn btn-primary" onclick="window.history.back();" { "Back" }
    }
}

pub fn error_page(description: &str) -> Markup {
    html! {
        h1 { "An error occurred" }
        p { (description) }
        (back_button())
    }
}

pub fn master_page(title: &str, contents: impl Render) -> String {
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
