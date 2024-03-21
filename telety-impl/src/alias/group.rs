use syn::{Ident, Path};

#[derive(Debug)]
pub(crate) struct Group {
    containing_path: Path,
    mod_ident: Ident,
}

impl Group {
    pub(crate) fn new(containing_path: Path, mod_ident: Ident) -> Self {
        Self {
            containing_path,
            mod_ident,
        }
    }

    #[allow(dead_code)]
    /// The [struct@Ident] of the module.
    pub fn ident(&self) -> Ident {
        self.mod_ident.clone()
    }

    /// The qualified [Path] to the module.
    pub fn path(&self) -> Path {
        let mut path = self.containing_path.clone();
        path.segments.push(self.mod_ident.clone().into());
        path
    }
}
