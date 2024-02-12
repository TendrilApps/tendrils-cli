use crate::cli::ansi_style;
use inline_colorization::{
    color_bright_green,
    color_reset,
    style_underline,
    style_reset,
};
use tabled::settings::Style;
use tabled::builder::Builder;

pub struct TdTable {
    builder: Builder,
}

impl TdTable {
    pub fn new() -> TdTable {
        TdTable {
            builder: Builder::default(),
        }
    }

    /// Note: This must be called *before* any other
    /// data is inserted
    pub fn set_header(&mut self, col_names: &[String]) {
        let prefix = color_bright_green.to_string() + style_underline;
        let suffix = color_reset.to_string() + style_reset;
        let styled_names: Vec<String> = col_names.iter().map(
            |n| ansi_style(n, prefix.clone(), &suffix)
        ).collect();

        self.push_row(&styled_names);
    }

    /// Adds a row of data to the bottom of the table.
    pub fn push_row(&mut self, data: &[String]) {
        self.builder.push_record(data);
    }

    pub fn draw(self) -> String {
        let overall_style = Style::modern_rounded();
        let tbl = self.builder.build()
            .with(overall_style)
            .to_owned();

        tbl.to_string()
    }
}
