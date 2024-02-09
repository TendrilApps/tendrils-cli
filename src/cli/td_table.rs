use tabled::builder::Builder;

// See https://github.com/nushell/nushell/blob/main/crates/nu-table/src/table.rs
// for a 'tabled' usage example

pub struct TdTable {
    builder: Builder,
}

impl TdTable {
    pub fn new() -> TdTable {
        TdTable {
            builder: Builder::default(),
        }
    }

    pub fn push_row(&mut self, data: &[String]) {
        self.builder.push_record(data);
    }

    pub fn build(self) -> String {
        self.builder.build().to_string()
    }
}
