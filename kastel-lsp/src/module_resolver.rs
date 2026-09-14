use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct ModuleResolver;

impl ModuleResolver {
    pub fn new(_root: Option<PathBuf>) -> Self {
        Self
    }

    pub fn resolve(&self, current_file: &Path, parts: &[String]) -> Option<PathBuf> {
        if parts.is_empty() {
            return None;
        }

        let current_file = current_file.canonicalize().ok()?;

        let parent = current_file.parent()?;

        let mut path = parent.to_path_buf();

        for part in parts {
            if part.is_empty() || part == "." || part == ".." {
                return None;
            }

            path.push(part);
        }

        path.set_extension("ks");

        if !path.is_file() {
            return None;
        }

        path.canonicalize().ok()
    }

    pub fn path_to_uri(&self, path: &Path) -> Option<String> {
        let path = path.canonicalize().ok()?;

        #[cfg(windows)]
        {
            let mut path = path.to_string_lossy().to_string();

            if let Some(stripped) = path.strip_prefix(r"\\?\") {
                path = stripped.to_string();
            }

            path = path.replace('\\', "/");

            Some(format!("file:///{}", path))
        }

        #[cfg(not(windows))]
        {
            Some(format!("file://{}", path.to_string_lossy()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_dot_module_path() {
        let temp = std::env::temp_dir().join("kastel_lsp_module_test");

        let math_dir = temp.join("math");

        std::fs::create_dir_all(&math_dir).unwrap();

        let main_file = temp.join("main.ks");

        let module_file = math_dir.join("xx.ks");

        std::fs::write(&main_file, "import math.xx").unwrap();

        std::fs::write(&module_file, "const VALUE = 42").unwrap();

        let resolver = ModuleResolver::new(Some(temp.clone()));

        let parts = vec!["math".to_string(), "xx".to_string()];

        let resolved = resolver.resolve(&main_file, &parts).unwrap();

        assert_eq!(resolved, module_file.canonicalize().unwrap());

        let _ = std::fs::remove_dir_all(temp);
    }

    #[test]
    fn windows_path_to_uri_has_no_extended_prefix() {
        let temp = std::env::temp_dir().join("kastel_lsp_uri_test");

        std::fs::create_dir_all(&temp).unwrap();

        let file = temp.join("test.ks");

        std::fs::write(&file, "").unwrap();

        let resolver = ModuleResolver::new(Some(temp.clone()));

        let uri = resolver.path_to_uri(&file).unwrap();

        #[cfg(windows)]
        {
            assert!(uri.starts_with("file:///"));

            assert!(!uri.contains("\\\\?\\"));

            assert!(!uri.contains("\\"));
        }

        #[cfg(not(windows))]
        {
            assert!(uri.starts_with("file://"));
        }

        let _ = std::fs::remove_dir_all(temp);
    }
}
