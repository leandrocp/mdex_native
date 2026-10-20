//! Compatibility rendering for the deprecated GFM tag filter.
//!
//! Preserve the Comrak 0.55 behavior on raw HTML nodes. This is deliberately
//! separate from sanitization and does not filter generated markup or HEEx.

use comrak::html::{ChildRendering, Context};
use std::fmt::{self, Write};

fn filtered(literal: &str) -> bool {
    let bytes = literal.as_bytes();
    if bytes.len() < 3 || bytes[0] != b'<' {
        return false;
    }
    let start = if bytes[1] == b'/' { 2 } else { 1 };
    [
        "title",
        "textarea",
        "style",
        "xmp",
        "iframe",
        "noembed",
        "noframes",
        "script",
        "plaintext",
    ]
    .iter()
    .any(|tag| {
        let end = start + tag.len();
        bytes
            .get(start..end)
            .is_some_and(|name| name.eq_ignore_ascii_case(tag.as_bytes()))
            && bytes.get(end).is_some_and(|&next| {
                matches!(next, b' ' | b'\t' | b'\n' | b'\r')
                    || next == b'>'
                    || (next == b'/' && bytes.get(end + 1) == Some(&b'>'))
            })
    })
}

pub(super) fn render_inline<T>(
    context: &mut Context<T>,
    entering: bool,
    literal: &str,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        if filtered(literal) {
            context.write_str("&lt;")?;
            context.write_str(&literal[1..])?;
        } else {
            context.write_str(literal)?;
        }
    }
    Ok(ChildRendering::HTML)
}

pub(super) fn render_block<T>(
    context: &mut Context<T>,
    entering: bool,
    literal: &str,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.cr()?;
        let mut offset = 0;
        for (index, _) in literal.match_indices('<') {
            context.write_str(&literal[offset..index])?;
            context.write_str(if filtered(&literal[index..]) {
                "&lt;"
            } else {
                "<"
            })?;
            offset = index + 1;
        }
        context.write_str(&literal[offset..])?;
        context.cr()?;
    }
    Ok(ChildRendering::HTML)
}
