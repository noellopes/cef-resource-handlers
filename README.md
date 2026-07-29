# cef-resource-handlers

Resource handlers for [cef-rs](https://github.com/tauri-apps/cef-rs) applications.

This crate provides CEF resource handlers for serving:

- Local files (with MIME detection)
- Dynamically rendered HTML pages
- Fully custom byte streams via your own content provider

## Features

Core features:

- Generic `ContentProvider` abstraction for custom data sources
- `LocalFileResourceHandlerFactory` for static assets
- `WebPageResourceHandlerFactory<T>` for dynamic HTML

Supporting utilities:

- Request parsing via `RequestInfo`:
  - URL protocol and path
  - Query parameters
  - POST form values
- Unified error type: `ResourceHandlerError`

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
cef-resource-handlers = "0.1"
```

## Quick Start

### 1. Register custom schemes with CEF

In your `App::on_register_custom_schemes` callback, register your scheme as CEF custom scheme.

```rust
use cef::*;
use cef_dll_sys::cef_scheme_options_t;
use std::os::raw::c_int;

const LOCAL_FILE_SCHEME: &str = "local";

fn on_register_custom_schemes(registrar: Option<&mut SchemeRegistrar>) {
    let Some(registrar) = registrar else { return; };

    let options = cef_scheme_options_t::CEF_SCHEME_OPTION_STANDARD as c_int
        | cef_scheme_options_t::CEF_SCHEME_OPTION_SECURE as c_int
        | cef_scheme_options_t::CEF_SCHEME_OPTION_CORS_ENABLED as c_int;

    let scheme = CefString::from(LOCAL_FILE_SCHEME);
    let _ = registrar.add_custom_scheme(Some(&scheme), options);
}
```

### 2. Register the resource handler factory

After CEF is initialized (typically in `on_context_initialized`), register the crate factory:

```rust
use cef_resource_handlers::LocalFileResourceHandlerFactory;

const LOCAL_FILE_SCHEME: &str = "local";

fn register_handlers() {
    LocalFileResourceHandlerFactory::register(LOCAL_FILE_SCHEME, None)
        .expect("failed to register local scheme handler");
}
```

Then URLs like `local://bootstrap.min.css` can be served from files relative to the executable directory.

## Dynamic HTML Pages

Implement `WebPageHandler`, then register `WebPageResourceHandlerFactory<YourHandler>`:

```rust
use cef_resource_handlers::{
    RequestInfo, ResourceHandlerError, WebPageHandler, WebPageResourceHandlerFactory,
};

const APP_SCHEME: &str = "app";

struct MyPage;

impl WebPageHandler for MyPage {
    fn from_request(_request_info: &RequestInfo) -> Result<Self, ResourceHandlerError> {
        Ok(Self)
    }

    fn render(&self) -> String {
        "<h1>Hello from app://</h1>".to_owned()
    }
}

fn register_web_pages() {
    WebPageResourceHandlerFactory::<MyPage>::register(APP_SCHEME, None)
        .expect("failed to register app scheme handler");
}
```

## Custom Content Providers

Use `CustomResourceHandlerFactory<T>` when you need full control over content generation and streaming.

```rust
use cef_resource_handlers::{
    ContentProvider, CustomResourceHandlerFactory, RequestInfo, ResourceHandlerError,
};

struct PlainTextProvider {
    body: Vec<u8>,
}

impl ContentProvider for PlainTextProvider {
    fn from_request(_request_info: &RequestInfo) -> Result<Self, ResourceHandlerError> {
        Ok(Self {
            body: b"Hello from custom provider".to_vec(),
        })
    }

    fn size(&self) -> Option<usize> {
        Some(self.body.len())
    }

    fn mime_type(&self) -> &str {
        "text/plain; charset=utf-8"
    }

    fn should_cache(&self) -> bool {
        false
    }

    fn read(
        &mut self,
        data_out: *mut u8,
        offset: usize,
        bytes_to_read: usize,
    ) -> Result<usize, ResourceHandlerError> {
        let remaining = self.body.len().saturating_sub(offset);
        let count = remaining.min(bytes_to_read);

        if count == 0 {
            return Ok(0);
        }

        // SAFETY: `data_out` is provided by CEF with at least `count` writable bytes.
        unsafe {
            std::ptr::copy_nonoverlapping(self.body.as_ptr().add(offset), data_out, count);
        }

        Ok(count)
    }
}

type PlainTextFactory = CustomResourceHandlerFactory<PlainTextProvider>;

fn register_plain_text_scheme() {
    PlainTextFactory::register("text", None).expect("failed to register text scheme handler");
}
```

## Request Data

`RequestInfo` exposes parsed request fields:

- `protocol`: scheme name (for example, `app`)
- `path`: URL path without query string
- `query`: URL query key/values
- `post_data`: URL-encoded POST key/values

You can access values with `.get("key")`.

## Example Application

This repository includes an example in `examples/hello`.

Run it:

```bash
cargo run -p hello --bin hello_app
```

## License

Licensed under either of:

- MIT License
- Apache License, Version 2.0

at your option.
