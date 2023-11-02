use wasm_bindgen::JsError;

pub type JsResult<T> = std::result::Result<T, JsError>;