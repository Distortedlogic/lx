use crate::error::LxError;
use crate::value::LxVal;
use miette::SourceSpan;

use super::{HttpBackend, HttpOpts};

pub struct DenyHttpBackend;

impl HttpBackend for DenyHttpBackend {
  fn request(&self, _method: &str, _url: &str, _opts: &HttpOpts, _span: SourceSpan) -> Result<LxVal, LxError> {
    Ok(LxVal::Err(Box::new(LxVal::str("network access denied by sandbox policy"))))
  }
}
