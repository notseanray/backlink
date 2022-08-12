use std::{collections::VecDeque, path::PathBuf};

struct Record {

}

pub(crate) struct RecordKeeper {
    record_cache: VecDeque<Record>,
    file_path: PathBuf,
}

impl RecordKeeper {
    pub(crate) fn write_loop() {

    }
}
