use swc_core::ecma::ast::{ImportSpecifier as SWCImportSpecifier, ModuleExportName};

pub(crate) struct ImportSpecifier(SWCImportSpecifier);

const IMPORT_META_NAME: &'static str = "_importMeta";

impl ImportSpecifier {
    pub(crate) fn get_local_name(&self) -> String {
        match &self.0 {
            SWCImportSpecifier::Default(default) => default.local.sym.to_string(),
            SWCImportSpecifier::Named(named) => named.local.sym.to_string(),
            SWCImportSpecifier::Namespace(as_star) => as_star.local.sym.to_string()
        }
    }

    pub(crate) fn is_meta_decl(&self) -> Option<bool> {
        let named_specifier = (&self.0).to_owned().named()?;
        let export_name = named_specifier.imported?;

        match export_name {
            ModuleExportName::Ident(ident) => Some(ident.sym.to_string() == IMPORT_META_NAME),
            ModuleExportName::Str(str) => Some(str.value.to_string() == IMPORT_META_NAME)
        }
    }

    pub(crate) fn into_inner(self) -> SWCImportSpecifier {
        self.0
    }
}

impl AsRef<SWCImportSpecifier> for ImportSpecifier {
    fn as_ref(&self) -> &SWCImportSpecifier {
        &self.0
    }
}

impl From<SWCImportSpecifier> for ImportSpecifier {
    fn from(value: SWCImportSpecifier) -> Self {
        ImportSpecifier(value)
    }
}
