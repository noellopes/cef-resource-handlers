use crate::ResourceHandlerError;
use cef::{ImplPostData, ImplPostDataElement, ImplRequest};
use cef_dll_sys::cef_postdataelement_type_t;
use std::collections::HashMap;

/// A collection of key-value request values.
pub struct ParsedRequestValues(Option<HashMap<String, String>>);

impl ParsedRequestValues {
    fn from_values(values: HashMap<String, String>) -> Self {
        Self(Some(values))
    }

    fn absent() -> Self {
        Self(None)
    }

    /// Returns `true` if the collection (e.g. (post data, query parameters)) is present in the request.
    pub fn is_present(&self) -> bool {
        self.0.is_some()
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.0
            .as_ref()
            .and_then(|values| values.get(key))
            .map(String::as_str)
    }
}

pub type QueryParameters = ParsedRequestValues;
pub type PostDataValues = ParsedRequestValues;

/// Parsed request information.
pub struct RequestInfo {
    pub protocol: String,
    pub path: String,
    pub query: QueryParameters,
    pub post_data: PostDataValues,
}

impl RequestInfo {
    pub(crate) fn from_request(request: &cef::Request) -> Result<Self, ResourceHandlerError> {
        let url = cef::CefString::from(&request.url()).to_string();

        let (protocol, full_path) = match url.trim_end_matches('/').split_once("://") {
            Some((protocol, path)) if !protocol.is_empty() && !path.is_empty() => {
                (protocol.to_owned(), path.to_owned())
            }
            _ => return Err(ResourceHandlerError::UrlParseError(url)),
        };

        let (path, query) = match full_path.split_once('?') {
            Some((path, query)) => {
                match serde_urlencoded::from_str::<HashMap<String, String>>(query) {
                    Ok(query) => (path.to_owned(), QueryParameters::from_values(query)),
                    _ => return Err(ResourceHandlerError::UrlParseError(url)),
                }
            }
            None => (full_path, QueryParameters::absent()),
        };

        let post_data = Self::fetch_post_data(request)
            .map_err(|e| ResourceHandlerError::PostDataError(url, e.to_string()))?;

        Ok(Self {
            protocol,
            path,
            query,
            post_data,
        })
    }

    fn fetch_post_data(
        request: &cef::Request,
    ) -> Result<PostDataValues, serde_urlencoded::de::Error> {
        let Some(post_data) = request.post_data() else {
            return Ok(PostDataValues::absent());
        };

        let mut post_data_values = HashMap::<String, String>::new();

        let element_count = post_data.element_count();
        if element_count == 0 {
            return Ok(PostDataValues::from_values(post_data_values));
        }

        let mut vector_elements: Vec<Option<cef::PostDataElement>> = vec![None; element_count];
        post_data.elements(Some(&mut vector_elements));

        for element in vector_elements.into_iter().flatten() {
            match element.get_type().as_ref() {
                cef_postdataelement_type_t::PDE_TYPE_FILE => {
                    let raw_file_path = cef::CefString::from(&element.file()).to_string();
                    let file_path = serde_urlencoded::from_str::<String>(&raw_file_path)?;
                    post_data_values.insert("file".into(), file_path);
                }
                cef_postdataelement_type_t::PDE_TYPE_BYTES => {
                    let byte_count = element.bytes_count();
                    if byte_count == 0 {
                        continue;
                    }

                    let mut buffer = vec![0u8; byte_count];
                    let bytes_read = element.bytes(byte_count, buffer.as_mut_ptr());
                    if bytes_read != byte_count {
                        if bytes_read == 0 {
                            continue;
                        }

                        buffer.truncate(bytes_read);
                    }

                    let elements =
                        serde_urlencoded::from_bytes::<HashMap<String, String>>(&buffer)?;
                    post_data_values.extend(elements);
                }
                _ => {}
            }
        }

        Ok(PostDataValues::from_values(post_data_values))
    }
}
