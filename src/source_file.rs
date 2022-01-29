use std::rc::Rc;

#[doc(hidden)]
struct Inner {
    contents: String,
    name: String
}

/// SourceFile represents a source into which spans
/// point. Source files can be cheaply cloned as the
/// actual contents of them live behind an `Rc`.
#[derive(Clone)]
pub struct SourceFile(Rc<Inner>);

impl SourceFile {
    /// Create a new SourceFile
    pub fn new(contents: String, name: String) -> Self {
        Self(Rc::new(Inner {
            contents,
            name
        }))
    }
}