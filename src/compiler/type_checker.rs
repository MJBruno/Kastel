use std::collections::{HashMap, HashSet};

use crate::error::compile_error::CompileError;
use crate::frontend::ast::*;

use super::{
    builtin_types,
    compiler::MAX_EXPRESSION_DEPTH,
    module_types::{ImportedType, ModuleTypeInterface, ModuleTypeLoader},
    types::{FunctionType, Type},
};

use std::{path::PathBuf, rc::Rc};

#[derive(Clone)]
pub struct TypeCheckContext {
    pub current_module: PathBuf,
    pub module_loader: Rc<ModuleTypeLoader>,
}

impl TypeCheckContext {
    pub fn new(current_module: PathBuf, module_loader: Rc<ModuleTypeLoader>) -> Self {
        Self {
            current_module,
            module_loader,
        }
    }
}

#[derive(Debug, Clone)]
struct Binding {
    ty: Type,
    _mutable: bool,
    native: bool,
}

/// Ce que le vérificateur sait d'une classe (ou interface) : bases,
/// méthodes surchargées, champs typés, membres privés.
///
/// Exporté avec l'INTERFACE de types d'un module : une classe importée est
/// ainsi vérifiée comme une classe locale (arité des constructeurs,
/// surcharges, champs, visibilité `private`).
#[derive(Debug, Clone)]
pub(crate) struct ClassInfo {
    bases: Vec<String>,
    /// Une classe peut surcharger une méthode par son arité.
    /// Deux signatures de même nom et de même arité restent interdites.
    methods: HashMap<String, Vec<FunctionType>>,
    /// Champs déclarés par `let nom: type = ...;` dans le corps de la classe.
    fields: HashMap<String, Type>,
    /// Membres (champs et méthodes) déclarés `private` dans cette classe.
    private_members: HashSet<String>,
}

/// Vérificateur statique graduel de Kastel.
///
/// Règle centrale : `Dynamic` ne bloque jamais un programme. Une erreur
/// n'est produite que lorsqu'une incompatibilité est certaine à la compilation.
pub struct TypeChecker {
    scopes: Vec<HashMap<String, Binding>>,
    functions: HashMap<String, FunctionType>,
    classes: HashMap<String, ClassInfo>,
    parents: HashMap<String, Vec<String>>,
    current_return_type: Option<Type>,
    return_types: Vec<Type>,
    current_class: Option<String>,
    context: Option<TypeCheckContext>,

    /// Profondeur d'expression courante (voir `MAX_EXPRESSION_DEPTH`).
    expression_depth: usize,

    /// Alias de type du fichier (`type Person = { ... };`), développés par
    /// `resolve_type`. Toujours en `TypeExpr` brut : ils peuvent référencer
    /// d'autres alias LOCAUX déclarés plus loin dans le même fichier.
    aliases: HashMap<String, TypeExpr>,

    /// Alias de type IMPORTÉS d'un autre module (`from m import Person;`,
    /// `import m.Person;`), déjà résolus par le module qui les exporte (voir
    /// `ModuleTypeInterface::type_aliases`) : autonomes, sans référence aux
    /// noms locaux de ce module-ci.
    imported_type_aliases: HashMap<String, Type>,

    /// Signatures de chaque fonction GLOBALE, par nom : plusieurs entrées =
    /// surcharge par arité. Les appels directs (`add(1, 2)`) choisissent la
    /// signature comme pour les méthodes ; le nom lui-même, pris comme valeur,
    /// reste dynamique.
    function_overloads: HashMap<String, Vec<FunctionType>>,

    /// Comme `function_overloads`, pour les fonctions LOCALES : une entrée
    /// par portée (parallèle à `scopes`, l'indice 0 — le global — restant
    /// vide car il utilise `function_overloads`).
    local_functions: Vec<HashMap<String, Vec<FunctionType>>>,
}

impl TypeChecker {
    pub fn check(statements: &[Statement]) -> Result<(), CompileError> {
        let mut checker = Self::new();
        checker.collect_top_level(statements)?;
        checker.check_statements(statements)
    }

    pub fn check_with_context(
        statements: &[Statement],
        context: TypeCheckContext,
    ) -> Result<(), CompileError> {
        let mut checker = Self::new_with_context(context);
        checker.collect_top_level(statements)?;
        checker.check_statements(statements)
    }

    /// Analyse un module sans l'exécuter : exports (valeurs), classes en
    /// détail, et alias de type exportés par `export type X = ...;` (résolus
    /// en `Type` autonome, sans référence aux noms locaux du module source —
    /// voir `resolve_type`).
    pub(crate) fn analyze_module(
        statements: &[Statement],
        context: TypeCheckContext,
    ) -> Result<
        (HashMap<String, Type>, HashMap<String, ClassInfo>, HashMap<String, Type>),
        CompileError,
    > {
        let mut checker = Self::new_with_context(context);
        checker.collect_top_level(statements)?;
        checker.check_statements(statements)?;

        let mut exports = HashMap::new();
        let mut type_aliases = HashMap::new();
        let mut function_exports: HashSet<String> = HashSet::new();

        for statement in statements {
            let Statement::Export { statement } = Self::strip_position(statement) else {
                continue;
            };
            let inner = Self::strip_position(statement);

            // `export type Person = { ... };` : n'entre PAS dans `exports`
            // (aucune valeur à l'exécution), mais dans sa propre table.
            if let Statement::TypeAlias { name, type_expr } = inner {
                let resolved = checker.resolve_type(type_expr);

                if type_aliases.insert(name.clone(), resolved).is_some() {
                    return Err(CompileError::DuplicateExport(name.clone()));
                }

                continue;
            }

            let (name, ty) = checker.export_type(inner)?;

            // Une fonction exportée plusieurs fois (surcharge par arité) ne
            // forme qu'UN export, de type dynamique côté importateur.
            let is_function = matches!(inner, Statement::Function { .. });

            if exports.insert(name.clone(), ty).is_some() {
                // Deuxième `export func` de même nom : `export_type` a déjà
                // rendu l'ensemble COMPLET des signatures (surcharge), que
                // l'insertion vient de mettre en place. Tout autre doublon
                // est une erreur.
                if !(is_function && function_exports.contains(&name)) {
                    return Err(CompileError::DuplicateExport(name));
                }
            }

            if is_function {
                function_exports.insert(name);
            }
        }

        Ok((exports, checker.classes, type_aliases))
    }

    fn new() -> Self {
        let mut globals = HashMap::new();
        for (name, ty) in builtin_types::all() {
            globals.insert(
                name,
                Binding {
                    ty,
                    _mutable: false,
                    native: true,
                },
            );
        }

        Self {
            scopes: vec![globals],
            functions: HashMap::new(),
            classes: HashMap::new(),
            parents: HashMap::new(),
            current_return_type: None,
            return_types: Vec::new(),
            current_class: None,
            expression_depth: 0,
            aliases: HashMap::new(),
            imported_type_aliases: HashMap::new(),
            function_overloads: HashMap::new(),
            local_functions: vec![HashMap::new()],
            context: None,
        }
    }

    fn new_with_context(context: TypeCheckContext) -> Self {
        let mut checker = Self::new();
        checker.context = Some(context);
        checker
    }

    /// Point d'entrée de la collecte : enregistre D'ABORD tous les alias de
    /// type (un alias peut être utilisé avant sa déclaration, dans la
    /// signature d'une fonction par exemple), puis les autres déclarations.
    fn collect_top_level(&mut self, statements: &[Statement]) -> Result<(), CompileError> {
        self.register_aliases(statements)?;
        self.collect_declarations(statements)
    }

    fn register_aliases(&mut self, statements: &[Statement]) -> Result<(), CompileError> {
        for statement in statements {
            // `export type X = ...;` doit être reconnu au même titre qu'un
            // alias non exporté : l'export ne change que sa visibilité pour
            // les AUTRES modules, pas sa disponibilité dans CE fichier. Sans
            // ce déballage, une signature de fonction du même fichier qui
            // utilise un alias EXPORTÉ (même déclaré plus haut) le voyait
            // comme un type nommé non résolu au lieu de sa forme réelle.
            if let Statement::TypeAlias { name, type_expr } =
                Self::strip_position_and_export(statement)
            {
                if self.aliases.contains_key(name) {
                    return Err(CompileError::VariableAlreadyDeclared(name.clone()));
                }

                self.aliases.insert(name.clone(), type_expr.clone());
            }
        }

        Ok(())
    }

    /// Comme `strip_position`, en retirant aussi UN `Export` enveloppant.
    fn strip_position_and_export(statement: &Statement) -> &Statement {
        let statement = Self::strip_position(statement);

        match statement {
            Statement::Export { statement } => Self::strip_position(statement),
            other => other,
        }
    }

    /// Convertit une annotation en type sémantique en développant les ALIAS
    /// (`Person` -> `{ name: str, age: int }`), y compris dans les arguments
    /// génériques, les unions et les champs de records.
    fn resolve_type(&self, expr: &TypeExpr) -> Type {
        self.resolve_type_at(expr, 0)
    }

    fn resolve_type_at(&self, expr: &TypeExpr, depth: usize) -> Type {
        // Alias cyclique (`type A = A;`) : on s'arrête plutôt que de boucler.
        if depth > 32 {
            return Type::Dynamic;
        }

        match expr {
            TypeExpr::Named(name) => {
                if let Some(target) = self.aliases.get(name) {
                    return self.resolve_type_at(target, depth + 1);
                }

                if let Some(resolved) = self.imported_type_aliases.get(name) {
                    return resolved.clone();
                }

                Type::from_type_expr(expr)
            }

            TypeExpr::Generic { name, arguments } => Type::build_generic(
                name,
                arguments
                    .iter()
                    .map(|argument| self.resolve_type_at(argument, depth + 1))
                    .collect(),
            ),

            TypeExpr::Union(members) => Type::union_of(
                members
                    .iter()
                    .map(|member| self.resolve_type_at(member, depth + 1))
                    .collect(),
            ),

            TypeExpr::Record(fields) => Type::Record(
                fields
                    .iter()
                    .map(|(name, field)| (name.clone(), self.resolve_type_at(field, depth + 1)))
                    .collect(),
            ),
        }
    }

    fn collect_declarations(&mut self, statements: &[Statement]) -> Result<(), CompileError> {
        for statement in statements {
            match statement {
                Statement::Positioned { statement, .. } => {
                    self.collect_declarations(std::slice::from_ref(statement))?;
                }

                Statement::Export { statement } => {
                    self.collect_declarations(std::slice::from_ref(statement))?;
                }

                Statement::Function {
                    name,
                    params,
                    param_types,
                    return_type,
                    ..
                } => {
                    let parameters = params
                        .iter()
                        .enumerate()
                        .map(|(index, _)| {
                            param_types
                                .get(index)
                                .and_then(|annotation| annotation.as_ref())
                                .map_or(Type::Dynamic, |annotation| {
                                    self.resolve_type(annotation)
                                })
                        })
                        .collect::<Vec<_>>();

                    let signature = FunctionType {
                        params: parameters,
                        return_type: Box::new(
                            return_type
                                .as_ref()
                                .map(|annotation| self.resolve_type(annotation))
                                .unwrap_or(Type::Dynamic),
                        ),
                    };

                    // Surcharge par arité : les déclarations suivantes du même
                    // nom s'ajoutent à l'ensemble ; le nom devient alors une
                    // valeur dynamique (les appels directs sont résolus par
                    // arité, voir `Expression::Call`).
                    if let Some(overloads) = self.function_overloads.get_mut(name) {
                        if overloads
                            .iter()
                            .any(|existing| existing.params.len() == signature.params.len())
                        {
                            return Err(CompileError::DuplicateFunction {
                                name: name.clone(),
                                arity: signature.params.len(),
                            });
                        }

                        overloads.push(signature);

                        if let Some(binding) = self.scopes[0].get_mut(name) {
                            binding.ty = Type::Dynamic;
                        }

                        self.functions.remove(name);
                    } else {
                        self.function_overloads
                            .insert(name.clone(), vec![signature.clone()]);
                        self.declare_global_declaration(name, Type::Function(signature.clone()))?;
                        self.functions.insert(name.clone(), signature);
                    }
                }

                Statement::Class {
                    name,
                    bases,
                    fields,
                    methods,
                } => {
                    let mut method_map: HashMap<String, Vec<FunctionType>> = HashMap::new();
                    let mut field_map: HashMap<String, Type> = HashMap::new();
                    let mut private_members: HashSet<String> = HashSet::new();

                    for field in fields {
                        field_map.insert(
                            field.name.clone(),
                            field
                                .type_annotation
                                .as_ref()
                                .map(|annotation| self.resolve_type(annotation))
                                .unwrap_or(Type::Dynamic),
                        );

                        if field.visibility == Visibility::Private {
                            private_members.insert(field.name.clone());
                        }
                    }

                    for method in methods {
                        let params = method
                            .params
                            .iter()
                            .enumerate()
                            .map(|(index, _)| {
                                method
                                    .param_types
                                    .get(index)
                                    .and_then(|annotation| annotation.as_ref())
                                    .map_or(Type::Dynamic, |annotation| {
                                        self.resolve_type(annotation)
                                    })
                            })
                            .collect::<Vec<_>>();

                        let return_type = method
                            .return_type
                            .as_ref()
                            .map(|annotation| self.resolve_type(annotation))
                            .unwrap_or(Type::Dynamic);

                        let signature = FunctionType {
                            params,
                            return_type: Box::new(return_type),
                        };

                        let overloads = method_map.entry(method.name.clone()).or_default();

                        if overloads
                            .iter()
                            .any(|existing| existing.params.len() == signature.params.len())
                        {
                            return Err(CompileError::DuplicateMethod {
                                class_name: name.clone(),
                                method_name: method.name.clone(),
                                arity: signature.params.len(),
                            });
                        }

                        overloads.push(signature);

                        if method.visibility == Visibility::Private {
                            private_members.insert(method.name.clone());
                        }
                    }

                    self.parents.insert(name.clone(), bases.clone());
                    self.classes.insert(
                        name.clone(),
                        ClassInfo {
                            bases: bases.clone(),
                            methods: method_map,
                            fields: field_map,
                            private_members,
                        },
                    );
                    self.declare_global_declaration(name, Type::Named(name.clone()))?;
                }

                Statement::Interface {
                    name,
                    bases,
                    methods,
                } => {
                    // Une interface se surcharge comme une classe : le
                    // couple (nom, arité) doit rester unique.
                    let mut method_map: HashMap<String, Vec<FunctionType>> = HashMap::new();

                    for method in methods {
                        let params = (0..method.arity)
                            .map(|index| {
                                method
                                    .param_types
                                    .get(index)
                                    .and_then(|annotation| annotation.as_ref())
                                    .map_or(Type::Dynamic, |annotation| self.resolve_type(annotation))
                            })
                            .collect::<Vec<_>>();

                        let signature = FunctionType {
                            params,
                            return_type: Box::new(
                                method
                                    .return_type
                                    .as_ref()
                                    .map(|annotation| self.resolve_type(annotation))
                                    .unwrap_or(Type::Dynamic),
                            ),
                        };

                        let overloads = method_map.entry(method.name.clone()).or_default();

                        if overloads
                            .iter()
                            .any(|existing| existing.params.len() == signature.params.len())
                        {
                            return Err(CompileError::DuplicateMethod {
                                class_name: name.clone(),
                                method_name: method.name.clone(),
                                arity: signature.params.len(),
                            });
                        }

                        overloads.push(signature);
                    }

                    self.parents.insert(name.clone(), bases.clone());
                    // Enregistrée comme « classe sans corps » : les appels sur
                    // une valeur typée par l'interface sont ainsi vérifiés
                    // (arité + types) via `find_methods`.
                    self.classes.insert(
                        name.clone(),
                        ClassInfo {
                            bases: bases.clone(),
                            methods: method_map,
                            fields: HashMap::new(),
                            private_members: HashSet::new(),
                        },
                    );
                    self.declare_global_declaration(name, Type::Named(name.clone()))?;
                }

                _ => {}
            }
        }

        Ok(())
    }

    fn strip_position(mut statement: &Statement) -> &Statement {
        while let Statement::Positioned {
            statement: inner, ..
        } = statement
        {
            statement = &**inner;
        }
        statement
    }

    fn declare_global_declaration(&mut self, name: &str, ty: Type) -> Result<(), CompileError> {
        let existing_native = self
            .scopes
            .first()
            .and_then(|scope| scope.get(name))
            .is_some_and(|binding| binding.native);

        if let Some(existing) = self.scopes.first().and_then(|scope| scope.get(name)) {
            if !existing.native || !existing_native {
                return Err(CompileError::VariableAlreadyDeclared(name.to_string()));
            }
        }

        self.scopes[0].insert(
            name.to_string(),
            Binding {
                ty,
                _mutable: true,
                native: false,
            },
        );
        Ok(())
    }

    fn check_statements(&mut self, statements: &[Statement]) -> Result<(), CompileError> {
        for statement in statements {
            self.check_statement(statement)?;
        }
        Ok(())
    }

    fn check_statement(&mut self, statement: &Statement) -> Result<(), CompileError> {
        match statement {
            Statement::Positioned {
                line,
                column,
                statement,
            } => self.with_location(*line, *column, |checker| checker.check_statement(statement)),

            Statement::Let {
                name,
                value,
                mutable,
                type_annotation,
            } => {
                let actual = self.check_expression(value)?;
                let declared = type_annotation
                    .as_ref()
                    .map(|annotation| self.resolve_type(annotation))
                    .unwrap_or_else(|| actual.clone());

                self.ensure_assignable(&actual, &declared)?;

                self.declare(
                    name,
                    Binding {
                        ty: declared,
                        _mutable: *mutable,
                        native: false,
                    },
                )
            }

            Statement::Assignment { target, value } => {
                let actual = self.check_expression(value)?;
                self.check_assignment_target(target, &actual)
            }

            Statement::Expression { expression } => {
                self.check_expression(expression)?;
                Ok(())
            }

            Statement::Block(statements) => {
                self.push_scope();
                let result = self.check_statements(statements);
                self.pop_scope();
                result
            }

            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.check_expression(condition)?;
                self.push_scope();
                self.check_statements(then_branch)?;
                self.pop_scope();

                if let Some(else_branch) = else_branch {
                    self.push_scope();
                    self.check_statements(else_branch)?;
                    self.pop_scope();
                }

                Ok(())
            }

            Statement::While { condition, body } => {
                self.check_expression(condition)?;
                self.push_scope();
                let result = self.check_statements(body);
                self.pop_scope();
                result
            }

            Statement::ForIn {
                variable,
                iterable,
                body,
            } => {
                let iterable_type = self.check_expression(iterable)?;
                let element_type = iterable_type.element_type();

                self.push_scope();
                self.declare(
                    variable,
                    Binding {
                        ty: element_type,
                        _mutable: true,
                        native: false,
                    },
                )?;
                let result = self.check_statements(body);
                self.pop_scope();
                result
            }

            Statement::Match { value, arms } => {
                let value_type = self.check_expression(value)?;

                for arm in arms {
                    self.push_scope();
                    self.bind_pattern(&arm.pattern, &value_type)?;

                    if let Some(guard) = &arm.guard {
                        self.check_expression(guard)?;
                    }

                    self.check_statements(&arm.body)?;
                    self.pop_scope();
                }

                Ok(())
            }

            Statement::Throw { value } => {
                self.check_expression(value)?;
                Ok(())
            }

            Statement::Try {
                try_body,
                catch_name,
                catch_body,
                finally_body,
            } => {
                self.push_scope();
                self.check_statements(try_body)?;
                self.pop_scope();

                if let Some(catch_body) = catch_body {
                    self.push_scope();
                    if let Some(name) = catch_name {
                        self.declare(
                            name,
                            Binding {
                                ty: Type::Dynamic,
                                _mutable: true,
                                native: false,
                            },
                        )?;
                    }
                    self.check_statements(catch_body)?;
                    self.pop_scope();
                }

                if let Some(finally_body) = finally_body {
                    self.push_scope();
                    self.check_statements(finally_body)?;
                    self.pop_scope();
                }

                Ok(())
            }

            Statement::Function {
                name,
                params,
                param_types,
                return_type,
                body,
            } => self.check_function(name, params, param_types, return_type.as_ref(), body),

            Statement::Return { value } => {
                let actual = value
                    .as_ref()
                    .map(|expression| self.check_expression(expression))
                    .transpose()?
                    .unwrap_or(Type::None);

                if let Some(expected) = &self.current_return_type {
                    self.ensure_assignable(&actual, expected)?;
                }

                self.return_types.push(actual);
                Ok(())
            }

            Statement::Import { path } => self.check_import(path),

            Statement::FromImport { module, items } => self.check_from_import(module, items),

            Statement::Export { statement } => self.check_statement(statement),

            Statement::Class { name, methods, .. } => self.check_class(name, methods),

            Statement::Interface { .. } => Ok(()),

            // Enregistré par `register_aliases` pour les alias de premier
            // niveau ; un alias déclaré dans un bloc s'enregistre ici.
            Statement::TypeAlias { name, type_expr } => {
                self.aliases
                    .entry(name.clone())
                    .or_insert_with(|| type_expr.clone());

                Ok(())
            }

            Statement::Break | Statement::Continue => Ok(()),
        }
    }

    fn check_import(&mut self, path: &[String]) -> Result<(), CompileError> {
        let imported = self.resolve_import(path)?;
        let binding_name = path.last().ok_or(CompileError::InvalidImport)?;

        self.declare_import_binding(binding_name, imported)
    }

    fn check_from_import(
        &mut self,
        module: &ModulePath,
        items: &[ImportItem],
    ) -> Result<(), CompileError> {
        let context = self.context.as_ref().ok_or(CompileError::InvalidImport)?;
        let module_path = context
            .module_loader
            .resolver()
            .resolve(&context.current_module, &module.parts)?;
        let interface = context.module_loader.interface(&module_path)?;

        if items.len() == 1 && items[0].name == "*" {
            for (name, ty) in &interface.exports {
                self.declare_import_binding(
                    name,
                    ImportedType::Export {
                        ty: ty.clone(),
                        name: name.clone(),
                        interface: Rc::clone(&interface),
                    },
                )?;
            }

            // Les alias de type exportés entrent aussi dans le `*`.
            for (name, resolved) in &interface.type_aliases {
                self.declare_import_binding(
                    name,
                    ImportedType::TypeAlias {
                        resolved: resolved.clone(),
                    },
                )?;
            }

            return Ok(());
        }

        for item in items {
            if item.name == "*" {
                return Err(CompileError::InvalidImport);
            }

            let binding_name = item.alias.as_deref().unwrap_or(&item.name);

            if let Some(ty) = interface.exports.get(&item.name).cloned() {
                self.declare_import_binding(
                    binding_name,
                    ImportedType::Export {
                        ty,
                        name: item.name.clone(),
                        interface: Rc::clone(&interface),
                    },
                )?;
                continue;
            }

            // `from m import Person;` : Person n'est qu'un alias de type,
            // sans valeur à l'exécution (voir `declare_import_binding`).
            if let Some(resolved) = interface.type_aliases.get(&item.name).cloned() {
                self.declare_import_binding(binding_name, ImportedType::TypeAlias { resolved })?;
                continue;
            }

            return Err(CompileError::ExportNotFound {
                module: module.parts.join("."),
                name: item.name.clone(),
            });
        }

        Ok(())
    }

    fn resolve_import(&self, path: &[String]) -> Result<ImportedType, CompileError> {
        let context = self.context.as_ref().ok_or(CompileError::InvalidImport)?;
        context
            .module_loader
            .resolve_import(&context.current_module, path)
    }

    /// Enregistre la classe `exported_name` d'un module importé, et ses bases
    /// (transitivement), sous leurs noms d'origine. Une classe déjà connue
    /// localement sous ce nom n'est pas remplacée.
    fn import_class_info(&mut self, interface: &ModuleTypeInterface, exported_name: &str) {
        let mut pending = vec![exported_name.to_string()];
        let mut visited = HashSet::new();

        while let Some(name) = pending.pop() {
            if !visited.insert(name.clone()) {
                continue;
            }

            let Some(info) = interface.classes.get(&name) else {
                continue;
            };

            self.classes
                .entry(name.clone())
                .or_insert_with(|| info.clone());
            self.parents
                .entry(name.clone())
                .or_insert_with(|| info.bases.clone());

            pending.extend(info.bases.iter().cloned());
        }
    }

    /// Nom de la classe désignée par `name`, en suivant un alias d'import
    /// (`from m import Personne as P`).
    fn canonical_class_name(&self, name: &str) -> String {
        match self.aliases.get(name) {
            Some(TypeExpr::Named(target)) => target.clone(),
            _ => name.to_string(),
        }
    }

    fn declare_import_binding(
        &mut self,
        binding_name: &str,
        imported: ImportedType,
    ) -> Result<(), CompileError> {
        // Un alias de type n'a AUCUNE existence à l'exécution : pas de
        // liaison-valeur à déclarer, juste le type rendu disponible sous
        // `binding_name` (voir `resolve_type_at`).
        if let ImportedType::TypeAlias { resolved } = imported {
            self.imported_type_aliases
                .insert(binding_name.to_string(), resolved);

            return Ok(());
        }

        let ty = match imported {
            ImportedType::Module(path) => Type::Module(path.to_string_lossy().into_owned()),

            ImportedType::TypeAlias { .. } => unreachable!("traité ci-dessus"),

            ImportedType::Export { ty, name, interface } => {
                // Une classe importée est connue en détail (constructeurs,
                // méthodes, champs, membres privés), pas seulement par son nom.
                if interface.classes.contains_key(&name) {
                    self.import_class_info(&interface, &name);

                    // `from m import Personne as P` : `P` désigne `Personne`.
                    if binding_name != name {
                        self.aliases
                            .insert(binding_name.to_string(), TypeExpr::Named(name));
                    }
                }

                ty
            }
        };

        let is_global_scope = self.scopes.len() == 1;
        let native_collision = self
            .scopes
            .last()
            .and_then(|scope| scope.get(binding_name))
            .is_some_and(|binding| binding.native);

        if is_global_scope && native_collision {
            let scope = self
                .scopes
                .last_mut()
                .expect("TypeChecker scope is never empty");
            scope.insert(
                binding_name.to_string(),
                Binding {
                    ty,
                    _mutable: false,
                    native: false,
                },
            );
            return Ok(());
        }

        self.declare(
            binding_name,
            Binding {
                ty,
                _mutable: false,
                native: false,
            },
        )
    }

    fn export_type(&self, statement: &Statement) -> Result<(String, Type), CompileError> {
        match statement {
            Statement::Let { name, .. } => self
                .lookup(name)
                .map(|binding| (name.clone(), binding.ty))
                .ok_or_else(|| CompileError::UndefinedVariable {
                    name: name.clone(),
                    suggestion: None,
                }),

            // Fonction surchargée : UN export, typé par l'ensemble de ses
            // signatures (l'importateur choisit par arité et par type).
            Statement::Function { name, .. }
                if self
                    .function_overloads
                    .get(name)
                    .is_some_and(|overloads| overloads.len() > 1) =>
            {
                let signatures = self.function_overloads.get(name).cloned().unwrap_or_default();

                Ok((name.clone(), Type::Overloads(signatures)))
            }

            Statement::Function { name, .. } => self
                .functions
                .get(name)
                .cloned()
                .map(|signature| (name.clone(), Type::Function(signature)))
                .ok_or_else(|| CompileError::UndefinedVariable {
                    name: name.clone(),
                    suggestion: None,
                }),

            Statement::Class { name, .. } | Statement::Interface { name, .. } => {
                Ok((name.clone(), Type::Named(name.clone())))
            }

            _ => Err(CompileError::InvalidExport),
        }
    }

    fn check_function(
        &mut self,
        name: &str,
        params: &[String],
        param_types: &[Option<TypeExpr>],
        return_type: Option<&TypeExpr>,
        body: &[Statement],
    ) -> Result<(), CompileError> {
        let previous_return = self.current_return_type.take();
        let previous_returns = std::mem::take(&mut self.return_types);

        let declared_signature = FunctionType {
            params: params
                .iter()
                .enumerate()
                .map(|(index, _)| {
                    param_types
                        .get(index)
                        .and_then(|annotation| annotation.as_ref())
                        .map_or(Type::Dynamic, |annotation| self.resolve_type(annotation))
                })
                .collect(),
            return_type: Box::new(
                return_type
                    .map(|annotation| self.resolve_type(annotation))
                    .unwrap_or(Type::Dynamic),
            ),
        };

        let nested = self.scopes.len() > 1;
        let parent_scope_index = self.scopes.len() - 1;
        if nested {
            // Surcharge locale : une fonction du même nom est déjà déclarée
            // dans CETTE portée (même arité = erreur).
            if let Some(overloads) = self.local_functions[parent_scope_index].get_mut(name) {
                if overloads
                    .iter()
                    .any(|existing| existing.params.len() == declared_signature.params.len())
                {
                    return Err(CompileError::DuplicateFunction {
                        name: name.to_string(),
                        arity: declared_signature.params.len(),
                    });
                }

                overloads.push(declared_signature.clone());

                if let Some(binding) = self.scopes[parent_scope_index].get_mut(name) {
                    binding.ty = Type::Dynamic;
                }
            } else {
                self.declare(
                    name,
                    Binding {
                        ty: Type::Function(declared_signature.clone()),
                        _mutable: true,
                        native: false,
                    },
                )?;

                self.local_functions[parent_scope_index]
                    .insert(name.to_string(), vec![declared_signature.clone()]);
            }
        }

        self.push_scope();

        for (index, parameter) in params.iter().enumerate() {
            let ty = param_types
                .get(index)
                .and_then(|annotation| annotation.as_ref())
                .map_or(Type::Dynamic, |annotation| self.resolve_type(annotation));

            self.declare(
                parameter,
                Binding {
                    ty,
                    _mutable: true,
                    native: false,
                },
            )?;
        }

        self.current_return_type = return_type.map(|annotation| self.resolve_type(annotation));
        self.check_statements(body)?;

        let inferred_return = self.infer_return_type();
        let missing_annotated_return = self
            .current_return_type
            .clone()
            .filter(|_| self.return_types.is_empty());

        if let Some(expected) = missing_annotated_return {
            if self.return_types.is_empty() {
                self.pop_scope();
                self.current_return_type = previous_return;
                self.return_types = previous_returns;

                return Err(CompileError::TypeMismatch {
                    expected: expected.to_string(),
                    found: Type::None.to_string(),
                });
            }
        }

        if return_type.is_none() {
            if let Some(function) = self.functions.get_mut(name) {
                function.return_type = Box::new(inferred_return.clone());
            }

            // Fonction locale surchargée : signature de MÊME arité.
            if nested
                && let Some(overloads) = self.local_functions[parent_scope_index].get_mut(name)
                && let Some(signature) = overloads
                    .iter_mut()
                    .find(|signature| signature.params.len() == params.len())
            {
                signature.return_type = Box::new(inferred_return.clone());
            }

            // Fonction surchargée : on met à jour la signature de MÊME arité.
            if !nested
                && let Some(overloads) = self.function_overloads.get_mut(name)
                && let Some(signature) = overloads
                    .iter_mut()
                    .find(|signature| signature.params.len() == params.len())
            {
                signature.return_type = Box::new(inferred_return.clone());
            }

            if nested {
                if let Some(binding) = self.scopes[parent_scope_index].get_mut(name) {
                    if let Type::Function(signature) = &mut binding.ty {
                        signature.return_type = Box::new(inferred_return);
                    }
                }
            }
        }

        self.pop_scope();
        self.current_return_type = previous_return;
        self.return_types = previous_returns;

        Ok(())
    }

    fn check_class(
        &mut self,
        class_name: &str,
        methods: &[FunctionMethod],
    ) -> Result<(), CompileError> {
        let previous_class = self.current_class.take();
        self.current_class = Some(class_name.to_string());

        for method in methods {
            self.check_method(class_name, method)?;
        }

        self.current_class = previous_class;
        Ok(())
    }

    fn check_method(
        &mut self,
        class_name: &str,
        method: &FunctionMethod,
    ) -> Result<(), CompileError> {
        let previous_return = self.current_return_type.take();
        let previous_returns = std::mem::take(&mut self.return_types);

        self.push_scope();

        self.declare(
            "this",
            Binding {
                ty: Type::Named(class_name.to_string()),
                _mutable: true,
                native: false,
            },
        )?;

        for (index, parameter) in method.params.iter().enumerate() {
            let ty = method
                .param_types
                .get(index)
                .and_then(|annotation| annotation.as_ref())
                .map_or(Type::Dynamic, |annotation| self.resolve_type(annotation));

            self.declare(
                parameter,
                Binding {
                    ty,
                    _mutable: true,
                    native: false,
                },
            )?;
        }

        self.current_return_type = method.return_type.as_ref().map(|annotation| self.resolve_type(annotation));
        self.check_statements(&method.body)?;

        let inferred_return = self.infer_return_type();
        let missing_annotated_return = self
            .current_return_type
            .clone()
            .filter(|_| self.return_types.is_empty());

        if let Some(expected) = missing_annotated_return {
            if self.return_types.is_empty() {
                self.pop_scope();
                self.current_return_type = previous_return;
                self.return_types = previous_returns;

                return Err(CompileError::TypeMismatch {
                    expected: expected.to_string(),
                    found: Type::None.to_string(),
                });
            }
        }

        if let Some(class) = self.classes.get_mut(class_name) {
            if let Some(overloads) = class.methods.get_mut(&method.name) {
                if let Some(signature) = overloads
                    .iter_mut()
                    .find(|signature| signature.params.len() == method.params.len())
                    && method.return_type.is_none()
                {
                    signature.return_type = Box::new(inferred_return);
                }
            }
        }

        self.pop_scope();
        self.current_return_type = previous_return;
        self.return_types = previous_returns;

        Ok(())
    }

    fn infer_return_type(&self) -> Type {
        self.return_types
            .iter()
            .cloned()
            .reduce(|left, right| left.merge(&right))
            .unwrap_or(Type::None)
    }

    fn check_assignment_target(
        &mut self,
        target: &AssignmentTarget,
        actual: &Type,
    ) -> Result<(), CompileError> {
        match target {
            AssignmentTarget::Variable(name) => {
                if let Some(binding) = self.lookup(name) {
                    let expected = binding.ty.clone();
                    self.ensure_assignable(actual, &expected)
                } else {
                    Ok(())
                }
            }

            AssignmentTarget::Index { object, index } => {
                let object_type = self.check_expression(object)?;
                let index_type = self.check_expression(index)?;

                match object_type {
                    Type::Array(_) | Type::ArrayDynamic | Type::Tuple(_) | Type::TupleDynamic => {
                        if !matches!(index_type, Type::Int | Type::Float | Type::Dynamic) {
                            return Err(CompileError::TypeMismatch {
                                expected: "int".to_string(),
                                found: index_type.to_string(),
                            });
                        }

                        let expected = object_type.element_type();
                        self.ensure_assignable(actual, &expected)
                    }

                    Type::Dict(key, value) => {
                        self.ensure_assignable(&index_type, &key)?;
                        self.ensure_assignable(actual, &value)
                    }

                    Type::DictDynamic => Ok(()),
                    Type::Dynamic => Ok(()),
                    _ => Ok(()),
                }
            }

            AssignmentTarget::Member { object, name } => {
                let object_type = self.check_expression(object)?;

                if let Type::Named(class_name) = &object_type {
                    self.check_member_visibility(class_name, name)?;

                    // `this.age = valeur` : la valeur doit respecter le type
                    // déclaré du champ (`let age: int`).
                    if let Some(expected) = self.find_field(class_name, name) {
                        self.ensure_assignable(actual, &expected)?;
                    }
                }

                // Record : on ne modifie qu'un champ EXISTANT, avec un type
                // compatible.
                if let Type::Record(fields) = &object_type {
                    match fields.iter().find(|(field, _)| field == name) {
                        Some((_, expected)) => self.ensure_assignable(actual, expected)?,

                        None => {
                            return Err(CompileError::InvalidMemberAccess {
                                name: name.to_string(),
                            });
                        }
                    }
                }

                Ok(())
            }
        }
    }

    fn check_expression(&mut self, expression: &Expression) -> Result<Type, CompileError> {
        if self.expression_depth >= MAX_EXPRESSION_DEPTH {
            return Err(CompileError::ExpressionTooDeep {
                limit: MAX_EXPRESSION_DEPTH,
            });
        }

        self.expression_depth += 1;
        let result = self.check_expression_inner(expression);
        self.expression_depth -= 1;

        result
    }

    fn check_expression_inner(&mut self, expression: &Expression) -> Result<Type, CompileError> {
        match expression {
            Expression::Literal(literal) => Ok(match literal {
                Literal::Integer(_) => Type::Int,
                Literal::Float(_) => Type::Float,
                Literal::String(_) => Type::Str,
                Literal::Bool(_) => Type::Bool,
                Literal::None => Type::None,
            }),

            Expression::Variable(name) => Ok(self
                .lookup(name)
                .map(|binding| binding.ty)
                .or_else(|| self.functions.get(name).cloned().map(Type::Function))
                .unwrap_or(Type::Dynamic)),

            Expression::Unary {
                operator, right, ..
            } => {
                let right_type = self.check_expression(right)?;

                match operator {
                    UnaryOp::Not => Ok(Type::Bool),
                    UnaryOp::Negate => {
                        // `-x` avec `x: int | float` : chaque membre doit être
                        // numérique.
                        if let Type::Union(members) = &right_type
                            && members.iter().all(|member| member.numeric_kind().is_some())
                        {
                            Ok(right_type.clone())
                        } else if right_type.is_dynamic() {
                            Ok(Type::Dynamic)
                        } else if right_type.numeric_kind().is_some() {
                            Ok(right_type)
                        } else {
                            Err(CompileError::InvalidUnaryOperation {
                                operator: "-".to_string(),
                                found: right_type.to_string(),
                            })
                        }
                    }
                    UnaryOp::BitNot => {
                        if right_type.is_dynamic() || right_type == Type::Int {
                            Ok(Type::Int)
                        } else {
                            Err(CompileError::InvalidUnaryOperation {
                                operator: "~".to_string(),
                                found: right_type.to_string(),
                            })
                        }
                    }
                }
            }

            Expression::Binary {
                left,
                operator,
                right,
                ..
            } => self.check_binary(left, operator, right),

            Expression::Function { params, body } => {
                let previous_return = self.current_return_type.take();
                let previous_returns = std::mem::take(&mut self.return_types);

                self.push_scope();
                for parameter in params {
                    self.declare(
                        parameter,
                        Binding {
                            ty: Type::Dynamic,
                            _mutable: true,
                            native: false,
                        },
                    )?;
                }

                self.current_return_type = None;
                self.check_statements(body)?;
                let return_type = self.infer_return_type();
                self.pop_scope();

                self.current_return_type = previous_return;
                self.return_types = previous_returns;

                Ok(Type::Function(FunctionType {
                    params: vec![Type::Dynamic; params.len()],
                    return_type: Box::new(return_type),
                }))
            }

            Expression::Call {
                callee, arguments, ..
            } => {
                // Fonction globale SURCHARGÉE (`add(1)`, `add(1, 2)`) : la
                // signature est choisie par arité et par type, comme pour une
                // méthode.
                if let Expression::Variable(name) = callee.as_ref()
                    && let Some(signatures) = self.overloads_of(name)
                {
                    let signature = self.resolve_overload(&signatures, arguments, name)?;

                    return Ok(*signature.return_type);
                }

                // `Set(a, b, c)` (native, non redéfinie) : le type d'élément
                // est déduit des arguments -> `Set<int>` pour `Set(1, 2, 3)`.
                if let Expression::Variable(name) = callee.as_ref()
                    && name == "Set"
                    && self.lookup(name).is_some_and(|binding| binding.native)
                {
                    let mut element: Option<Type> = None;

                    for argument in arguments {
                        let argument_type = self.check_expression(argument)?;

                        element = Some(match element {
                            Some(current) => current.merge(&argument_type),
                            None => argument_type,
                        });
                    }

                    return Ok(Type::Set(Box::new(element.unwrap_or(Type::Dynamic))));
                }

                // Pour une méthode de classe, l'arité fait partie de la
                // résolution. Cela permet `obj.foo()` et `obj.foo(x)`
                // d'aboutir à deux signatures différentes.
                if let Expression::Member { object, name, .. } = callee.as_ref() {
                    let object_type = self.check_expression(object)?;

                    if let Type::Named(class_name) = &object_type {
                        self.check_member_visibility(class_name, name)?;

                        let signatures = self.find_methods(class_name, name);

                        if !signatures.is_empty() {
                            let signature = self.resolve_overload(
                                &signatures,
                                arguments,
                                &format!("{class_name}.{name}"),
                            )?;

                            return Ok(*signature.return_type);
                        }
                    }

                    // Array, Dict, Tuple, String, Range : méthodes STANDARD
                    // typées pour un APPEL (`a.size()`, `d.get(k)`...), et
                    // erreur guidée pour les noms supprimés (`a.length`,
                    // `a.push(x)`, `d.has(k)`...).
                    if let Some(replacement) = object_type.renamed_member(name) {
                        return Err(CompileError::RenamedMember {
                            name: name.to_string(),
                            replacement: replacement.to_string(),
                        });
                    }

                    if !matches!(object_type, Type::Set(_) | Type::SetDynamic)
                        && let Some(Type::Function(signature)) =
                            object_type.collection_member_type(name)
                    {
                        return self.check_call_signature(&signature, arguments, name);
                    }
                }

                let callee_type = self.check_expression(callee)?;

                match callee_type {
                    Type::Function(signature) => {
                        let function_name = self.expression_name(callee);
                        self.check_call_signature(&signature, arguments, &function_name)
                    }

                    // Fonction surchargée importée d'un module : signature
                    // choisie par arité et par type.
                    Type::Overloads(signatures) => {
                        let function_name = self.expression_name(callee);
                        let signature =
                            self.resolve_overload(&signatures, arguments, &function_name)?;

                        Ok(*signature.return_type)
                    }

                    Type::Dynamic => {
                        for argument in arguments {
                            self.check_expression(argument)?;
                        }
                        Ok(Type::Dynamic)
                    }

                    other => Err(CompileError::NotCallable {
                        found: other.to_string(),
                    }),
                }
            }

            Expression::Array(elements) => {
                if elements.is_empty() {
                    return Ok(Type::Array(Box::new(Type::Dynamic)));
                }

                let mut element_type = self.check_expression(&elements[0])?;
                for element in &elements[1..] {
                    let next = self.check_expression(element)?;
                    element_type = element_type.merge(&next);
                }
                Ok(Type::Array(Box::new(element_type)))
            }

            Expression::Tuple(elements) => {
                let mut types = Vec::with_capacity(elements.len());
                for element in elements {
                    types.push(self.check_expression(element)?);
                }
                Ok(Type::Tuple(types))
            }

            // Record `{ name: "Bruno", age: 25 }` : type structurel, un champ
            // par clé.
            Expression::Record(fields) => {
                let mut typed_fields = Vec::with_capacity(fields.len());

                for (name, value) in fields {
                    typed_fields.push((name.clone(), self.check_expression(value)?));
                }

                Ok(Type::Record(typed_fields))
            }

            Expression::Dict(fields) => {
                if fields.is_empty() {
                    return Ok(Type::Dict(Box::new(Type::Str), Box::new(Type::Dynamic)));
                }

                let mut value_type = self.check_expression(&fields[0].1)?;
                for (_, value) in &fields[1..] {
                    value_type = value_type.merge(&self.check_expression(value)?);
                }

                Ok(Type::Dict(Box::new(Type::Str), Box::new(value_type)))
            }

            Expression::Index { object, index, .. } => {
                let object_type = self.check_expression(object)?;
                let index_type = self.check_expression(index)?;

                match object_type {
                    Type::Array(_) | Type::ArrayDynamic | Type::Tuple(_) | Type::TupleDynamic => {
                        if !matches!(index_type, Type::Int | Type::Float | Type::Dynamic) {
                            return Err(CompileError::TypeMismatch {
                                expected: "int".to_string(),
                                found: index_type.to_string(),
                            });
                        }
                        // `t[0]` sur un `Tuple<int, str>` : type EXACT de
                        // l'élément quand l'index est un littéral entier.
                        if let (
                            Type::Tuple(elements),
                            Expression::Literal(Literal::Integer(position)),
                        ) = (&object_type, index.as_ref())
                            && let Ok(position) = usize::try_from(*position)
                            && let Some(element) = elements.get(position)
                        {
                            return Ok(element.clone());
                        }

                        Ok(object_type.element_type())
                    }
                    Type::Dict(key, value) => {
                        self.ensure_assignable(&index_type, &key)?;
                        Ok(*value)
                    }
                    Type::DictDynamic | Type::Dynamic => Ok(Type::Dynamic),
                    _ => Ok(Type::Dynamic),
                }
            }

            Expression::Member { object, name, .. } => {
                let object_type = self.check_expression(object)?;
                self.member_type(&object_type, name)
            }

            Expression::New {
                class_name,
                arguments,
                ..
            } => {
                let class_name = &self.canonical_class_name(class_name);

                // Constructeur `private` : `new` n'est permis que dans le
                // corps de la classe qui le déclare.
                self.check_member_visibility(class_name, CONSTRUCTOR_NAME)?;

                let signatures = self.find_methods(class_name, CONSTRUCTOR_NAME);

                if !signatures.is_empty() {
                    self.resolve_overload(
                        &signatures,
                        arguments,
                        &format!("{class_name}.{CONSTRUCTOR_NAME}"),
                    )?;
                } else {
                    // Aucun constructeur déclaré : on évalue quand même les
                    // arguments (typage graduel)...
                    for argument in arguments {
                        self.check_expression(argument)?;
                    }

                    // ...mais si toute la hiérarchie est connue, c'est le
                    // constructeur par défaut implicite qui s'applique : il
                    // n'accepte aucun argument. Pour une classe importée (ou
                    // dont une base est inconnue), c'est la VM qui tranche.
                    if !arguments.is_empty() && self.hierarchy_is_known(class_name) {
                        return Err(CompileError::WrongArgumentCount {
                            expected: 0,
                            found: arguments.len(),
                        });
                    }
                }

                Ok(Type::Named(class_name.clone()))
            }

            Expression::This => Ok(self
                .current_class
                .as_ref()
                .map(|name| Type::Named(name.clone()))
                .unwrap_or(Type::Dynamic)),

            Expression::Base => Ok(Type::Dynamic),

            Expression::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.check_expression(condition)?;
                let then_type = self.check_expression(then_expr)?;
                let else_type = self.check_expression(else_expr)?;
                Ok(then_type.merge(&else_type))
            }
        }
    }

    fn check_binary(
        &mut self,
        left: &Expression,
        operator: &BinaryOp,
        right: &Expression,
    ) -> Result<Type, CompileError> {
        let left_type = self.check_expression(left)?;
        let right_type = self.check_expression(right)?;

        self.binary_type(operator, left_type, right_type)
    }

    /// Type du résultat de `left operator right`. Un opérande UNION
    /// (`int | float`) est typé pour chaque combinaison de ses membres ; les
    /// résultats sont fusionnés (`Number + int` -> `int | float`).
    fn binary_type(
        &self,
        operator: &BinaryOp,
        left_type: Type,
        right_type: Type,
    ) -> Result<Type, CompileError> {
        if matches!(left_type, Type::Union(_)) || matches!(right_type, Type::Union(_)) {
            let mut results = Vec::new();

            for left_member in left_type.members() {
                for right_member in right_type.members() {
                    results.push(self.binary_type(operator, left_member.clone(), right_member)?);
                }
            }

            return Ok(Type::union_of(results));
        }

        match operator {
            BinaryOp::And | BinaryOp::Or => Ok(left_type.merge(&right_type)),

            BinaryOp::Equal | BinaryOp::NotEqual | BinaryOp::Is => Ok(Type::Bool),

            BinaryOp::Add => {
                if left_type.is_dynamic() && right_type == Type::Str {
                    return Ok(Type::Str);
                }

                if right_type.is_dynamic() && left_type == Type::Str {
                    return Ok(Type::Str);
                }

                if left_type.is_dynamic() {
                    if right_type.numeric_kind().is_some() {
                        return Ok(right_type);
                    }
                    return Ok(Type::Dynamic);
                }

                if right_type.is_dynamic() {
                    if left_type.numeric_kind().is_some() {
                        return Ok(left_type);
                    }
                    return Ok(Type::Dynamic);
                }

                if left_type == Type::Str && right_type == Type::Str {
                    return Ok(Type::Str);
                }

                if let (Some(left), Some(right)) =
                    (left_type.numeric_kind(), right_type.numeric_kind())
                {
                    return Ok(match (left, right) {
                        (super::types::NumericType::Int, super::types::NumericType::Int) => {
                            Type::Int
                        }
                        _ => Type::Float,
                    });
                }

                Err(CompileError::InvalidBinaryOperation {
                    operator: "+".to_string(),
                    left: left_type.to_string(),
                    right: right_type.to_string(),
                })
            }

            BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => {
                // `/` renvoie TOUJOURS un flottant à l'exécution : même avec
                // un opérande dynamique, le résultat n'est jamais un `int`.
                let is_divide = matches!(operator, BinaryOp::Divide);

                if left_type.is_dynamic() {
                    if right_type.numeric_kind().is_some() {
                        return Ok(if is_divide { Type::Float } else { right_type });
                    }
                    return Ok(Type::Dynamic);
                }

                if right_type.is_dynamic() {
                    if left_type.numeric_kind().is_some() {
                        return Ok(if is_divide { Type::Float } else { left_type });
                    }
                    return Ok(Type::Dynamic);
                }

                if let (Some(left), Some(right)) =
                    (left_type.numeric_kind(), right_type.numeric_kind())
                {
                    return Ok(match operator {
                        BinaryOp::Divide => Type::Float,
                        BinaryOp::Modulo => match (left, right) {
                            (super::types::NumericType::Int, super::types::NumericType::Int) => {
                                Type::Int
                            }
                            _ => Type::Float,
                        },
                        _ => match (left, right) {
                            (super::types::NumericType::Int, super::types::NumericType::Int) => {
                                Type::Int
                            }
                            _ => Type::Float,
                        },
                    });
                }

                Err(CompileError::InvalidBinaryOperation {
                    operator: binary_symbol(operator).to_string(),
                    left: left_type.to_string(),
                    right: right_type.to_string(),
                })
            }

            BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual => {
                if left_type.is_dynamic() || right_type.is_dynamic() {
                    return Ok(Type::Bool);
                }

                if left_type.numeric_kind().is_some() && right_type.numeric_kind().is_some() {
                    return Ok(Type::Bool);
                }

                Err(CompileError::InvalidBinaryOperation {
                    operator: binary_symbol(operator).to_string(),
                    left: left_type.to_string(),
                    right: right_type.to_string(),
                })
            }

            BinaryOp::BitAnd
            | BinaryOp::BitOr
            | BinaryOp::BitXor
            | BinaryOp::ShiftLeft
            | BinaryOp::ShiftRight => {
                if left_type.is_dynamic() || right_type.is_dynamic() {
                    return Ok(Type::Int);
                }

                if left_type == Type::Int && right_type == Type::Int {
                    return Ok(Type::Int);
                }

                Err(CompileError::InvalidBinaryOperation {
                    operator: binary_symbol(operator).to_string(),
                    left: left_type.to_string(),
                    right: right_type.to_string(),
                })
            }
        }
    }

    fn member_type(&self, object_type: &Type, name: &str) -> Result<Type, CompileError> {
        match object_type {
            Type::Named(class_name) => {
                self.check_member_visibility(class_name, name)?;

                let signatures = self.find_methods(class_name, name);

                if signatures.len() == 1 {
                    return Ok(Type::Function(signatures[0].clone()));
                }

                if signatures.len() > 1 {
                    // Une surcharge ne forme pas une valeur de fonction unique
                    // sans contexte d'appel. Les appels directs sont traités
                    // plus haut et sélectionnent la bonne signature.
                    return Ok(Type::Dynamic);
                }

                // Champ déclaré (`let age: int = 0;`) : son type est connu.
                if let Some(field_type) = self.find_field(class_name, name) {
                    return Ok(field_type);
                }
            }

            Type::Module(path) => {
                let context =
                    self.context
                        .as_ref()
                        .ok_or_else(|| CompileError::InvalidMemberAccess {
                            name: name.to_string(),
                        })?;
                let interface = context
                    .module_loader
                    .interface(std::path::Path::new(path))?;
                return interface.exports.get(name).cloned().ok_or_else(|| {
                    CompileError::InvalidMemberAccess {
                        name: name.to_string(),
                    }
                });
            }

            // Record : forme fixe, donc un champ inexistant est une erreur
            // certaine. Un champ l'emporte sur les méthodes d'introspection.
            Type::Record(fields) => {
                if let Some((_, field_type)) = fields.iter().find(|(field, _)| field == name) {
                    return Ok(field_type.clone());
                }

                return object_type.record_method_type(name).ok_or_else(|| {
                    CompileError::InvalidMemberAccess {
                        name: name.to_string(),
                    }
                });
            }

            Type::Set(_) | Type::SetDynamic => {
                if let Some(replacement) = object_type.renamed_member(name) {
                    return Err(CompileError::RenamedMember {
                        name: name.to_string(),
                        replacement: replacement.to_string(),
                    });
                }

                // Type connu : un membre inexistant est une erreur certaine.
                return object_type.set_member_type(name).ok_or_else(|| {
                    CompileError::InvalidMemberAccess {
                        name: name.to_string(),
                    }
                });
            }

            // Accès SANS appel (`a.length`) : seuls les noms supprimés sont
            // signalés. Les méthodes standard typées sont traitées à
            // l'appel (voir `Expression::Call`), car `d.size` peut être une
            // clé de dict.
            Type::Array(_)
            | Type::ArrayDynamic
            | Type::Dict(_, _)
            | Type::DictDynamic
            | Type::Tuple(_)
            | Type::TupleDynamic
            | Type::Str
            | Type::Range => {
                if let Some(replacement) = object_type.renamed_member(name) {
                    return Err(CompileError::RenamedMember {
                        name: name.to_string(),
                        replacement: replacement.to_string(),
                    });
                }
            }

            _ => {}
        }

        Ok(Type::Dynamic)
    }

    /// Signatures de la fonction SURCHARGÉE désignée par `name` (locale ou
    /// globale : la portée la plus proche qui définit `name` l'emporte, comme
    /// pour une variable). `None` si ce n'est pas une fonction surchargée.
    fn overloads_of(&self, name: &str) -> Option<Vec<FunctionType>> {
        for index in (0..self.scopes.len()).rev() {
            if self.scopes[index].contains_key(name) {
                let signatures = if index == 0 {
                    self.function_overloads.get(name)
                } else {
                    self.local_functions[index].get(name)
                };

                return signatures.filter(|signatures| signatures.len() > 1).cloned();
            }
        }

        None
    }

    /// `true` si `class_name` et toutes ses bases sont déclarées dans ce
    /// fichier (donc si l'absence de constructeur est certaine).
    fn hierarchy_is_known(&self, class_name: &str) -> bool {
        let mut pending = vec![class_name.to_string()];
        let mut visited = HashSet::new();

        while let Some(name) = pending.pop() {
            if !visited.insert(name.clone()) {
                continue;
            }

            match self.classes.get(&name) {
                Some(class) => pending.extend(class.bases.iter().cloned()),
                None => return false,
            }
        }

        true
    }

    /// Première classe de la hiérarchie de `class_name` qui déclare `member`
    /// (champ ou méthode), avec sa visibilité (`true` = privé).
    fn find_member_declaration(&self, class_name: &str, member: &str) -> Option<(String, bool)> {
        let mut pending = vec![class_name.to_string()];
        let mut visited = HashSet::new();

        while let Some(name) = pending.pop() {
            if !visited.insert(name.clone()) {
                continue;
            }

            if let Some(class) = self.classes.get(&name) {
                if class.fields.contains_key(member) || class.methods.contains_key(member) {
                    return Some((name, class.private_members.contains(member)));
                }

                pending.extend(class.bases.iter().cloned());
            }
        }

        None
    }

    /// Refuse `objet.membre` si `membre` est privé et que le code courant
    /// n'est pas dans le corps de la classe qui le déclare.
    ///
    /// Ne voit que les classes connues de ce fichier : pour une classe
    /// importée (ou une valeur dynamique), c'est la VM qui contrôle.
    fn check_member_visibility(&self, class_name: &str, member: &str) -> Result<(), CompileError> {
        match self.find_member_declaration(class_name, member) {
            Some((owner, true)) if self.current_class.as_deref() != Some(owner.as_str()) => {
                Err(CompileError::PrivateMemberAccess {
                    class_name: owner,
                    member: member.to_string(),
                })
            }

            _ => Ok(()),
        }
    }

    /// Type déclaré d'un champ, en remontant la hiérarchie.
    fn find_field(&self, class_name: &str, field: &str) -> Option<Type> {
        let mut pending = vec![class_name.to_string()];
        let mut visited = HashSet::new();

        while let Some(name) = pending.pop() {
            if !visited.insert(name.clone()) {
                continue;
            }

            if let Some(class) = self.classes.get(&name) {
                if let Some(ty) = class.fields.get(field) {
                    return Some(ty.clone());
                }

                pending.extend(class.bases.iter().cloned());
            }
        }

        None
    }

    fn find_methods(&self, class_name: &str, method_name: &str) -> Vec<FunctionType> {
        let mut pending = vec![class_name.to_string()];
        let mut visited = HashSet::new();
        let mut seen_arities = HashSet::new();
        let mut result = Vec::new();

        while let Some(name) = pending.pop() {
            if !visited.insert(name.clone()) {
                continue;
            }

            if let Some(class) = self.classes.get(&name) {
                if let Some(overloads) = class.methods.get(method_name) {
                    for signature in overloads {
                        // Une surcharge définie dans la classe dérivée masque
                        // la signature de même arité d'une classe de base.
                        if seen_arities.insert(signature.params.len()) {
                            result.push(signature.clone());
                        }
                    }
                }

                pending.extend(class.bases.iter().cloned());
            }
        }

        result
    }

    fn check_call_signature(
        &mut self,
        signature: &FunctionType,
        arguments: &[Expression],
        function_name: &str,
    ) -> Result<Type, CompileError> {
        if signature.params.len() != arguments.len() {
            return Err(CompileError::WrongArgumentCount {
                expected: signature.params.len() as i32,
                found: arguments.len(),
            });
        }

        for (index, (argument, expected)) in
            arguments.iter().zip(&signature.params).enumerate()
        {
            let actual = self.check_expression(argument)?;

            if !self.are_assignable(&actual, expected) {
                return Err(CompileError::WrongArgumentType {
                    function: function_name.to_string(),
                    index: index + 1,
                    expected: expected.to_string(),
                    found: actual.to_string(),
                });
            }
        }

        Ok((*signature.return_type).clone())
    }

    fn resolve_overload(
        &mut self,
        signatures: &[FunctionType],
        arguments: &[Expression],
        function_name: &str,
    ) -> Result<FunctionType, CompileError> {
        let signature = signatures
            .iter()
            .find(|signature| signature.params.len() == arguments.len());

        let Some(signature) = signature else {
            let expected = signatures
                .first()
                .map(|signature| signature.params.len() as i32)
                .unwrap_or(0);

            return Err(CompileError::WrongArgumentCount {
                expected,
                found: arguments.len(),
            });
        };

        self.check_call_signature(signature, arguments, function_name)?;

        Ok(signature.clone())
    }

    fn bind_pattern(&mut self, pattern: &Pattern, matched_type: &Type) -> Result<(), CompileError> {
        match pattern {
            Pattern::Wildcard | Pattern::Literal(_) | Pattern::Range { .. } => Ok(()),
            Pattern::Binding(name) => self.declare(
                name,
                Binding {
                    ty: matched_type.clone(),
                    _mutable: true,
                    native: false,
                },
            ),
            Pattern::Or(patterns) => {
                for pattern in patterns {
                    self.bind_pattern(pattern, matched_type)?;
                }
                Ok(())
            }
            Pattern::Array(patterns) => {
                let element_type = matched_type.element_type();
                for pattern in patterns {
                    self.bind_pattern(pattern, &element_type)?;
                }
                Ok(())
            }
        }
    }

    fn expression_name(&self, expression: &Expression) -> String {
        match expression {
            Expression::Variable(name) => name.clone(),
            Expression::Member { object, name, .. } => {
                let object = self.expression_name(object);
                format!("{object}.{name}")
            }
            _ => "<call>".to_string(),
        }
    }

    fn are_assignable(&self, actual: &Type, expected: &Type) -> bool {
        let parents = |name: &str| self.parents.get(name).cloned().unwrap_or_default();
        actual.is_assignable_to(expected, &parents)
    }

    fn ensure_assignable(&self, actual: &Type, expected: &Type) -> Result<(), CompileError> {
        if self.are_assignable(actual, expected) {
            Ok(())
        } else {
            Err(CompileError::TypeMismatch {
                expected: expected.to_string(),
                found: actual.to_string(),
            })
        }
    }

    fn declare(&mut self, name: &str, binding: Binding) -> Result<(), CompileError> {
        let is_global_scope = self.scopes.len() == 1;
        let existing_native = self
            .scopes
            .last()
            .and_then(|scope| scope.get(name))
            .is_some_and(|binding| binding.native);

        let scope = self
            .scopes
            .last_mut()
            .expect("TypeChecker scope is never empty");

        if scope.contains_key(name) {
            if is_global_scope && existing_native {
                scope.insert(
                    name.to_string(),
                    Binding {
                        native: false,
                        ..binding
                    },
                );
                return Ok(());
            }

            return Err(CompileError::VariableAlreadyDeclared(name.to_string()));
        }

        scope.insert(name.to_string(), binding);
        Ok(())
    }

    fn lookup(&self, name: &str) -> Option<Binding> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned())
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.local_functions.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        debug_assert!(self.scopes.len() > 1);
        self.scopes.pop();
        self.local_functions.pop();
    }

    fn with_location<T>(
        &mut self,
        line: usize,
        column: usize,
        f: impl FnOnce(&mut Self) -> Result<T, CompileError>,
    ) -> Result<T, CompileError> {
        f(&mut *self).map_err(|error| match error {
            CompileError::WithLocation { .. } => error,
            other => CompileError::WithLocation {
                line,
                column,
                source: Box::new(other),
            },
        })
    }
}

fn binary_symbol(operator: &BinaryOp) -> &'static str {
    match operator {
        BinaryOp::Add => "+",
        BinaryOp::Subtract => "-",
        BinaryOp::Multiply => "*",
        BinaryOp::Divide => "/",
        BinaryOp::Modulo => "%",
        BinaryOp::Equal => "==",
        BinaryOp::NotEqual => "!=",
        BinaryOp::Less => "<",
        BinaryOp::LessEqual => "<=",
        BinaryOp::Greater => ">",
        BinaryOp::GreaterEqual => ">=",
        BinaryOp::And => "and",
        BinaryOp::Or => "or",
        BinaryOp::Is => "is",
        BinaryOp::BitAnd => "&",
        BinaryOp::BitOr => "|",
        BinaryOp::BitXor => "^",
        BinaryOp::ShiftLeft => "<<",
        BinaryOp::ShiftRight => ">>",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::{lexer::lexer::Lexer, parser::Parser};

    fn check(source: &str) -> Result<(), CompileError> {
        let tokens = Lexer::new(source.to_string()).scan_token().unwrap();
        let statements = Parser::new(tokens).parse().unwrap();
        TypeChecker::check(&statements)
    }

    #[test]
    fn inference_and_annotations_are_accepted() {
        let result = check(
            r#"
let x = 10;
let y: int = 20;
let z = x + y;
let name: str = "Bruno";
let values: List<int> = [1, 2, 3];
let users: Dict<str, int> = {
    "age": 25
};
let p: { name: str, age: int } = { name: "Bruno", age: 25 };
func add(a: int, b: int) -> int {
    return a + b;
}
let result = add(10, 20);
"#,
        );
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn nested_generics_and_compact_equal_parse() {
        let result = check(
            "let m: Dict<str, List<int>> = { \"a\": [1, 2] };\nlet v: List<int>= [1, 2];",
        );
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn wrong_annotation_is_rejected() {
        assert!(check("let x: int = \"a\";").is_err());
    }

    #[test]
    fn division_with_dynamic_operand_is_float() {
        assert!(check("func f(n) { let q: int = n / 2; return q; }").is_err());
        assert!(check("func g(n) { let q: float = n / 2; return q; }").is_ok());
    }

    #[test]
    fn methods_and_constructors_can_be_overloaded_by_arity() {
        let result = check(
            r#"
class Point {
    func initialize() { this.x = 0; this.y = 0; }
    func initialize(x: int) { this.x = x; this.y = 0; }
    func initialize(x: int, y: int) { this.x = x; this.y = y; }
    func scale() -> int { return 1; }
    func scale(k: int) -> int { return k; }
}
let a = new Point();
let b = new Point(1);
let c = new Point(1, 2);
let k: int = c.scale();
let m: int = c.scale(3);
"#,
        );
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn same_arity_method_is_a_duplicate() {
        let result = check("class A { func f(x) { return 1; } func f(y) { return 2; } }");
        assert!(matches!(result, Err(CompileError::DuplicateMethod { .. })));
    }

    #[test]
    fn no_overload_for_the_given_arity_is_rejected() {
        assert!(
            check("class A { func f(x) { return 1; } } let a = new A(); a.f(1, 2);").is_err()
        );
        assert!(
            check("class B { func initialize(x) { this.x = x; } } let b = new B();").is_err()
        );
    }

    #[test]
    fn interfaces_can_declare_overloads() {
        let result = check(
            r#"
interface Shape {
    func area() -> float;
    func area(scale: float) -> float;
}
class Square: Shape {
    func area() -> float { return 1.0; }
    func area(scale: float) -> float { return scale; }
}
let s: Shape = new Square();
let a: float = s.area();
let b: float = s.area(2.0);
"#,
        );
        assert!(result.is_ok(), "{:?}", result.err());

        let duplicate = check("interface I { func f(); func f(); }");
        assert!(matches!(duplicate, Err(CompileError::DuplicateMethod { .. })));

        let wrong_call = check(
            r#"
interface Shape { func area() -> float; }
class Square: Shape { func area() -> float { return 1.0; } }
let s: Shape = new Square();
s.area(1.0, 2.0);
"#,
        );
        assert!(wrong_call.is_err());
    }

    #[test]
    fn tuple_annotations_are_checked_element_by_element() {
        let result = check(
            r#"
let t: Tuple<float, str> = (1, "a");
func first(p: Tuple<int, int>) -> int { return p[0]; }
let r: int = first((1, 2));
"#,
        );
        assert!(result.is_ok(), "{:?}", result.err());

        assert!(check("let bad: Tuple<int> = (1, 2);").is_err());
        assert!(check("let bad: Tuple<int, str> = (1.5, \"a\");").is_err());
    }

    fn parse_fails(source: &str) -> bool {
        let tokens = Lexer::new(source.to_string()).scan_token().unwrap();
        Parser::new(tokens).parse().is_err()
    }

    /// Les erreurs relevées pendant `check_statements` sont enveloppées dans
    /// `WithLocation` : on remonte à l'erreur d'origine.
    fn is_private_access(result: &Result<(), CompileError>) -> bool {
        let mut error = match result {
            Err(error) => error,
            Ok(()) => return false,
        };

        while let CompileError::WithLocation { source, .. } = error {
            error = &**source;
        }

        matches!(error, CompileError::PrivateMemberAccess { .. })
    }

    const PERSONNE: &str = r#"
class Personne {
    private let age: int = 0;

    public func initialize(age: int) {
        this.age = age;
    }

    public func setAge(age: int) {
        this.age = age;
    }

    public func getAge() -> int {
        return this.age;
    }

    public func number() -> int {
        return 22;
    }
}

let p: Personne = new Personne(26);
p.setAge(44);
let age: int = p.getAge();
"#;

    #[test]
    fn public_api_of_a_class_with_private_field_type_checks() {
        let result = check(PERSONNE);
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn private_field_is_forbidden_outside_the_class() {
        let read = check(&format!("{PERSONNE}\nprintln(p.age);"));
        assert!(is_private_access(&read), "{:?}", read);

        let write = check(&format!("{PERSONNE}\np.age = 3;"));
        assert!(is_private_access(&write), "{:?}", write);
    }

    #[test]
    fn private_method_is_forbidden_outside_the_class() {
        let result = check(
            r#"
class A {
    private func secret() -> int { return 1; }
    func open() -> int { return this.secret(); }
}
let a = new A();
let x: int = a.open();
a.secret();
"#,
        );
        assert!(is_private_access(&result), "{:?}", result);
    }

    #[test]
    fn private_members_are_usable_by_other_instances_and_callbacks() {
        let result = check(
            r#"
class A {
    private let n: int = 1;
    func same(other: A) -> int { return other.n; }
    func later() { let f = func() { return this.n; }; return f(); }
}
"#,
        );
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn declared_field_types_are_enforced() {
        assert!(check("class A { let n: int = 0; func f() { this.n = \"x\"; } }").is_err());
        assert!(check("class A { let n: int = \"x\"; }").is_err());
        assert!(check("class A { let n: float = 1; }").is_ok());
    }

    #[test]
    fn initialize_is_the_constructor_and_legacy_init_is_rejected() {
        assert!(
            check("class A { func initialize(x: int) { this.x = x; } } let a = new A(1);").is_ok()
        );
        assert!(
            check("class A { func initialize(x: int) { this.x = x; } } let a = new A();").is_err()
        );

        // L'ancien nom est refusé avec un message de migration.
        assert!(parse_fails("class A { func init(x) { this.x = x; } }"));
    }

    #[test]
    fn constructors_are_overloaded_by_arity() {
        let source = r#"
class Point {
    func initialize() { this.x = 0; }
    func initialize(x: int) { this.x = x; }
    func initialize(x: int, y: int) { this.x = x; this.y = y; }
}
"#;
        assert!(check(&format!("{source}\nlet a = new Point(); let b = new Point(1); let c = new Point(1, 2);")).is_ok());
        assert!(check(&format!("{source}\nlet d = new Point(1, 2, 3);")).is_err());
        assert!(check(&format!("{source}\nlet e = new Point(\"x\");")).is_err());

        let duplicate = check("class P { func initialize(a) {} func initialize(b) {} }");
        assert!(matches!(duplicate, Err(CompileError::DuplicateMethod { .. })));
    }

    #[test]
    fn class_without_constructor_uses_an_implicit_default_constructor() {
        assert!(check("class A { func f() { return 1; } } let a = new A();").is_ok());
        assert!(check("class A { func f() { return 1; } } let a = new A(1);").is_err());

        // Des champs avec valeur initiale ne changent rien : toujours le
        // constructeur par défaut.
        assert!(check("class B { let n: int = 0; } let b = new B();").is_ok());
        assert!(check("class B { let n: int = 0; } let b = new B(5);").is_err());
    }

    #[test]
    fn derived_class_inherits_base_constructors() {
        let base = "class A { func initialize(x: int) { this.x = x; } }\n";

        assert!(check(&format!("{base}class B: A {{ }}\nlet b = new B(1);")).is_ok());
        assert!(check(&format!("{base}class B: A {{ }}\nlet b = new B();")).is_err());
    }

    #[test]
    fn duplicate_fields_and_mixed_overload_visibility_are_parse_errors() {
        assert!(parse_fails("class A { let n = 1; let n = 2; }"));
        assert!(parse_fails(
            "class A { func f() { return 1; } private func f(x) { return 2; } }"
        ));
    }

    #[test]
    fn sets_are_typed_by_their_elements() {
        let ok = check(
            r#"
let s = Set(1, 2, 3);
s.add(4);
let n: int = s.size();
let has: bool = s.contains(2);
let empty: bool = s.is_empty();
s.remove(2);
s.clear();

let typed: Set<int> = Set(1, 2, 2);
let a = Set(1, 2, 3);
let b = Set(3, 4, 5);
let u: Set<int> = a.union(b);
let i: Set<int> = a.intersection(b);
let sub: bool = a.is_subset(b);
let c = a.copy();
c.add(9);
let list: List<int> = a.to_list();
let from_literal: Set<int> = {1, 2, 3};

for x in a {
    let doubled: int = x * 2;
}
"#,
        );
        assert!(ok.is_ok(), "{:?}", ok.err());

        // Le type d'élément est protégé à l'ajout.
        assert!(check("let s: Set<int> = Set(); s.add(\"a\");").is_err());
        assert!(check("let s = Set(1, 2); let n: str = s.size();").is_err());
        assert!(check("let s: Set<str> = Set(1, 2);").is_err());

        // Membre inexistant ou mauvaise arité.
        assert!(check("let s = Set(1); s.nope();").is_err());
        assert!(check("let s = Set(1); s.size(1);").is_err());
    }

    #[test]
    fn brace_literal_is_a_set_unless_it_starts_with_a_key() {
        let ok = check(
            r#"
let d: Dict<str, int> = { "age": 25 };
let quoted = { "k": 2 };
let empty = {};
let s: Set<str> = { "x", "y" };
let one: Set<int> = { 1 };
let trailing: Set<int> = { 1, 2, };
"#,
        );
        assert!(ok.is_ok(), "{:?}", ok.err());

        // `{1, 2}` n'est PAS un dict.
        assert!(check("let d: Dict<str, int> = { 1, 2 };").is_err());
    }

    #[test]
    fn standard_collection_api_is_typed() {
        let ok = check(
            r#"
let a = [1, 2, 3];
let n: int = a.size();
let e: bool = a.is_empty();
let c: bool = a.contains(2);
a.add(4);
a.remove(2);
let b = a.copy();
let shown: str = a.to_string();
a.clear();

let d = {"a": 10, "b": 20};
let m: int = d.size();
let has: bool = d.contains("a");
let v: int = d.get("a");
d.set("a", 11);
let ks: List<str> = d.keys();
let entries = d.entries();

let t = (1, 2, 3);
let f: int = t.first();
let l: int = t.size();
let converted: List<int> = t.to_list();

let s = "Hello";
let sz: int = s.size();
let empty: bool = s.is_empty();
let inside: bool = s.contains("ll");

for x in Set(1, 2) {
    let doubled: int = x * 2;
}
"#,
        );
        assert!(ok.is_ok(), "{:?}", ok.err());

        // Mauvaise arité ou mauvais type de retour.
        assert!(check("let a = [1]; a.size(1);").is_err());
        assert!(check("let a = [1]; let n: str = a.size();").is_err());
        assert!(check("let d = {\"a\": 1}; let k: int = d.keys();").is_err());
    }

    #[test]
    fn removed_collection_names_are_reported_with_their_replacement() {
        let renamed = |source: &str| -> bool {
            let mut error = match check(source) {
                Err(error) => error,
                Ok(()) => return false,
            };

            while let CompileError::WithLocation { source, .. } = error {
                error = *source;
            }

            matches!(error, CompileError::RenamedMember { .. })
        };

        assert!(renamed("let a = [1]; let n = a.length;"));
        assert!(renamed("let a = [1]; a.push(2);"));
        assert!(renamed("let t = (1, 2); let n = t.length;"));
        assert!(renamed("let s = \"x\"; let n = s.length;"));
        assert!(renamed("let s = Set(1); let n = s.length;"));
        assert!(renamed("let d = {\"a\": 1}; d.has(\"a\");"));
        assert!(renamed("let d = {\"a\": 1}; let e = d.items();"));
        assert!(renamed("let a = [1]; let i = a.to_iterator();"));
    }

    #[test]
    fn record_fields_named_like_methods_and_user_classes_are_not_affected() {
        // `size` et `length` sont ici de simples champs de record.
        let ok = check(
            r#"
let d = { size: 3, length: 4 };
let a: int = d.size;
let b: int = d.length;

class Pile {
    func size() -> int { return 1; }
    func add(x) { return x; }
}
let p = new Pile();
let n: int = p.size();
p.add(2);
"#,
        );
        assert!(ok.is_ok(), "{:?}", ok.err());
    }

    #[test]
    fn records_have_named_fixed_fields_and_structural_types() {
        let ok = check(
            r#"
type Person = { name: str, age: int };

let p: Person = { name: "Bruno", age: 25 };
let n: str = p.name;
let a: int = p.age;
p.age = 26;

func greet(who: Person) -> str {
    return who.name;
}
let g: str = greet(p);

let q = p.copy();
let keys: List<str> = p.keys();
let shown: str = p.to_string();

// Typage structurel : des champs EN PLUS sont permis.
let wider = { name: "Alice", age: 30, city: "Paris" };
let w: Person = wider;

let people: List<Person> = [{ name: "A", age: 1 }, { name: "B", age: 2 }];
"#,
        );
        assert!(ok.is_ok(), "{:?}", ok.err());

        let prelude = "type Person = { name: str, age: int };\n";

        // Champ manquant, mauvais type, champ inexistant.
        assert!(check(&format!("{prelude}let p: Person = {{ name: \"Bruno\" }};")).is_err());
        assert!(
            check(&format!("{prelude}let p: Person = {{ name: 1, age: 25 }};")).is_err()
        );
        assert!(
            check(&format!(
                "{prelude}let p: Person = {{ name: \"B\", age: 1 }}; p.age = \"x\";"
            ))
            .is_err()
        );
        assert!(
            check(&format!(
                "{prelude}let p: Person = {{ name: \"B\", age: 1 }}; let e = p.email;"
            ))
            .is_err()
        );
        assert!(
            check(&format!(
                "{prelude}let p: Person = {{ name: \"B\", age: 1 }}; p.email = \"x\";"
            ))
            .is_err()
        );

        // Un record n'est pas un dict, et inversement.
        assert!(check("let d: Dict<str, int> = { name: 1 };").is_err());
        assert!(check("let r: { name: int } = { \"name\": 1 };").is_err());
    }

    #[test]
    fn record_and_dict_literals_do_not_mix() {
        assert!(parse_fails("let x = { name: 1, \"age\": 2 };"));
        assert!(parse_fails("let x = { \"name\": 1, age: 2 };"));
        assert!(parse_fails("let x = { name: 1, name: 2 };"));
        assert!(parse_fails("let x = { \"a\": 1, \"a\": 2 };"));

        assert!(check("let x = { name: 1, age: 2 };").is_ok());
        assert!(check("let x = { \"name\": 1, \"age\": 2 };").is_ok());
    }

    #[test]
    fn type_aliases_can_be_used_before_their_declaration() {
        let ok = check(
            r#"
func double(x: Number) -> Number {
    return x * 2;
}

type Number = int | float;
type Names = List<str>;

let n: Number = double(2);
let names: Names = ["a", "b"];
"#,
        );
        assert!(ok.is_ok(), "{:?}", ok.err());

        // Alias cyclique : pas de boucle infinie.
        assert!(check("type A = A; let x: A = 1;").is_ok());

        // Deux alias de même nom.
        assert!(check("type A = int; type A = str;").is_err());

        // `type` reste une fonction ordinaire.
        assert!(check("let t = type(1);").is_ok());
    }

    #[test]
    fn union_types_accept_any_member_and_support_arithmetic() {
        let ok = check(
            r#"
type Number = int | float;

let a: Number = 1;
let b: Number = 2.5;
let c: Number = a + b;
let d: Number = a * 2;
let e: bool = a < b;
let neg: Number = -a;
let f: float = a;

let t: str | int = "a";
let u: str | int = 5;

func half(x: int | float) -> float {
    return x / 2;
}
"#,
        );
        assert!(ok.is_ok(), "{:?}", ok.err());

        // Valeur hors de l'union, ou union plus large que la cible.
        assert!(check("type Number = int | float; let s: Number = \"x\";").is_err());
        assert!(check("type Number = int | float; let a: Number = 1; let i: int = a;").is_err());
        assert!(check("let t: str | int = 2.5;").is_err());
    }

    #[test]
    fn array_was_renamed_list() {
        assert!(check("let a: List<int> = [1, 2];").is_ok());
        assert!(parse_fails("let a: Array<int> = [1, 2];"));
    }

    #[test]
    fn global_functions_can_be_overloaded_by_arity() {
        let ok = check(
            r#"
func area() -> int { return 0; }
func area(a: int) -> int { return a; }
func area(a: int, b: int) -> int { return a * b; }

let x: int = area();
let y: int = area(3);
let z: int = area(2, 5);

// Pris comme valeur, le nom reste dynamique.
let f = area;

// Une variable locale de même nom masque la fonction.
func shadow() {
    let area = 5;
    return area;
}
"#,
        );
        assert!(ok.is_ok(), "{:?}", ok.err());

        // Aucune surcharge à 3 arguments, ou mauvais type d'argument.
        assert!(check("func f(a) {} func f(a, b) {} f(1, 2, 3);").is_err());
        assert!(check("func f(a: int) {} func f(a: int, b: int) {} f(\"x\");").is_err());

        // Même arité = doublon.
        let duplicate = check("func f(a) {} func f(b) {}");
        assert!(matches!(duplicate, Err(CompileError::DuplicateFunction { .. })));

        // Le type de retour dépend de la surcharge choisie.
        assert!(
            check("func g(a: int) -> int { return a; } func g(a: str) -> str { return a; }")
                .is_err()
        );
        assert!(
            check(
                "func g(a: int) -> int { return a; } func g(a: int, b: int) -> str { return \"s\"; } let s: str = g(1, 2); let n: int = g(1);"
            )
            .is_ok()
        );
        assert!(
            check(
                "func g(a: int) -> int { return a; } func g(a: int, b: int) -> str { return \"s\"; } let n: int = g(1, 2);"
            )
            .is_err()
        );
    }

    #[test]
    fn private_constructors_are_refused_statically_outside_the_class() {
        assert!(
            check("class S { private func initialize() { this.v = 1; } } let s = new S();")
                .is_err()
        );

        // Public : aucun problème ; `base.initialize()` reste permis.
        assert!(
            check("class S { func initialize() { this.v = 1; } } let s = new S();").is_ok()
        );
    }

    #[test]
    fn an_exported_alias_resolves_in_a_function_declared_after_it_in_the_same_file() {
        // Régression : `register_aliases` ne déballait pas l'enveloppe
        // `Export`, donc un alias EXPORTÉ n'était jamais enregistré avant que
        // `collect_declarations` calcule les signatures de fonctions du même
        // fichier — `origin()` était alors typé `Point` (non résolu) au lieu
        // de `{ x: int, y: int }`, même si `Point` est déclaré AVANT
        // `origin` dans le fichier.
        let ok = check(
            r#"
export type Point = { x: int, y: int };
export type Number = int | float;

export func origin() -> Point {
    return { x: 0, y: 0 };
}

let p: Point = origin();
let x: int = p.x;

func half(n: Number) -> float {
    return n / 2;
}
let h: float = half(4);
"#,
        );
        assert!(ok.is_ok(), "{:?}", ok.err());
    }

    #[test]
    fn an_exported_alias_resolves_in_a_function_declared_before_it() {
        // Sens inverse : la fonction précède l'alias qu'elle utilise dans le
        // même fichier (référence en avant), couvert par `register_aliases`
        // (première passe, avant `collect_declarations`).
        let ok = check(
            r#"
export func origin() -> Point {
    return { x: 0, y: 0 };
}

export type Point = { x: int, y: int };

let p: Point = origin();
let x: int = p.x;
"#,
        );
        assert!(ok.is_ok(), "{:?}", ok.err());
    }
}

