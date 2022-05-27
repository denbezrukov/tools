use rome_formatter::printer::PrinterOptions;
use rome_formatter::{FormatOptions, IndentStyle, LineWidth};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Default)]
pub struct JsFormatOptions {
    /// The indent style.
    pub indent_style: IndentStyle,

    /// What's the max width of a line. Defaults to 80.
    pub line_width: LineWidth,

    // The style for quotes. Defaults to double.
    pub quote_style: QuoteStyle,
}

impl JsFormatOptions {
    pub fn tab_width(&self) -> u8 {
        match self.indent_style {
            IndentStyle::Tab => 2,
            IndentStyle::Space(quantities) => quantities,
        }
    }
}

impl FormatOptions for JsFormatOptions {
    fn indent_style(&self) -> IndentStyle {
        self.indent_style
    }

    fn line_with(&self) -> LineWidth {
        self.line_width
    }

    fn as_print_options(&self) -> PrinterOptions {
        PrinterOptions::default()
            .with_indent(self.indent_style)
            .with_print_width(self.line_width)
    }
}

impl fmt::Display for JsFormatOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Indent style: {}", self.indent_style)?;
        writeln!(f, "Line width: {}", self.line_width.value())?;
        writeln!(f, "Quote style: {}", self.quote_style)?;
        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum QuoteStyle {
    Double,
    Single,
}

impl Default for QuoteStyle {
    fn default() -> Self {
        Self::Double
    }
}

impl FromStr for QuoteStyle {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "double" | "Double" => Ok(Self::Double),
            "single" | "Single" => Ok(Self::Single),
            // TODO: replace this error with a diagnostic
            _ => Err("Value not supported for QuoteStyle"),
        }
    }
}

impl fmt::Display for QuoteStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuoteStyle::Double => write!(f, "Double Quotes"),
            QuoteStyle::Single => write!(f, "Single Quotes"),
        }
    }
}

impl QuoteStyle {
    pub fn as_char(&self) -> char {
        match self {
            QuoteStyle::Double => '"',
            QuoteStyle::Single => '\'',
        }
    }

    pub fn as_string(&self) -> &str {
        match self {
            QuoteStyle::Double => "\"",
            QuoteStyle::Single => "'",
        }
    }

    /// Returns the quote, prepended with a backslash (escaped)
    pub fn as_escaped(&self) -> &str {
        match self {
            QuoteStyle::Double => "\\\"",
            QuoteStyle::Single => "\\'",
        }
    }

    pub fn as_bytes(&self) -> u8 {
        self.as_char() as u8
    }

    /// Returns the quote in HTML entity
    pub fn as_html_entity(&self) -> &str {
        match self {
            QuoteStyle::Double => "&quot;",
            QuoteStyle::Single => "&apos;",
        }
    }

    /// Given the current quote, it returns the other one
    pub fn other(&self) -> Self {
        match self {
            QuoteStyle::Double => QuoteStyle::Single,
            QuoteStyle::Single => QuoteStyle::Double,
        }
    }
}
