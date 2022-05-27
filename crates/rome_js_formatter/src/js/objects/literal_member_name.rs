use crate::prelude::*;
use crate::utils::FormatLiteralStringToken;
use crate::FormatNodeFields;
use rome_js_syntax::JsLiteralMemberNameFields;
use rome_js_syntax::{JsLiteralMemberName, JsSyntaxKind};

impl FormatNodeFields<JsLiteralMemberName> for FormatNodeRule<JsLiteralMemberName> {
    fn format_fields(
        node: &JsLiteralMemberName,
        formatter: &Formatter<JsFormatOptions>,
    ) -> FormatResult<FormatElement> {
        let JsLiteralMemberNameFields { value } = node.as_fields();

        let value = value?;

        match value.kind() {
            JsSyntaxKind::JS_STRING_LITERAL => {
                FormatLiteralStringToken::from_string(&value).format(formatter)
            }
            _ => formatted![formatter, [value.format()]],
        }
    }
}
