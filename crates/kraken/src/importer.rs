/// importer.rs
use dungeon_tax::sheet;
use std::io;

pub struct FileImporter<R> {
    _io: R,
}

impl<R: io::Read> sheet::Importer for FileImporter<R> {
    fn import(&self, _entries: &mut Vec<sheet::Entry>, _config: sheet::Headers) {
        unimplemented!()
    }

    fn size_hint(&self) -> usize {
        unimplemented!()
    }
}

pub struct HttpImporter<R> {
    _io: R,
}

impl<R: io::Read> sheet::Importer for HttpImporter<R> {
    fn import(&self, _entries: &mut Vec<sheet::Entry>, _config: sheet::Headers) {
        unimplemented!()
    }

    fn size_hint(&self) -> usize {
        unimplemented!()
    }
}
