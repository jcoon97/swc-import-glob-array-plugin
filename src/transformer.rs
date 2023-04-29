use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use glob::glob;
use swc_core::common::DUMMY_SP;
use swc_core::ecma::ast::{
    BindingIdent, Expr, ExprOrSpread, Ident, ImportDecl, ImportDefaultSpecifier,
    ImportNamedSpecifier, ImportSpecifier, ImportStarAsSpecifier, Pat, Str, VarDecl,
};

use crate::utils::{
    get_import_map_expr, get_local_specifier_name, is_specifier_import_meta_decl, to_var_decls,
    upsert_map,
};
use crate::ImportGlobArrayPlugin;

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
    decl: &ImportDecl,
) -> Option<(Vec<ImportDecl>, Vec<VarDecl>, Vec<VarDecl>)> {
    let glob_path = PathBuf::from_str("/cwd")
        .unwrap_or_default()
        .join(&plugin.filename)
        .with_file_name(&decl.src.value.to_string().trim_start_matches(&['.', '/']));
    let glob_path = glob_path.to_str()?;

    let mut name_placeholder_map: HashMap<Pat, Vec<Option<ExprOrSpread>>> = HashMap::new();
    let mut import_meta_map: HashMap<Pat, Vec<Option<ExprOrSpread>>> = HashMap::new();

    let glob_pattern = glob(glob_path).ok()?;
    let import_statements: Vec<ImportDecl> = glob_pattern
        .map(|result| match result {
            Ok(file_path) => {
                let import_paths = plugin.get_paths(&file_path)?;
                let specifiers: Vec<ImportSpecifier> =
                    decl.specifiers.iter().fold(vec![], |mut acc, specifier| {
                        let local_name = &*get_local_specifier_name(&specifier);

                        let name_ident = Pat::Ident(BindingIdent {
                            id: Ident::new(local_name.into(), DUMMY_SP),
                            type_ann: None,
                        });

                        if is_specifier_import_meta_decl(&specifier).unwrap_or(false) {
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

                        acc.push(match specifier {
                            ImportSpecifier::Default(_) => {
                                ImportSpecifier::Default(ImportDefaultSpecifier {
                                    local: Ident::new(placeholder.into(), DUMMY_SP),
                                    span: DUMMY_SP,
                                })
                            }
                            ImportSpecifier::Named(named) => {
                                ImportSpecifier::Named(ImportNamedSpecifier {
                                    imported: named.imported.clone(),
                                    is_type_only: false,
                                    local: Ident::new(placeholder.into(), DUMMY_SP),
                                    span: DUMMY_SP,
                                })
                            }
                            ImportSpecifier::Namespace(_) => {
                                ImportSpecifier::Namespace(ImportStarAsSpecifier {
                                    local: Ident::new(placeholder.into(), DUMMY_SP),
                                    span: DUMMY_SP,
                                })
                            }
                        });
                        acc
                    });

                Some(ImportDecl {
                    asserts: None,
                    span: DUMMY_SP,
                    specifiers,
                    src: Box::new(Str {
                        raw: None,
                        span: DUMMY_SP,
                        value: import_paths.imported_path.into(),
                    }),
                    type_only: false,
                })
            }
            Err(_) => None,
        })
        .filter(|i| i.is_some())
        .map(|i| i.unwrap())
        .collect();

    Some((
        import_statements,
        to_var_decls(name_placeholder_map),
        to_var_decls(import_meta_map),
    ))
}
