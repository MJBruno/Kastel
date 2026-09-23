//! Résolution déterministe des chemins d'import Kastel.
//!
//! Trois espaces de résolution, essayés dans cet ordre :
//!
//!   1. `std.*`   -> toujours résolu contre `std_root`, sans repli.
//!      Namespace explicite et réservé : un import qui commence par
//!      `std` ne peut JAMAIS résoudre autre chose que la bibliothèque
//!      standard, même si un fichier `std.ks` existe par ailleurs
//!      dans le projet de l'utilisateur.
//!
//!   2. local     -> relatif au répertoire du module qui importe.
//!      C'est la résolution historique de Kastel : `import utils;`
//!      écrit depuis `models/point.ks` cherche `models/utils.ks`.
//!
//!   3. projet    -> relatif à la racine du projet (le répertoire du
//!      fichier d'entrée passé sur la ligne de commande), en repli
//!      si la résolution locale échoue. Permet d'importer un module
//!      partagé depuis n'importe quelle profondeur de l'arborescence
//!      sans chemins relatifs `../../..`.
//!
//! `ModuleLoader` délègue tout calcul de chemin à `ModuleResolver` et
//! ne s'occupe plus, lui, que du cache et de la détection de cycles.
//! La VM elle-même ne connaît ni `std_root` ni `project_root` : elle
//! demande juste à `ModuleLoader::resolve` un chemin, et transmet le
//! même `ModuleLoader` (donc le même `ModuleResolver`) à chaque
//! module chargé récursivement, pour que `project_root`/`std_root`
//! restent constants sur tout le graphe d'imports d'un programme.

use std::path::{Path, PathBuf};

use crate::error::compile_error::CompileError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportResolution {
    /// `import foo.bar` quand `foo/bar.ks` existe réellement.
    Module(PathBuf),

    /// `import foo.Bar` quand `foo.ks` existe et exporte `Bar`.
    Export { module: PathBuf, name: String },
}

#[derive(Debug, Clone)]
pub struct ModuleResolver {
    project_root: PathBuf,
    std_root: PathBuf,
}

impl ModuleResolver {
    /// `project_root` est la racine de résolution du projet. `std_root`
    /// est déterminé automatiquement — voir `default_std_root`.
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            project_root,
            std_root: Self::default_std_root(),
        }
    }

    /// Impose un `std_root` explicite plutôt que la détection
    /// automatique. Utile pour les tests, ou un déploiement qui
    /// connaît déjà l'emplacement exact de sa bibliothèque standard.
    pub fn with_std_root(mut self, std_root: PathBuf) -> Self {
        self.std_root = std_root;
        self
    }

    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub fn std_root(&self) -> &Path {
        &self.std_root
    }

    /// Résout la sémantique d'un import qualifié de Kastel de manière unique
    /// pour le TypeChecker et la VM : `parts` (le chemin pointé par un
    /// `import` / `from ... import`, écrit depuis le fichier
    /// `current_module`) désigne soit un MODULE, soit un EXPORT d'un module.
    ///
    /// Exemples :
    /// - `import export_mod` -> module `export_mod.ks`
    /// - `import std.math` -> module `std/math.ks`
    /// - `import export_mod.Personne` -> export `Personne` de `export_mod.ks`
    /// - `import std.math.sin` -> export `sin` de `std/math.ks` si
    ///   `std/math/sin.ks` n'existe pas.
    pub fn resolve_import(
        &self,
        current_module: &Path,
        parts: &[String],
    ) -> Result<ImportResolution, CompileError> {
        if parts.is_empty() {
            return Err(CompileError::ModuleInvalidPath(String::new()));
        }

        match self.resolve(current_module, parts) {
            Ok(path) => return Ok(ImportResolution::Module(path)),
            Err(whole_error) => {
                if parts.len() < 2 {
                    return Err(whole_error);
                }
            }
        }

        let split_at = parts.len() - 1;
        let module_parts = &parts[..split_at];
        let export_name = parts[split_at].clone();

        match self.resolve(current_module, module_parts) {
            Ok(module) => Ok(ImportResolution::Export {
                module,
                name: export_name,
            }),
            Err(module_error) => Err(module_error),
        }
    }

    /// Résout `parts` en chemin d'un FICHIER module (pas d'export), selon les
    /// trois espaces décrits en tête de fichier : `std.*`, local, projet.
    /// Le fichier `current_module` n'a pas besoin d'exister : seul son
    /// répertoire compte (c'est le cas du REPL).
    pub fn resolve(
        &self,
        current_module: &Path,
        parts: &[String],
    ) -> Result<PathBuf, CompileError> {
        if parts.is_empty() {
            return Err(CompileError::ModuleInvalidPath(String::new()));
        }

        // 1. Espace de noms réservé `std.*` : jamais de repli vers
        //    local/projet, pour qu'un `import std.x` se comporte de
        //    façon identique quel que soit l'endroit du projet d'où
        //    il est écrit.
        if parts[0] == "std" {
            let rest = &parts[1..];

            if rest.is_empty() {
                return Err(CompileError::ModuleInvalidPath("std".to_string()));
            }

            return Self::resolve_under(&self.std_root, rest);
        }

        // 2. Local : relatif au module courant (comportement
        //    historique de Kastel).
        let current_dir = current_module.parent().unwrap_or_else(|| Path::new("."));

        if let Ok(path) = Self::resolve_under(current_dir, parts) {
            return Ok(path);
        }

        // 3. Repli : relatif à la racine du projet.
        Self::resolve_under(&self.project_root, parts)
    }

    fn resolve_under(root: &Path, parts: &[String]) -> Result<PathBuf, CompileError> {
        let mut path = root.to_path_buf();

        for part in parts {
            path.push(part);
        }

        // ⚠️ Adapte "ks" à l'extension réelle de tes fichiers source
        // Kastel si ce n'est pas celle-ci (ex. "kastel", "kst"...).
        path.set_extension("ks");

        if !path.exists() {
            return Err(CompileError::ModuleNotFound(path.display().to_string()));
        }

        path.canonicalize()
            .map_err(|error| CompileError::ModuleReadError {
                path: path.display().to_string(),
                message: error.to_string(),
            })
    }

    /// Localise la bibliothèque standard, dans l'ordre :
    ///
    ///   1. `KASTEL_STD_PATH` (variable d'environnement, prioritaire —
    ///      utile pour un déploiement personnalisé ou pour les tests) ;
    ///   2. `<répertoire de l'exécutable>/std/` (déploiement normal :
    ///      la std est distribuée à côté du binaire `kastel`) ;
    ///   3. `<CARGO_MANIFEST_DIR>/std/` (confort développeur : `cargo
    ///      run` place l'exécutable dans `target/debug/`, loin de la
    ///      racine du dépôt — ce repli, résolu à la compilation,
    ///      retombe sur le dépôt source connu au moment du build).
    ///
    /// Si aucun de ces répertoires n'existe réellement, on renvoie
    /// quand même le meilleur candidat (le répertoire de
    /// l'exécutable) : l'erreur `ModuleNotFound` produite ensuite par
    /// `resolve_under` nommera alors clairement le chemin attendu,
    /// plutôt que de faire échouer silencieusement la construction du
    /// VM elle-même.
    fn default_std_root() -> PathBuf {
        if let Ok(custom) = std::env::var("KASTEL_STD_PATH") {
            return PathBuf::from(custom);
        }

        let exe_sibling = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|dir| dir.join("std")));

        if let Some(path) = &exe_sibling {
            if path.is_dir() {
                return path.clone();
            }
        }

        let manifest_std = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("std");

        if manifest_std.is_dir() {
            return manifest_std;
        }

        exe_sibling.unwrap_or_else(|| PathBuf::from("std"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    /// Chaque test travaille dans son propre sous-répertoire de
    /// `std::env::temp_dir()`, nettoyé avant et après, pour rester
    /// indépendant de la structure réelle du dépôt Kastel.
    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(name);
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    fn write(path: &Path, content: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    #[test]
    fn resolves_std_namespace_against_std_root() {
        let dir = scratch_dir("kastel_resolver_std_test");

        let project_root = dir.join("project");
        let std_root = dir.join("std");

        write(&std_root.join("collections.ks"), "export const x = 1;");

        let resolver = ModuleResolver::new(project_root.clone()).with_std_root(std_root.clone());

        let current_module = project_root.join("main.ks");

        let resolved = resolver
            .resolve(
                &current_module,
                &["std".to_string(), "collections".to_string()],
            )
            .expect("std.collections should resolve");

        assert_eq!(
            resolved,
            std_root.join("collections.ks").canonicalize().unwrap()
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolves_local_before_project_root() {
        let dir = scratch_dir("kastel_resolver_local_test");

        let project_root = dir.join("project");
        let sub_dir = project_root.join("nested");

        write(&sub_dir.join("helper.ks"), "export const x = 1;");
        write(&project_root.join("helper.ks"), "export const x = 2;");

        let resolver =
            ModuleResolver::new(project_root.clone()).with_std_root(dir.join("std_unused"));

        let current_module = sub_dir.join("main.ks");

        let resolved = resolver
            .resolve(&current_module, &["helper".to_string()])
            .expect("local helper.ks should resolve");

        assert_eq!(resolved, sub_dir.join("helper.ks").canonicalize().unwrap());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn falls_back_to_project_root_when_not_local() {
        let dir = scratch_dir("kastel_resolver_fallback_test");

        let project_root = dir.join("project");
        let sub_dir = project_root.join("nested").join("deeper");

        write(
            &project_root.join("shared").join("utils.ks"),
            "export const x = 1;",
        );

        let resolver =
            ModuleResolver::new(project_root.clone()).with_std_root(dir.join("std_unused"));

        let current_module = sub_dir.join("main.ks");

        let resolved = resolver
            .resolve(
                &current_module,
                &["shared".to_string(), "utils".to_string()],
            )
            .expect("project-root-relative import should resolve");

        assert_eq!(
            resolved,
            project_root
                .join("shared")
                .join("utils.ks")
                .canonicalize()
                .unwrap()
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolves_qualified_import_as_export_when_parent_module_exists() {
        let dir = scratch_dir("kastel_resolver_qualified_export_test");

        let project_root = dir.join("project");
        let demo = project_root.join("demo");
        let std_root = dir.join("std");

        write(&demo.join("export_mod.ks"), "export class Personne {}");

        let resolver = ModuleResolver::new(project_root).with_std_root(std_root);

        let current_module = demo.join("main.ks");
        let resolution = resolver
            .resolve_import(
                &current_module,
                &["export_mod".to_string(), "Personne".to_string()],
            )
            .expect("qualified import should resolve to parent module export");

        match resolution {
            ImportResolution::Export { module, name } => {
                assert_eq!(module, demo.join("export_mod.ks").canonicalize().unwrap());
                assert_eq!(name, "Personne");
            }
            other => panic!("expected export resolution, got {other:?}"),
        }

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn std_namespace_never_falls_back_to_local_or_project() {
        let dir = scratch_dir("kastel_resolver_std_isolation_test");

        let project_root = dir.join("project");
        let std_root = dir.join("std");

        // Un fichier `std.ks` existe localement, mais ne doit JAMAIS
        // être choisi pour un import `std.collections` : le
        // namespace `std` est réservé et pointe uniquement vers
        // `std_root`.
        write(&project_root.join("std.ks"), "export const trap = true;");

        let resolver = ModuleResolver::new(project_root.clone()).with_std_root(std_root.clone());

        let current_module = project_root.join("main.ks");

        let result = resolver.resolve(
            &current_module,
            &["std".to_string(), "collections".to_string()],
        );

        assert!(result.is_err());

        match result {
            Err(CompileError::ModuleNotFound(path)) => {
                assert!(path.contains("std"));
            }
            other => panic!("expected ModuleNotFound pointing at std_root, got {other:?}"),
        }

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn bare_std_import_is_invalid() {
        let dir = scratch_dir("kastel_resolver_bare_std_test");

        let project_root = dir.join("project");

        let resolver = ModuleResolver::new(project_root.clone()).with_std_root(dir.join("std"));

        let current_module = project_root.join("main.ks");

        let result = resolver.resolve(&current_module, &["std".to_string()]);

        assert!(matches!(result, Err(CompileError::ModuleInvalidPath(_))));

        let _ = fs::remove_dir_all(&dir);
    }
}
