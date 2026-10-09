//! Tests for HTML email rendering.

use super::*;

#[test]
fn escapes_html_metacharacters() {
    assert_eq!(escape("<b>&\"</b>"), "&lt;b&gt;&amp;&quot;&lt;/b&gt;");
}

#[test]
fn escape_passes_plain_text_through() {
    assert_eq!(escape("обычный текст"), "обычный текст");
}
