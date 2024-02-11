use inline_colorization::{
    color_bright_green,
    color_reset,
    style_underline,
    style_reset,
};
use tabled::grid::ansi::ANSIBuf;
use tabled::grid::config::{ColoredConfig, Entity, EntityMap, SpannedConfig};
use tabled::settings::Style;
use tabled::builder::Builder;

pub struct TdTable {
    builder: Builder,
    style_map: EntityMap<ANSIBuf>,
}

impl TdTable {
    pub fn new() -> TdTable {
        TdTable {
            builder: Builder::default(),
            style_map: EntityMap::default(),
        }
    }

    /// Note: This must be called *before* any other
    /// data is inserted
    pub fn set_header(&mut self, col_names: &[String]) {
        let green_ul = ANSIBuf::new(
            color_bright_green.to_string() + style_underline,
            color_reset.to_string() + style_reset
        );

        self.style_map.insert(Entity::Row(0), green_ul);
        self.push_row(col_names);
    }

    // Must use ANSIBuf instead of simply passing a pre-formatted ANSI string
    // as data otherwise the width of the cell does not display properly
    fn targetted_styling(style_map: EntityMap<ANSIBuf>) -> ColoredConfig {
        let spanned_cfg = SpannedConfig::default();
        let mut clr_cfg = ColoredConfig::new(spanned_cfg);
        clr_cfg.set_colors(style_map);
        clr_cfg
    }

    /// Adds a row of data to the bottom of the table.
    pub fn push_row(&mut self, data: &[String]) {
        self.builder.push_record(data);
    }

    pub fn set_cell_style(
        &mut self,
        ansi_prefix: &str,
        ansi_suffix: &str,
        row: usize,
        col: usize,
    ) {
        let ansi_style = ANSIBuf::new(
            ansi_prefix,
            ansi_suffix
        );
        self.style_map.insert(Entity::Cell(row, col), ansi_style);
    }

    pub fn draw(self) -> String {
        let overall_style = Style::modern_rounded();
        let tbl = self.builder.build()
            .with(TdTable::targetted_styling(self.style_map))
            .with(overall_style)
            .to_owned();

        tbl.to_string()
    }
}
