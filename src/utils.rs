use std::collections::HashMap;

use swc_core::common::DUMMY_SP;
use swc_core::ecma::ast::{
    ArrayLit, Expr, ExprOrSpread, Ident, KeyValueProp, Lit,
    ObjectLit, Pat, Prop, PropName, PropOrSpread, Str, VarDecl, VarDeclarator,
};
use swc_core::ecma::ast::VarDeclKind::Const;

use crate::ImportPaths;

/// Get an [ExprOrSpread](ExprOrSpread) that contains an [ObjectLit](ObjectLit) with
/// two embedded properties: `absolutePath` and `importedPath`, both of which will get
/// pulled from `absolute_path` and `imported_path` within [ImportPaths](ImportPaths),
/// respectively.
pub(crate) fn get_import_map_expr(import_paths: &ImportPaths) -> ExprOrSpread {
    ExprOrSpread::from(Expr::Object(ObjectLit {
        props: vec![
            PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Ident(Ident::new("absolutePath".into(), DUMMY_SP)),
                value: Box::new(Expr::Lit(Lit::Str(Str {
                    raw: None,
                    span: DUMMY_SP,
                    value: import_paths.absolute_path.to_owned().into(),
                }))),
            }))),
            PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Ident(Ident::new("importedPath".into(), DUMMY_SP)),
                value: Box::new(Expr::Lit(Lit::Str(Str {
                    raw: None,
                    span: DUMMY_SP,
                    value: import_paths.imported_path.to_owned().into(),
                }))),
            }))),
        ],
        span: DUMMY_SP,
    }))
}


/// Transform a map of names and [ExprOrSpread](ExprOrSpread) elements to a vector
/// (array) of [VarDecl](VarDecl)s.
pub(crate) fn to_var_decls(map: HashMap<Pat, Vec<Option<ExprOrSpread>>>) -> Vec<VarDecl> {
    map.into_iter()
        .map(|item| {
            let name = item.0;
            let elems = item.1;

            VarDecl {
                declare: false,
                decls: vec![VarDeclarator {
                    definite: false,
                    init: Some(Box::new(Expr::Array(ArrayLit {
                        elems,
                        span: DUMMY_SP,
                    }))),
                    name,
                    span: DUMMY_SP,
                }],
                kind: Const,
                span: DUMMY_SP,
            }
        })
        .collect()
}

/// Update the inner [Vec](Vec) within a [HashMap](HashMap); however, first check if it has yet to be
/// initialized, and if that's the case, initialize it first, then push the new value to it.
pub(crate) fn upsert_map(
    map: &mut HashMap<Pat, Vec<Option<ExprOrSpread>>>,
    key: &Pat,
    value: ExprOrSpread,
) {
    if !map.contains_key(&key) {
        map.insert(key.clone(), vec![]);
    }

    if let Some(inner_items) = map.get_mut(&key) {
        inner_items.push(Some(value))
    }
}
