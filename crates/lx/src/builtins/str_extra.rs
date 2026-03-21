use std::sync::Arc;

use num_traits::ToPrimitive;

use crate::backends::RuntimeCtx;
use crate::error::LxError;
use crate::span::Span;
use crate::value::LxVal;

pub(super) fn bi_replace(
    args: &[LxVal],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<LxVal, LxError> {
    let old = args[0].as_str().ok_or_else(|| {
        LxError::type_err(
            format!(
                "replace: first arg must be Str, got {}",
                args[0].type_name()
            ),
            span,
        )
    })?;
    let new = args[1].as_str().ok_or_else(|| {
        LxError::type_err(
            format!(
                "replace: second arg must be Str, got {}",
                args[1].type_name()
            ),
            span,
        )
    })?;
    let s = args[2].as_str().ok_or_else(|| {
        LxError::type_err(
            format!(
                "replace: third arg must be Str, got {}",
                args[2].type_name()
            ),
            span,
        )
    })?;
    Ok(LxVal::Str(Arc::from(s.replacen(old, new, 1).as_str())))
}

pub(super) fn bi_replace_all(
    args: &[LxVal],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<LxVal, LxError> {
    let old = args[0].as_str().ok_or_else(|| {
        LxError::type_err(
            format!(
                "replace_all: first arg must be Str, got {}",
                args[0].type_name()
            ),
            span,
        )
    })?;
    let new = args[1].as_str().ok_or_else(|| {
        LxError::type_err(
            format!(
                "replace_all: second arg must be Str, got {}",
                args[1].type_name()
            ),
            span,
        )
    })?;
    let s = args[2].as_str().ok_or_else(|| {
        LxError::type_err(
            format!(
                "replace_all: third arg must be Str, got {}",
                args[2].type_name()
            ),
            span,
        )
    })?;
    Ok(LxVal::Str(Arc::from(s.replace(old, new).as_str())))
}

pub(super) fn bi_repeat(
    args: &[LxVal],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<LxVal, LxError> {
    let n = args[0].as_int().ok_or_else(|| {
        LxError::type_err(
            format!("repeat: first arg must be Int, got {}", args[0].type_name()),
            span,
        )
    })?;
    let s = args[1].as_str().ok_or_else(|| {
        LxError::type_err(
            format!(
                "repeat: second arg must be Str, got {}",
                args[1].type_name()
            ),
            span,
        )
    })?;
    let count = n
        .to_usize()
        .ok_or_else(|| LxError::runtime("repeat: count out of range", span))?;
    Ok(LxVal::Str(Arc::from(s.repeat(count).as_str())))
}

pub(super) fn bi_starts(
    args: &[LxVal],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<LxVal, LxError> {
    let prefix = args[0].as_str().ok_or_else(|| {
        LxError::type_err(
            format!(
                "starts?: first arg must be Str, got {}",
                args[0].type_name()
            ),
            span,
        )
    })?;
    let s = args[1].as_str().ok_or_else(|| {
        LxError::type_err(
            format!(
                "starts?: second arg must be Str, got {}",
                args[1].type_name()
            ),
            span,
        )
    })?;
    Ok(LxVal::Bool(s.starts_with(prefix)))
}

pub(super) fn bi_ends(
    args: &[LxVal],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<LxVal, LxError> {
    let suffix = args[0].as_str().ok_or_else(|| {
        LxError::type_err(
            format!("ends?: first arg must be Str, got {}", args[0].type_name()),
            span,
        )
    })?;
    let s = args[1].as_str().ok_or_else(|| {
        LxError::type_err(
            format!("ends?: second arg must be Str, got {}", args[1].type_name()),
            span,
        )
    })?;
    Ok(LxVal::Bool(s.ends_with(suffix)))
}

fn pad(args: &[LxVal], span: Span, name: &str, left: bool) -> Result<LxVal, LxError> {
    let width = args[0]
        .as_int()
        .ok_or_else(|| {
            LxError::type_err(
                format!("{name}: first arg must be Int, got {}", args[0].type_name()),
                span,
            )
        })?
        .to_usize()
        .ok_or_else(|| LxError::runtime(format!("{name}: width out of range"), span))?;
    let s = args[1].as_str().ok_or_else(|| {
        LxError::type_err(
            format!(
                "{name}: second arg must be Str, got {}",
                args[1].type_name()
            ),
            span,
        )
    })?;
    let char_count = s.chars().count();
    if char_count >= width {
        Ok(LxVal::Str(Arc::from(s)))
    } else {
        let padding = " ".repeat(width - char_count);
        let result = if left {
            format!("{padding}{s}")
        } else {
            format!("{s}{padding}")
        };
        Ok(LxVal::Str(Arc::from(result.as_str())))
    }
}

pub(super) fn bi_pad_left(
    args: &[LxVal],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<LxVal, LxError> {
    pad(args, span, "pad_left", true)
}

pub(super) fn bi_pad_right(
    args: &[LxVal],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<LxVal, LxError> {
    pad(args, span, "pad_right", false)
}
