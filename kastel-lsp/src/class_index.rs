//! Index des classes et interfaces du document : méthodes, champs
//! (déduits des `this.x = ...`) et classes de base, en s'appuyant
//! directement sur l'AST réel du crate `kastel` (pas d'heuristique
//! textuelle) — voir `Statement::Class` / `Statement::Interface` dans
//! `frontend/ast.rs`.
//!
//! Sert à la complétion et au survol (`this.`, `base.`, et
//! `variable.` quand `variable` a été assignée via `new Classe(...)`).

use std::collections::HashMap;

use kastel::frontend::ast::{AssignmentTarget, Expression, Statement};

#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub name: String,
    pub params: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ClassInfo {
    pub bases: Vec<String>,
    pub methods: Vec<MethodInfo>,
    pub fields: Vec<String>,
    pub is_interface: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ClassIndex {
    classes: HashMap<String, ClassInfo>,
}

impl ClassIndex {
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
        }
    }

    pub fn rebuild(&mut self, statements: &[Statement]) {
        self.classes.clear();

        for statement in statements {
            self.collect_statement(statement);
        }
    }

    pub fn get(&self, name: &str) -> Option<&ClassInfo> {
        self.classes.get(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.classes.contains_key(name)
    }

    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.classes.keys()
    }

    /// Toutes les méthodes accessibles sur `name`, en remontant la
    /// chaîne de classes de base connues dans ce document. `seen`
    /// évite les boucles si une base référence (par erreur) un
    /// ancêtre déjà visité.
    pub fn all_methods(&self, name: &str) -> Vec<&MethodInfo> {
        let mut out = Vec::new();
        let mut seen = std::collections::HashSet::new();

        self.collect_methods(name, &mut out, &mut seen);

        out
    }

    /// Méthodes accessibles via `base.` depuis l'intérieur d'une
    /// méthode de `name` : celles de la/les classes de base de
    /// `name`, pas les siennes propres.
    pub fn base_methods(&self, name: &str) -> Vec<&MethodInfo> {
        let Some(info) = self.classes.get(name) else {
            return Vec::new();
        };

        let mut out = Vec::new();
        let mut seen = std::collections::HashSet::new();
        seen.insert(name.to_string());

        for base in &info.bases {
            self.collect_methods(base, &mut out, &mut seen);
        }

        out
    }

    /// Retourne le nom de la classe qui déclare effectivement `method_name`.
    pub fn method_owner(&self, name: &str, method_name: &str) -> Option<String> {
        let mut seen = std::collections::HashSet::new();
        self.find_method_owner(name, method_name, &mut seen)
    }

    /// Retourne le nom de la classe de base qui déclare effectivement
    /// `method_name`, utilisé pour `base.method()`.
    pub fn base_method_owner(&self, name: &str, method_name: &str) -> Option<String> {
        let info = self.classes.get(name)?;
        let mut seen = std::collections::HashSet::new();
        seen.insert(name.to_string());

        for base in &info.bases {
            if let Some(owner) = self.find_method_owner(base, method_name, &mut seen) {
                return Some(owner);
            }
        }

        None
    }

    fn find_method_owner(
        &self,
        name: &str,
        method_name: &str,
        seen: &mut std::collections::HashSet<String>,
    ) -> Option<String> {
        if !seen.insert(name.to_string()) {
            return None;
        }

        let info = self.classes.get(name)?;

        if info.methods.iter().any(|method| method.name == method_name) {
            return Some(name.to_string());
        }

        for base in &info.bases {
            if let Some(owner) = self.find_method_owner(base, method_name, seen) {
                return Some(owner);
            }
        }

        None
    }

    /// Tous les champs (`this.x`) accessibles sur `name`, en
    /// remontant la chaîne de bases.
    pub fn all_fields(&self, name: &str) -> Vec<&str> {
        let mut out = Vec::new();
        let mut seen = std::collections::HashSet::new();

        self.collect_fields(name, &mut out, &mut seen);

        out
    }

    fn collect_methods<'a>(
        &'a self,
        name: &str,
        out: &mut Vec<&'a MethodInfo>,
        seen: &mut std::collections::HashSet<String>,
    ) {
        if !seen.insert(name.to_string()) {
            return;
        }

        let Some(info) = self.classes.get(name) else {
            return;
        };

        for method in &info.methods {
            out.push(method);
        }

        for base in &info.bases {
            self.collect_methods(base, out, seen);
        }
    }

    fn collect_fields<'a>(
        &'a self,
        name: &str,
        out: &mut Vec<&'a str>,
        seen: &mut std::collections::HashSet<String>,
    ) {
        if !seen.insert(name.to_string()) {
            return;
        }

        let Some(info) = self.classes.get(name) else {
            return;
        };

        for field in &info.fields {
            out.push(field.as_str());
        }

        for base in &info.bases {
            self.collect_fields(base, out, seen);
        }
    }

    fn collect_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Positioned { statement, .. } => {
                self.collect_statement(statement);
            }

            Statement::Export { statement } => {
                self.collect_statement(statement);
            }

            Statement::Class {
                name,
                bases,
                methods,
            } => {
                let mut fields = Vec::new();

                for method in methods {
                    collect_this_fields(&method.body, &mut fields);
                }

                fields.sort();
                fields.dedup();

                self.classes.insert(
                    name.clone(),
                    ClassInfo {
                        bases: bases.clone(),
                        methods: methods
                            .iter()
                            .map(|method| MethodInfo {
                                name: method.name.clone(),
                                params: method.params.clone(),
                            })
                            .collect(),
                        fields,
                        is_interface: false,
                    },
                );
            }

            Statement::Interface {
                name,
                bases,
                methods,
            } => {
                self.classes.insert(
                    name.clone(),
                    ClassInfo {
                        bases: bases.clone(),
                        methods: methods
                            .iter()
                            .map(|method| MethodInfo {
                                name: method.name.clone(),
                                params: (0..method.arity)
                                    .map(|i| format!("arg{}", i + 1))
                                    .collect(),
                            })
                            .collect(),
                        fields: Vec::new(),
                        is_interface: true,
                    },
                );
            }

            _ => {}
        }
    }
}

/// Parcourt récursivement un corps de méthode à la recherche de
/// `this.nom = ...` afin d'en déduire les champs de l'instance.
fn collect_this_fields(body: &[Statement], out: &mut Vec<String>) {
    for statement in body {
        collect_this_fields_in_statement(statement, out);
    }
}

fn collect_this_fields_in_statement(statement: &Statement, out: &mut Vec<String>) {
    match statement {
        Statement::Positioned { statement, .. } => {
            collect_this_fields_in_statement(statement, out);
        }

        Statement::Assignment { target, .. } => {
            if let AssignmentTarget::Member { object, name } = target {
                if matches!(object.as_ref(), Expression::This) {
                    out.push(name.clone());
                }
            }
        }

        Statement::Let { .. } => {}

        Statement::Block(statements) => collect_this_fields(statements, out),

        Statement::If {
            then_branch,
            else_branch,
            ..
        } => {
            collect_this_fields(then_branch, out);

            if let Some(statements) = else_branch {
                collect_this_fields(statements, out);
            }
        }

        Statement::While { body, .. } | Statement::ForIn { body, .. } => {
            collect_this_fields(body, out);
        }

        Statement::Try {
            try_body,
            catch_body,
            finally_body,
            ..
        } => {
            collect_this_fields(try_body, out);

            if let Some(statements) = catch_body {
                collect_this_fields(statements, out);
            }

            if let Some(statements) = finally_body {
                collect_this_fields(statements, out);
            }
        }

        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use kastel::frontend::lexer::lexer::Lexer;
    use kastel::frontend::parser::Parser;

    fn build_index(source: &str) -> ClassIndex {
        let mut lexer = Lexer::new(source.to_owned());
        let tokens = lexer.scan_token().expect("lexer failed");
        let mut parser = Parser::new(tokens);
        let statements = parser.parse().expect("parser failed");

        let mut index = ClassIndex::new();
        index.rebuild(&statements);
        index
    }

    #[test]
    fn collects_methods_and_fields() {
        let source = "class Point {\n\
             func init(x, y) {\n\
             this.x = x;\n\
             this.y = y;\n\
             }\n\
             func to_string() {\n\
             return x;\n\
             }\n\
             }\n";

        let index = build_index(source);

        let info = index.get("Point").expect("Point not indexed");

        let method_names = info
            .methods
            .iter()
            .map(|m| m.name.as_str())
            .collect::<Vec<_>>();

        assert!(method_names.contains(&"init"));
        assert!(method_names.contains(&"to_string"));

        assert_eq!(info.fields, vec!["x".to_string(), "y".to_string()]);
    }

    #[test]
    fn resolves_inherited_methods() {
        let source = "class Animal {\n\
             func speak() { return 1; }\n\
             }\n\
             class Dog : Animal {\n\
             func bark() { return 2; }\n\
             }\n";

        let index = build_index(source);

        let names = index
            .all_methods("Dog")
            .into_iter()
            .map(|m| m.name.as_str())
            .collect::<Vec<_>>();

        assert!(names.contains(&"bark"));
        assert!(names.contains(&"speak"));
    }

    #[test]
    fn base_methods_excludes_own_methods() {
        let source = "class Animal {\n\
             func speak() { return 1; }\n\
             }\n\
             class Dog : Animal {\n\
             func bark() { return 2; }\n\
             }\n";

        let index = build_index(source);

        let names = index
            .base_methods("Dog")
            .into_iter()
            .map(|m| m.name.as_str())
            .collect::<Vec<_>>();

        assert!(names.contains(&"speak"));
        assert!(!names.contains(&"bark"));
    }
}
