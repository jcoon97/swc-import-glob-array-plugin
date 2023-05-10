use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use is_glob::is_glob;
use swc_core::ecma::ast::{Decl, ImportDecl, Module, ModuleDecl, ModuleItem, Stmt};
use swc_core::ecma::visit::Fold;
use swc_core::ecma::{ast::Program, visit::FoldWith};
use swc_core::plugin::metadata::TransformPluginMetadataContextKind::{Cwd, Filename};
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};

use crate::transformer::{transform_import_decl, TransformedStatements};

mod imports;
mod transformer;
mod utils;

#[derive(Debug)]
struct ImportGlobArrayPlugin {
    cwd: PathBuf,
    filename: PathBuf,
    id_counter: Rc<RefCell<usize>>,
}

#[derive(Debug)]
struct ImportPaths {
    absolute_path: String,
    imported_path: String,
}

impl ImportGlobArrayPlugin {
    fn build_module_items(&self, transformed: Option<TransformedStatements>) -> Vec<ModuleItem> {
        let mut results: Vec<ModuleItem> = vec![];

        if let Some(transformed) = transformed {
            let TransformedStatements {
                imports,
                names,
                meta,
            } = transformed;

            imports
                .into_iter()
                .for_each(|item| results.push(ModuleItem::ModuleDecl(ModuleDecl::Import(item))));

            names.into_iter().for_each(|item| {
                results.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(item)))))
            });

            meta.into_iter().for_each(|item| {
                results.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(item)))))
            });
        }
        results
    }

    fn get_paths(&self, path: &PathBuf) -> Option<ImportPaths> {
        let path = self.cwd.join(if path.starts_with("/cwd") {
            path.strip_prefix("/cwd").ok()?
        } else {
            path
        });
        let relative_path = path.strip_prefix(&self.cwd).ok()?.to_str()?.to_owned();
        let absolute_path = self.cwd.join(&relative_path).to_str()?.to_owned();
        let imported_path = if relative_path.starts_with('.') {
            relative_path.to_owned()
        } else {
            format!("./{relative_path}")
        };
        Some(ImportPaths {
            absolute_path,
            imported_path,
        })
    }

    fn next_id(&self, starting_id: &str) -> String {
        *self.id_counter.borrow_mut() = self.id_counter.take() + 1;
        format!("{}{}", starting_id, self.id_counter.borrow())
    }

    fn new(cwd: PathBuf, filename: PathBuf) -> impl Fold {
        Self {
            cwd,
            filename,
            id_counter: Rc::new(RefCell::new(0)),
        }
    }
}

impl Fold for ImportGlobArrayPlugin {
    fn fold_module(&mut self, mut module: Module) -> Module {
        module.body = module
            .body
            .into_iter()
            .flat_map(|item| match item {
                ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                    src, specifiers, ..
                })) if (src.value.starts_with('.') || src.value.starts_with('/'))
                    && is_glob(&src.value.to_string()) =>
                {
                    self.build_module_items(transform_import_decl(&self, src, specifiers))
                }
                _ => vec![item],
            })
            .collect();
        module
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let cwd = metadata
        .get_context(&Cwd)
        .map(PathBuf::from)
        .expect("Import Glob Array Plugin required cwd metadata");
    let filename = metadata
        .get_context(&Filename)
        .map(PathBuf::from)
        .expect("Import Glob Array Plugin requires filename metadata");
    let mut plugin = ImportGlobArrayPlugin::new(cwd, filename);
    program.fold_with(&mut plugin)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use swc_core::ecma::transforms::testing::{test_fixture, FixtureTestConfig};
    use swc_core::testing::fixture;

    use crate::ImportGlobArrayPlugin;

    #[fixture("tests/fixtures/**/input.js")]
    fn fixture(input: PathBuf) {
        let cwd = input.parent().unwrap().to_path_buf();
        let output = input.with_file_name("output.js");

        test_fixture(
            Default::default(),
            &|_| ImportGlobArrayPlugin::new(cwd.clone(), input.clone()),
            &input,
            &output,
            FixtureTestConfig {
                allow_error: false,
                sourcemap: false,
            },
        )
    }
}
