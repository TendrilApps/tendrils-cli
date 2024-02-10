use inline_colorization::{
    color_green,
    color_reset,
    style_underline,
    style_reset,
};
use tabled::grid::ansi::ANSIBuf;
use tabled::grid::config::{ColoredConfig, Entity, SpannedConfig};
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

    fn header_styling() -> ColoredConfig {
        // Must use ANSIBuf instead of simply passing an ANSI string
        // otherwise the width of the cell does not display properly
        let green_ul = ANSIBuf::new(
            color_green.to_string() + style_underline,
            color_reset.to_string() + style_reset
        );

        let spanned_cfg = SpannedConfig::default();
        let mut clr_cfg = ColoredConfig::new(spanned_cfg);
        let cells = Entity::Row(0);
        clr_cfg.set_color(cells, green_ul);

        return clr_cfg;
    }

    /// Adds a row of data to the bottom of the table.
    /// Note: The first row is treated as a header, so the
    /// column names should be pushed before anything else
    pub fn push_row(&mut self, data: &[String]) {
        self.builder.push_record(data);
    }

    pub fn draw(self) -> String {
        let style = Style::modern_rounded();
        let tbl = self.builder.build()
            .with(TdTable::header_styling())
            .with(style)
            .to_owned();

        tbl.to_string()
    }
}
