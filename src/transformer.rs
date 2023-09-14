use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use globwalk::glob;
use path_clean::PathClean;
use swc_core::common::DUMMY_SP;
use swc_core::ecma::ast::{
    BindingIdent, Expr, ExprOrSpread, Ident, ImportDecl, ImportDefaultSpecifier,
    ImportNamedSpecifier, ImportSpecifier as SWCImportSpecifier, ImportStarAsSpecifier, Pat, Str,
    VarDecl,
};

use crate::imports::ImportSpecifier;
use crate::utils::{get_import_map_expr, to_var_decls, upsert_map};
use crate::ImportGlobArrayPlugin;

pub(crate) struct TransformedStatements {
    pub(crate) imports: Vec<ImportDecl>,
    pub(crate) meta: Vec<VarDecl>,
    pub(crate) names: Vec<VarDecl>,
}

pub(crate) fn build_glob_path(plugin: &ImportGlobArrayPlugin, src: &str) -> Option<PathBuf> {
    let glob_path = plugin.filename.parent()?.join(src).clean();

    return if glob_path.starts_with(&plugin.cwd) {
        let glob_path = glob_path.strip_prefix(&plugin.cwd).ok()?;
        Some(
            PathBuf::from_str("/cwd")
                .unwrap_or_default()
                .join(glob_path),
        )
    } else {
        println!(
            "{:?} doesn't start with {:?}... Is it outside the current working directory?",
            glob_path, &plugin.cwd
        );
        None
    };
}

/// Expand the glob pattern embedded within an [ImportDecl](ImportDecl), and give back a tuple of three (3) values:
///
/// * The first, a vector of [ImportDecl](ImportDecl), with each item as the expanded representation of the original
///   glob pattern.
///
/// * The second, a vector of [VarDecl](VarDecl), with each item as an [ArrayLit](swc_core::ecma::ast::ArrayLit) that
///   contains each expanded import that was previously assigned to the variable.
///
/// * The third, a vector of [VarDecl](VarDecl), with each item as an [ArrayLit](swc_core::ecma::ast::ArrayLit) that
///   contains an embedded object for the special `_importMeta` token. This vector may be empty.
pub(crate) fn transform_import_decl(
    plugin: &ImportGlobArrayPlugin,
    import_src: Box<Str>,
    import_specifiers: Vec<SWCImportSpecifier>,
) -> Option<TransformedStatements> {
    let glob_path = build_glob_path(plugin, import_src.value.to_string().as_str())?;
    let glob_path = glob_path.to_str()?;

    let mut name_placeholder_map: HashMap<Pat, Vec<Option<ExprOrSpread>>> = HashMap::new();
    let mut import_meta_map: HashMap<Pat, Vec<Option<ExprOrSpread>>> = HashMap::new();

    let glob_walker = glob(glob_path).ok()?;
    let import_statements: Vec<ImportDecl> = glob_walker
        .map(|result| match result {
            Ok(file_path) => {
                let import_paths = plugin.get_paths(&file_path.into_path())?;
                let specifiers: Vec<SWCImportSpecifier> =
                    import_specifiers.iter().fold(vec![], |mut acc, specifier| {
                        let specifier: ImportSpecifier = specifier.to_owned().into();
                        let name_ident = Pat::Ident(BindingIdent {
                            id: Ident::new(specifier.get_local_name().into(), DUMMY_SP),
                            type_ann: None,
                        });

                        if specifier.is_meta_decl().unwrap_or(false) {
                            upsert_map(
                                &mut import_meta_map,
                                &name_ident,
                                get_import_map_expr(&import_paths),
                            );
                            return acc;
                        }

                        let placeholder = &*plugin.next_id("_iga");

                        upsert_map(
                            &mut name_placeholder_map,
                            &name_ident,
                            ExprOrSpread::from(Box::new(Expr::Ident(Ident::new(
                                placeholder.into(),
                                DUMMY_SP,
                            )))),
                        );

                        acc.push(match specifier.into_inner() {
                            SWCImportSpecifier::Default(_) => {
                                SWCImportSpecifier::Default(ImportDefaultSpecifier {
                                    local: Ident::new(placeholder.into(), DUMMY_SP),
                                    span: DUMMY_SP,
                                })
                            }
                            SWCImportSpecifier::Named(named) => {
                                SWCImportSpecifier::Named(ImportNamedSpecifier {
                                    imported: named.imported.clone(),
                                    is_type_only: false,
                                    local: Ident::new(placeholder.into(), DUMMY_SP),
                                    span: DUMMY_SP,
                                })
                            }
                            SWCImportSpecifier::Namespace(_) => {
                                SWCImportSpecifier::Namespace(ImportStarAsSpecifier {
                                    local: Ident::new(placeholder.into(), DUMMY_SP),
                                    span: DUMMY_SP,
                                })
                            }
                        });
                        acc
                    });

                Some(ImportDecl {
                    span: DUMMY_SP,
                    specifiers,
                    src: Box::new(Str {
                        raw: None,
                        span: DUMMY_SP,
                        value: import_paths.imported_path.into(),
                    }),
                    type_only: false,
                    with: None
                })
            }
            Err(_) => None,
        })
        .filter(|i| i.is_some())
        .map(|i| i.unwrap())
        .collect();

    Some(TransformedStatements {
        imports: import_statements,
        meta: to_var_decls(import_meta_map),
        names: to_var_decls(name_placeholder_map),
    })
}
