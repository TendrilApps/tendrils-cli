use crate::cli::ansi_style;
use inline_colorization::{
    color_bright_green,
    color_reset,
    style_underline,
    style_reset,
};
use tabled::settings::object::Columns;
use tabled::settings::{Modify, Style, Width};
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
        let prefix = String::from(color_bright_green) + style_underline;
        let suffix = String::from(color_reset) + style_reset;
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
            .with(Modify::new(Columns::single(2)).with(Width::wrap(50)))
            .to_owned();

        tbl.to_string()
    }
}
