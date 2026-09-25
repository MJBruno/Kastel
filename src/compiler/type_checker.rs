use std::collections::{HashMap, HashSet};

use crate::error::compile_error::CompileError;
use crate::frontend::ast::*;

use super::{
    builtin_types,
    capability::Capability,
    compiler::MAX_EXPRESSION_DEPTH,
    module_types::{ImportedType, ModuleTypeInterface, ModuleTypeLoader},
    types::{FunctionType, GenericConstraint, Type},
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

#[derive(Debug, Clone)]
struct LocalTypeAlias {
    generic_params: Vec<String>,
    body: TypeExpr,
}

#[derive(Debug, Clone)]
pub(crate) struct TypeAliasInfo {
    pub(crate) generic_params: Vec<String>,
    pub(crate) ty: Type,
}

/// Ce que le vérificateur sait d'une classe (ou interface) : bases,
/// méthodes surchargées, champs typés, membres privés.
///
/// Exporté avec l'INTERFACE de types d'un module : une classe importée est
/// ainsi vérifiée comme une classe locale (arité des constructeurs,
/// surcharges, champs, visibilité `protected`/`private`).
#[derive(Debug, Clone)]
pub(crate) struct ClassInfo {
    generic_params: Vec<String>,
    generic_constraints: Vec<(String, Vec<GenericConstraint>)>,
    /// `true` pour une déclaration `interface`, `false` pour une classe/enum.
    /// Cela empêche une contrainte générique de cibler arbitrairement une
    /// classe concrète comme s'il s'agissait d'un contrat.
    is_interface: bool,
    bases: Vec<String>,
    base_types: Vec<Type>,
    /// Une classe ou un enum peut surcharger une méthode par son arité.
    /// Deux signatures de même nom et de même arité restent interdites.
    methods: HashMap<String, Vec<FunctionType>>,
    /// Champs déclarés par `let nom: type = ...;` dans le corps de la classe.
    fields: HashMap<String, Type>,
    /// `static func ...` : appelées sur la CLASSE (`NomClasse.membre`), pas
    /// sur une instance. Séparées de `methods` car elles n'ont pas `this` et
    /// ne sont pas héritées ; voir `find_static_methods`, consultée en
    /// complément de `find_methods_for_type` lorsqu'aucune méthode
    /// d'instance ne correspond.
    static_methods: HashMap<String, Vec<FunctionType>>,
    /// `static let ...` : équivalent statique de `fields`.
    static_fields: HashMap<String, Type>,
    /// Membres (champs et méthodes, statiques ou non) déclarés `private`
    /// dans cette classe.
    private_members: HashSet<String>,
    /// Membres (champs et méthodes, statiques ou non) déclarés `protected`
    /// dans cette classe.
    protected_members: HashSet<String>,
    /// Variants nommés d'un enum. Vide pour les classes/interfaces.
    enum_variants: HashSet<String>,
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
    aliases: HashMap<String, LocalTypeAlias>,

    /// Interfaces connues avant la collecte des signatures. Cette pré-passe
    /// rend `T: Foo` indépendant de l'ordre des déclarations du fichier.
    known_interfaces: HashSet<String>,

    /// Alias de type IMPORTÉS d'un autre module (`from m import Person;`,
    /// `import m.Person;`), déjà résolus par le module qui les exporte (voir
    /// `ModuleTypeInterface::type_aliases`) : autonomes, sans référence aux
    /// noms locaux de ce module-ci.
    imported_type_aliases: HashMap<String, TypeAliasInfo>,

    /// Alias de noms de classes importées (`from m import Box as B`).
    class_aliases: HashMap<String, String>,

    /// Paramètres génériques actifs pendant le contrôle d'un corps.
    generic_params: Vec<String>,

    /// Contraintes/capabilities des paramètres génériques actifs.
    generic_constraints: HashMap<String, Vec<GenericConstraint>>,

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
        (
            HashMap<String, Type>,
            HashMap<String, ClassInfo>,
            HashMap<String, TypeAliasInfo>,
        ),
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
            if let Statement::TypeAlias {
                name,
                generic_params,
                type_expr,
            } = inner
            {
                let generic_names = generic_params.iter().map(|p| p.name.clone()).collect::<Vec<_>>();
                let resolved = checker.resolve_type_with_generic_names(type_expr, &generic_names);

                if type_aliases
                    .insert(
                        name.clone(),
                        TypeAliasInfo {
                            generic_params: generic_names,
                            ty: resolved,
                        },
                    )
                    .is_some()
                {
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
            known_interfaces: HashSet::new(),
            imported_type_aliases: HashMap::new(),
            class_aliases: HashMap::new(),
            generic_params: Vec::new(),
            generic_constraints: HashMap::new(),
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
        self.register_interfaces(statements);
        self.register_imported_types(statements)?;
        self.collect_declarations(statements)?;
        self.validate_interface_implementations()
    }

    fn register_aliases(&mut self, statements: &[Statement]) -> Result<(), CompileError> {
        for statement in statements {
            // `export type X = ...;` doit être reconnu au même titre qu'un
            // alias non exporté : l'export ne change que sa visibilité pour
            // les AUTRES modules, pas sa disponibilité dans CE fichier. Sans
            // ce déballage, une signature de fonction du même fichier qui
            // utilise un alias EXPORTÉ (même déclaré plus haut) le voyait
            // comme un type nommé non résolu au lieu de sa forme réelle.
            if let Statement::TypeAlias {
                name,
                generic_params,
                type_expr,
            } = Self::strip_position_and_export(statement)
            {
                if self.aliases.contains_key(name) {
                    return Err(CompileError::VariableAlreadyDeclared(name.clone()));
                }

                self.aliases.insert(
                    name.clone(),
                    LocalTypeAlias {
                        generic_params: generic_params.iter().map(|p| p.name.clone()).collect(),
                        body: type_expr.clone(),
                    },
                );
            }
        }

        Ok(())
    }

    fn register_interfaces(&mut self, statements: &[Statement]) {
        for statement in statements {
            let inner = Self::strip_position_and_export(statement);
            if let Statement::Interface { name, .. } = inner {
                self.known_interfaces.insert(name.clone());
            }
        }
    }

    fn register_imported_types(&mut self, statements: &[Statement]) -> Result<(), CompileError> {
        let Some(context) = self.context.as_ref() else {
            return Ok(());
        };
        let current_module = context.current_module.clone();
        let module_loader = Rc::clone(&context.module_loader);

        for statement in statements {
            match Self::strip_position(statement) {
                Statement::Import { path } => {
                    if let ImportedType::Export {
                        ty: _,
                        name,
                        interface,
                    } = module_loader.resolve_import(&current_module, path)?
                    {
                        self.import_class_info(&interface, &name);
                        if interface
                            .classes
                            .get(&name)
                            .is_some_and(|info| info.is_interface)
                        {
                            self.known_interfaces.insert(name.clone());
                        }
                    }
                }

                Statement::FromImport { module, items } => {
                    let module_path = module_loader
                        .resolver()
                        .resolve(&current_module, &module.parts)?;
                    let interface = module_loader.interface(&module_path)?;

                    if items.len() == 1 && items[0].name == "*" {
                        let names = interface.classes.keys().cloned().collect::<Vec<_>>();
                        for name in names {
                            self.import_class_info(&interface, &name);
                        }
                    } else {
                        for item in items {
                            if let Some(info) = interface.classes.get(&item.name).cloned() {
                                let canonical_name = item.name.clone();
                                self.import_class_info(&interface, &canonical_name);
                                if let Some(alias) = item
                                    .alias
                                    .as_deref()
                                    .filter(|alias| *alias != item.name.as_str())
                                {
                                    self.class_aliases
                                        .insert(alias.to_string(), canonical_name.clone());
                                }
                                if info.is_interface {
                                    self.known_interfaces.insert(canonical_name);
                                }
                            }
                        }
                    }
                }

                _ => {}
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
    fn active_generic_environment(&self) -> HashMap<String, Type> {
        self.generic_params
            .iter()
            .cloned()
            .map(|name| (name.clone(), Type::TypeParam(name)))
            .collect()
    }

    fn resolve_type(&self, expr: &TypeExpr) -> Type {
        let environment = self.active_generic_environment();
        self.resolve_type_in_environment(expr, &environment, 0)
    }

    fn resolve_type_with_generic_names(&self, expr: &TypeExpr, names: &[String]) -> Type {
        let environment = names
            .iter()
            .cloned()
            .map(|name| (name.clone(), Type::TypeParam(name)))
            .collect::<HashMap<_, _>>();
        self.resolve_type_in_environment(expr, &environment, 0)
    }

    fn resolve_type_in_environment(
        &self,
        expr: &TypeExpr,
        environment: &HashMap<String, Type>,
        depth: usize,
    ) -> Type {
        if depth > 32 {
            return Type::Dynamic;
        }

        match expr {
            TypeExpr::Named(name) => {
                if let Some(ty) = environment.get(name) {
                    return ty.clone();
                }

                if let Some(target) = self.class_aliases.get(name) {
                    return Type::Named(target.clone());
                }

                if let Some(alias) = self.aliases.get(name) {
                    if alias.generic_params.is_empty() {
                        return self.resolve_type_in_environment(
                            &alias.body,
                            environment,
                            depth + 1,
                        );
                    }
                }

                if let Some(alias) = self.imported_type_aliases.get(name) {
                    if alias.generic_params.is_empty() {
                        return alias.ty.clone();
                    }
                }

                Type::from_type_expr(expr)
            }

            TypeExpr::Generic { name, arguments } => {
                let resolved_arguments = arguments
                    .iter()
                    .map(|argument| {
                        self.resolve_type_in_environment(argument, environment, depth + 1)
                    })
                    .collect::<Vec<_>>();

                if let Some(target) = self.class_aliases.get(name) {
                    return Type::Generic {
                        name: target.clone(),
                        arguments: resolved_arguments,
                    };
                }

                if let Some(alias) = self.aliases.get(name) {
                    if alias.generic_params.len() == resolved_arguments.len() {
                        let alias_environment = alias
                            .generic_params
                            .iter()
                            .cloned()
                            .map(|parameter| {
                                let ty = Type::TypeParam(parameter.clone());
                                (parameter, ty)
                            })
                            .collect::<HashMap<_, _>>();
                        let resolved = self.resolve_type_in_environment(
                            &alias.body,
                            &alias_environment,
                            depth + 1,
                        );
                        let substitutions = alias
                            .generic_params
                            .iter()
                            .cloned()
                            .zip(resolved_arguments.iter().cloned())
                            .collect::<HashMap<_, _>>();
                        return Self::substitute_type(&resolved, &substitutions);
                    }
                }

                if let Some(alias) = self.imported_type_aliases.get(name) {
                    if alias.generic_params.len() == resolved_arguments.len() {
                        let substitutions = alias
                            .generic_params
                            .iter()
                            .cloned()
                            .zip(resolved_arguments.iter().cloned())
                            .collect::<HashMap<_, _>>();
                        return Self::substitute_type(&alias.ty, &substitutions);
                    }
                }

                Type::build_generic(name, resolved_arguments)
            }

            TypeExpr::Union(members) => Type::union_of(
                members
                    .iter()
                    .map(|member| {
                        self.resolve_type_in_environment(member, environment, depth + 1)
                    })
                    .collect(),
            ),

            TypeExpr::Record(fields) => Type::Record(
                fields
                    .iter()
                    .map(|(name, field)| {
                        (
                            name.clone(),
                            self.resolve_type_in_environment(field, environment, depth + 1),
                        )
                    })
                    .collect(),
            ),
        }
    }

    fn substitute_type(ty: &Type, substitutions: &HashMap<String, Type>) -> Type {
        match ty {
            Type::TypeParam(name) => substitutions
                .get(name)
                .cloned()
                .unwrap_or_else(|| ty.clone()),
            Type::Array(element) => {
                Type::Array(Box::new(Self::substitute_type(element, substitutions)))
            }
            Type::Dict(key, value) => Type::Dict(
                Box::new(Self::substitute_type(key, substitutions)),
                Box::new(Self::substitute_type(value, substitutions)),
            ),
            Type::Tuple(elements) => Type::Tuple(
                elements
                    .iter()
                    .map(|element| Self::substitute_type(element, substitutions))
                    .collect(),
            ),
            Type::Record(fields) => Type::Record(
                fields
                    .iter()
                    .map(|(name, field)| {
                        (name.clone(), Self::substitute_type(field, substitutions))
                    })
                    .collect(),
            ),
            Type::Union(members) => Type::union_of(
                members
                    .iter()
                    .map(|member| Self::substitute_type(member, substitutions))
                    .collect(),
            ),
            Type::Function(function) => Type::Function(FunctionType {
                generic_params: function.generic_params.clone(),
                generic_constraints: function.generic_constraints.clone(),
                params: function
                    .params
                    .iter()
                    .map(|param| Self::substitute_type(param, substitutions))
                    .collect(),
                return_type: Box::new(Self::substitute_type(
                    &function.return_type,
                    substitutions,
                )),
            }),
            Type::Overloads(overloads) => Type::Overloads(
                overloads
                    .iter()
                    .map(|signature| {
                        FunctionType {
                            generic_params: signature.generic_params.clone(),
                            generic_constraints: signature.generic_constraints.clone(),
                            params: signature
                                .params
                                .iter()
                                .map(|param| Self::substitute_type(param, substitutions))
                                .collect(),
                            return_type: Box::new(Self::substitute_type(
                                &signature.return_type,
                                substitutions,
                            )),
                        }
                    })
                    .collect(),
            ),
            Type::Set(element) => Type::Set(Box::new(Self::substitute_type(element, substitutions))),
            Type::Generic { name, arguments } => Type::Generic {
                name: name.clone(),
                arguments: arguments
                    .iter()
                    .map(|argument| Self::substitute_type(argument, substitutions))
                    .collect(),
            },
            other => other.clone(),
        }
    }

    fn nominal_type_name(type_expr: &TypeExpr) -> Option<String> {
        match type_expr {
            TypeExpr::Named(name) => Some(name.clone()),
            TypeExpr::Generic { name, .. } => Some(name.clone()),
            _ => None,
        }
    }

    fn validate_generic_declaration(
        params: &[GenericParam],
    ) -> Result<Vec<String>, CompileError> {
        let mut names = Vec::with_capacity(params.len());
        for parameter in params {
            if names.contains(&parameter.name) {
                return Err(CompileError::VariableAlreadyDeclared(parameter.name.clone()));
            }
            names.push(parameter.name.clone());
        }
        Ok(names)
    }

    fn generic_constraints(
        &self,
        params: &[GenericParam],
    ) -> Result<Vec<(String, Vec<GenericConstraint>)>, CompileError> {
        let mut result = Vec::with_capacity(params.len());

        for parameter in params {
            let mut constraints = Vec::with_capacity(parameter.bounds.len());

            for bound in &parameter.bounds {
                let constraint = match bound {
                    TypeExpr::Named(name) => {
                        if let Some(capability) = Capability::from_name(name) {
                            GenericConstraint::Capability(capability)
                        } else {
                            let interface = self.resolve_type(bound);
                            if !self.is_interface_type(&interface) {
                                return Err(CompileError::InvalidGenericConstraint {
                                    parameter: parameter.name.clone(),
                                    constraint: format!("{bound:?}"),
                                });
                            }
                            GenericConstraint::Interface(Box::new(interface))
                        }
                    }
                    _ => {
                        let interface = self.resolve_type(bound);
                        if !self.is_interface_type(&interface) {
                            return Err(CompileError::InvalidGenericConstraint {
                                parameter: parameter.name.clone(),
                                constraint: format!("{bound:?}"),
                            });
                        }
                        GenericConstraint::Interface(Box::new(interface))
                    }
                };

                if !constraints.contains(&constraint) {
                    constraints.push(constraint);
                }
            }

            result.push((parameter.name.clone(), constraints));
        }

        Ok(result)
    }

    fn is_interface_type(&self, ty: &Type) -> bool {
        let (Type::Named(name) | Type::Generic { name, .. }) = ty else {
            return false;
        };

        self.known_interfaces.contains(name)
            || self
                .classes
                .get(name)
                .is_some_and(|info| info.is_interface)
    }

    fn generic_constraints_map(
        &self,
        params: &[GenericParam],
    ) -> Result<HashMap<String, Vec<GenericConstraint>>, CompileError> {
        Ok(self.generic_constraints(params)?.into_iter().collect())
    }

    fn constraints_imply(&self, active: &GenericConstraint, required: &GenericConstraint) -> bool {
        match (active, required) {
            (GenericConstraint::Capability(active), GenericConstraint::Capability(required)) => {
                active == required
            }
            (
                GenericConstraint::Interface(active),
                GenericConstraint::Interface(required),
            ) => self.are_assignable(active, required),
            _ => false,
        }
    }

    fn type_supports_capability(&self, ty: &Type, capability: Capability) -> bool {
        match ty {
            Type::Dynamic => true,
            Type::TypeParam(name) => self
                .generic_constraints
                .get(name)
                .is_some_and(|constraints| {
                    constraints.iter().any(|constraint| {
                        self.constraints_imply(
                            constraint,
                            &GenericConstraint::Capability(capability),
                        )
                    })
                }),
            Type::Union(members) => members
                .iter()
                .all(|member| self.type_supports_capability(member, capability)),
            _ => capability.is_satisfied_by(ty),
        }
    }

    fn type_satisfies_constraint(&self, ty: &Type, constraint: &GenericConstraint) -> bool {
        match ty {
            Type::Dynamic => true,
            Type::TypeParam(name) => self
                .generic_constraints
                .get(name)
                .is_some_and(|constraints| {
                    constraints
                        .iter()
                        .any(|active| self.constraints_imply(active, constraint))
                }),
            Type::Union(members) => members
                .iter()
                .all(|member| self.type_satisfies_constraint(member, constraint)),
            _ => match constraint {
                GenericConstraint::Capability(capability) => capability.is_satisfied_by(ty),
                GenericConstraint::Interface(interface) => self.are_assignable(ty, interface),
            },
        }
    }

    fn validate_constraint_set(
        &self,
        constraints: &[(String, Vec<GenericConstraint>)],
        substitutions: &HashMap<String, Type>,
        subject_name: &str,
    ) -> Result<(), CompileError> {
        for (parameter, constraints) in constraints {
            let Some(actual) = substitutions.get(parameter) else {
                continue;
            };

            for constraint in constraints {
                if !self.type_satisfies_constraint(actual, constraint) {
                    return Err(CompileError::GenericConstraintNotSatisfied {
                        parameter: parameter.clone(),
                        constraint: constraint.name(),
                        found: actual.to_string(),
                        function: subject_name.to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    fn validate_generic_constraints(
        &self,
        signature: &FunctionType,
        substitutions: &HashMap<String, Type>,
        function_name: &str,
    ) -> Result<(), CompileError> {
        self.validate_constraint_set(
            &signature.generic_constraints,
            substitutions,
            function_name,
        )
    }

    fn validate_nested_generic_declaration(
        outer: &[String],
        params: &[GenericParam],
    ) -> Result<Vec<String>, CompileError> {
        let names = Self::validate_generic_declaration(params)?;
        if let Some(name) = names.iter().find(|name| outer.iter().any(|outer_name| outer_name == *name)) {
            return Err(CompileError::VariableAlreadyDeclared(name.clone()));
        }
        Ok(names)
    }

    fn validate_interface_implementations(&self) -> Result<(), CompileError> {
        for (class_name, class) in &self.classes {
            if class.is_interface || class.enum_variants.len() > 0 {
                continue;
            }

            for base in &class.base_types {
                if self.is_interface_type(base) {
                    self.validate_interface_implementation(class_name, base)?;
                }
            }
        }

        Ok(())
    }

    fn validate_interface_implementation(
        &self,
        class_name: &str,
        interface_type: &Type,
    ) -> Result<(), CompileError> {
        let interface_name = Self::type_name(interface_type).unwrap_or_else(|| interface_type.to_string());
        let requirements = self.interface_requirements(interface_type);

        for (method_name, required) in requirements {
            let implemented = self.concrete_methods_for_type(
                &Type::Named(class_name.to_string()),
                &method_name,
            );

            let matches = implemented
                .iter()
                .any(|candidate| self.function_signature_implements_interface(candidate, &required));

            if !matches {
                return Err(CompileError::MissingInterfaceMethod {
                    class_name: class_name.to_string(),
                    interface: interface_name.clone(),
                    method: method_name,
                    arity: required.params.len(),
                });
            }
        }

        Ok(())
    }

    fn function_signature_implements_interface(
        &self,
        candidate: &FunctionType,
        required: &FunctionType,
    ) -> bool {
        candidate.generic_params.len() == required.generic_params.len()
            && candidate.generic_constraints == required.generic_constraints
            && candidate.params.len() == required.params.len()
            && candidate
                .params
                .iter()
                .zip(&required.params)
                .all(|(actual, expected)| self.are_assignable(expected, actual))
            && self.are_assignable(&candidate.return_type, &required.return_type)
    }

    fn interface_requirements(&self, interface_type: &Type) -> Vec<(String, FunctionType)> {
        let mut pending = vec![interface_type.clone()];
        let mut visited = HashSet::new();
        let mut seen_methods = HashSet::new();
        let mut result = Vec::new();

        while let Some(current) = pending.pop() {
            let Some(name) = Self::type_name(&current) else {
                continue;
            };
            let key = current.to_string();
            if !visited.insert(key) {
                continue;
            }

            let Some(interface) = self.classes.get(&name) else {
                continue;
            };
            if !interface.is_interface {
                continue;
            }

            let substitutions = interface
                .generic_params
                .iter()
                .cloned()
                .zip(Self::type_arguments(&current))
                .collect::<HashMap<_, _>>();

            for (method_name, overloads) in &interface.methods {
                for signature in overloads {
                    let instantiated = Self::substitute_function_signature(signature, &substitutions);
                    let key = format!("{method_name}/{}", instantiated.params.len());
                    if seen_methods.insert(key) {
                        result.push((method_name.clone(), instantiated));
                    }
                }
            }

            for base in &interface.base_types {
                pending.push(Self::substitute_type(base, &substitutions));
            }
        }

        result
    }

    fn concrete_methods_for_type(&self, object_type: &Type, method_name: &str) -> Vec<FunctionType> {
        let mut pending = vec![object_type.clone()];
        let mut visited = HashSet::new();
        let mut seen_arities = HashSet::new();
        let mut result = Vec::new();

        while let Some(current) = pending.pop() {
            let Some(name) = Self::type_name(&current) else {
                continue;
            };
            let key = current.to_string();
            if !visited.insert(key) {
                continue;
            }

            let Some(class) = self.classes.get(&name) else {
                continue;
            };
            if class.is_interface {
                continue;
            }

            let substitutions = class
                .generic_params
                .iter()
                .cloned()
                .zip(Self::type_arguments(&current))
                .collect::<HashMap<_, _>>();

            if let Some(overloads) = class.methods.get(method_name) {
                for signature in overloads {
                    if seen_arities.insert(signature.params.len()) {
                        result.push(Self::substitute_function_signature(signature, &substitutions));
                    }
                }
            }

            for base in &class.base_types {
                pending.push(Self::substitute_type(base, &substitutions));
            }
        }

        result
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
                    generic_params,
                    params,
                    param_types,
                    return_type,
                    ..
                } => {
                    let generic_names = Self::validate_generic_declaration(generic_params)?;
                    let previous_generics = std::mem::replace(&mut self.generic_params, generic_names.clone());

                    let parameters = params
                        .iter()
                        .enumerate()
                        .map(|(index, _)| {
                            param_types
                                .get(index)
                                .and_then(|annotation| annotation.as_ref())
                                .map_or(Type::Dynamic, |annotation| self.resolve_type(annotation))
                        })
                        .collect::<Vec<_>>();

                    let return_type = return_type
                        .as_ref()
                        .map(|annotation| self.resolve_type(annotation))
                        .unwrap_or(Type::Dynamic);

                    let generic_constraints = self.generic_constraints(generic_params)?;
                    self.generic_params = previous_generics;

                    let signature = FunctionType {
                        generic_params: generic_names,
                        generic_constraints,
                        params: parameters,
                        return_type: Box::new(return_type),
                    };

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
                    generic_params,
                    bases,
                    fields,
                    methods,
                } => {
                    let class_generic_names = Self::validate_generic_declaration(generic_params)?;
                    let previous_generics = std::mem::replace(
                        &mut self.generic_params,
                        class_generic_names.clone(),
                    );
                    let class_generic_constraints = self.generic_constraints(generic_params)?;

                    let base_types = bases
                        .iter()
                        .map(|base| self.resolve_type(base))
                        .collect::<Vec<_>>();
                    let base_names = bases
                        .iter()
                        .filter_map(Self::nominal_type_name)
                        .collect::<Vec<_>>();

                    let mut method_map: HashMap<String, Vec<FunctionType>> = HashMap::new();
                    let mut field_map: HashMap<String, Type> = HashMap::new();
                    let mut static_method_map: HashMap<String, Vec<FunctionType>> = HashMap::new();
                    let mut static_field_map: HashMap<String, Type> = HashMap::new();
                    let mut private_members: HashSet<String> = HashSet::new();
                    let mut protected_members: HashSet<String> = HashSet::new();

                    for field in fields.iter() {
                        let resolved = field
                            .type_annotation
                            .as_ref()
                            .map(|annotation| self.resolve_type(annotation))
                            .unwrap_or(Type::Dynamic);

                        if field.is_static {
                            static_field_map.insert(field.name.clone(), resolved);
                        } else {
                            field_map.insert(field.name.clone(), resolved);
                        }

                        match field.visibility {
                            Visibility::Private => {
                                private_members.insert(field.name.clone());
                            }
                            Visibility::Protected => {
                                protected_members.insert(field.name.clone());
                            }
                            Visibility::Public => {}
                        }
                    }

                    for method in methods.iter() {
                        let method_generic_names = Self::validate_nested_generic_declaration(
                            &class_generic_names,
                            &method.generic_params,
                        )?;
                        let mut method_environment_names = class_generic_names.clone();
                        method_environment_names.extend(method_generic_names.iter().cloned());
                        let previous = std::mem::replace(&mut self.generic_params, method_environment_names);

                        let params = method
                            .params
                            .iter()
                            .enumerate()
                            .map(|(index, _)| {
                                method
                                    .param_types
                                    .get(index)
                                    .and_then(|annotation| annotation.as_ref())
                                    .map_or(Type::Dynamic, |annotation| self.resolve_type(annotation))
                            })
                            .collect::<Vec<_>>();

                        let return_type = method
                            .return_type
                            .as_ref()
                            .map(|annotation| self.resolve_type(annotation))
                            .unwrap_or(Type::Dynamic);

                        let generic_constraints = self.generic_constraints(&method.generic_params)?;
                        self.generic_params = previous;

                        let signature = FunctionType {
                            generic_params: method_generic_names,
                            generic_constraints,
                            params,
                            return_type: Box::new(return_type),
                        };

                        // Statique et instance occupent des espaces de noms
                        // SÉPARÉS (comme en TypeScript/PHP) : `foo` peut donc
                        // être à la fois une méthode d'instance et une
                        // méthode statique sans collision d'arité entre elles.
                        let overloads = if method.is_static {
                            static_method_map.entry(method.name.clone()).or_default()
                        } else {
                            method_map.entry(method.name.clone()).or_default()
                        };

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

                        match method.visibility {
                            Visibility::Private => {
                                private_members.insert(method.name.clone());
                            }
                            Visibility::Protected => {
                                protected_members.insert(method.name.clone());
                            }
                            Visibility::Public => {}
                        }
                    }

                    self.generic_params = previous_generics;
                    self.parents.insert(name.clone(), base_names.clone());
                    self.classes.insert(
                        name.clone(),
                        ClassInfo {
                            generic_params: class_generic_names,
                            generic_constraints: class_generic_constraints,
                            is_interface: false,
                            bases: base_names,
                            base_types,
                            methods: method_map,
                            fields: field_map,
                            static_methods: static_method_map,
                            static_fields: static_field_map,
                            private_members,
                            protected_members,
                            enum_variants: HashSet::new(),
                        },
                    );
                    self.declare_global_declaration(name, Type::Named(name.clone()))?;
                }

                Statement::Enum {
                    name,
                    generic_params,
                    variants,
                    methods,
                } => {
                    let enum_generic_names = Self::validate_generic_declaration(generic_params)?;
                    let previous_generics = std::mem::replace(
                        &mut self.generic_params,
                        enum_generic_names.clone(),
                    );
                    let enum_generic_constraints = self.generic_constraints(generic_params)?;
                    let mut method_map: HashMap<String, Vec<FunctionType>> = HashMap::new();

                    for method in methods {
                        let method_generic_names = Self::validate_nested_generic_declaration(
                            &enum_generic_names,
                            &method.generic_params,
                        )?;
                        let mut method_environment_names = enum_generic_names.clone();
                        method_environment_names.extend(method_generic_names.iter().cloned());
                        let previous = std::mem::replace(&mut self.generic_params, method_environment_names);

                        let params = method
                            .params
                            .iter()
                            .enumerate()
                            .map(|(index, _)| {
                                method
                                    .param_types
                                    .get(index)
                                    .and_then(|annotation| annotation.as_ref())
                                    .map_or(Type::Dynamic, |annotation| self.resolve_type(annotation))
                            })
                            .collect::<Vec<_>>();

                        let return_type = method
                            .return_type
                            .as_ref()
                            .map(|annotation| self.resolve_type(annotation))
                            .unwrap_or(Type::Dynamic);

                        let generic_constraints = self.generic_constraints(&method.generic_params)?;
                        self.generic_params = previous;

                        let signature = FunctionType {
                            generic_params: method_generic_names,
                            generic_constraints,
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
                    }

                    self.generic_params = previous_generics;
                    let enum_variants = variants.iter().cloned().collect::<HashSet<_>>();

                    self.parents.insert(name.clone(), Vec::new());
                    self.classes.insert(
                        name.clone(),
                        ClassInfo {
                            generic_params: enum_generic_names,
                            generic_constraints: enum_generic_constraints,
                            is_interface: false,
                            bases: Vec::new(),
                            base_types: Vec::new(),
                            methods: method_map,
                            fields: HashMap::new(),
                            static_methods: HashMap::new(),
                            static_fields: HashMap::new(),
                            private_members: HashSet::new(),
                            protected_members: HashSet::new(),
                            enum_variants,
                        },
                    );
                    self.declare_global_declaration(name, Type::Named(name.clone()))?;
                }

                Statement::Interface {
                    name,
                    generic_params,
                    bases,
                    methods,
                } => {
                    let interface_generic_names = Self::validate_generic_declaration(generic_params)?;
                    let previous_generics = std::mem::replace(
                        &mut self.generic_params,
                        interface_generic_names.clone(),
                    );
                    let interface_generic_constraints = self.generic_constraints(generic_params)?;
                    let base_types = bases
                        .iter()
                        .map(|base| self.resolve_type(base))
                        .collect::<Vec<_>>();
                    let base_names = bases
                        .iter()
                        .filter_map(Self::nominal_type_name)
                        .collect::<Vec<_>>();
                    let mut method_map: HashMap<String, Vec<FunctionType>> = HashMap::new();

                    for method in methods {
                        let method_generic_names = Self::validate_nested_generic_declaration(
                            &interface_generic_names,
                            &method.generic_params,
                        )?;
                        let mut method_environment_names = interface_generic_names.clone();
                        method_environment_names.extend(method_generic_names.iter().cloned());
                        let previous = std::mem::replace(&mut self.generic_params, method_environment_names);

                        let params = (0..method.arity)
                            .map(|index| {
                                method
                                    .param_types
                                    .get(index)
                                    .and_then(|annotation| annotation.as_ref())
                                    .map_or(Type::Dynamic, |annotation| self.resolve_type(annotation))
                            })
                            .collect::<Vec<_>>();

                        let return_type = method
                            .return_type
                            .as_ref()
                            .map(|annotation| self.resolve_type(annotation))
                            .unwrap_or(Type::Dynamic);

                        let generic_constraints = self.generic_constraints(&method.generic_params)?;
                        self.generic_params = previous;

                        let signature = FunctionType {
                            generic_params: method_generic_names,
                            generic_constraints,
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
                    }

                    self.generic_params = previous_generics;
                    self.parents.insert(name.clone(), base_names.clone());
                    self.classes.insert(
                        name.clone(),
                        ClassInfo {
                            generic_params: interface_generic_names,
                            generic_constraints: interface_generic_constraints,
                            is_interface: true,
                            bases: base_names,
                            base_types,
                            methods: method_map,
                            fields: HashMap::new(),
                            static_methods: HashMap::new(),
                            static_fields: HashMap::new(),
                            private_members: HashSet::new(),
                            protected_members: HashSet::new(),
                            enum_variants: HashSet::new(),
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
                generic_params,
                params,
                param_types,
                return_type,
                body,
            } => self.check_function(
                name,
                generic_params,
                params,
                param_types,
                return_type.as_ref(),
                body,
            ),

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

            Statement::Enum { name, methods, .. } => self.check_class(name, methods),

            Statement::Interface { .. } => Ok(()),

            // Enregistré par `register_aliases` pour les alias de premier
            // niveau ; un alias déclaré dans un bloc s'enregistre ici.
            Statement::TypeAlias {
                name,
                generic_params,
                type_expr,
            } => {
                self.aliases.entry(name.clone()).or_insert_with(|| LocalTypeAlias {
                    generic_params: generic_params.iter().map(|p| p.name.clone()).collect(),
                    body: type_expr.clone(),
                });

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
        self.class_aliases
            .get(name)
            .cloned()
            .unwrap_or_else(|| name.to_string())
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

            ImportedType::Export {
                ty,
                name,
                interface,
            } => {
                // Une classe importée est connue en détail (constructeurs,
                // méthodes, champs, membres privés), pas seulement par son nom.
                if interface.classes.contains_key(&name) {
                    self.import_class_info(&interface, &name);

                    // `from m import Personne as P` : `P` désigne `Personne`.
                    if binding_name != name {
                        self.class_aliases
                            .insert(binding_name.to_string(), name.clone());
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
                let signatures = self
                    .function_overloads
                    .get(name)
                    .cloned()
                    .unwrap_or_default();

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

            Statement::Class { name, .. } | Statement::Interface { name, .. } | Statement::Enum { name, .. } => {
                Ok((name.clone(), Type::Named(name.clone())))
            }

            _ => Err(CompileError::InvalidExport),
        }
    }

    fn check_function(
        &mut self,
        name: &str,
        generic_params: &[GenericParam],
        params: &[String],
        param_types: &[Option<TypeExpr>],
        return_type: Option<&TypeExpr>,
        body: &[Statement],
    ) -> Result<(), CompileError> {
        let previous_return = self.current_return_type.take();
        let previous_returns = std::mem::take(&mut self.return_types);
        let previous_generics = std::mem::replace(
            &mut self.generic_params,
            generic_params.iter().map(|p| p.name.clone()).collect(),
        );
        let active_generic_constraints = self.generic_constraints_map(generic_params)?;
        let previous_constraints = std::mem::replace(
            &mut self.generic_constraints,
            active_generic_constraints,
        );

        let declared_signature = FunctionType {
            generic_params: generic_params.iter().map(|p| p.name.clone()).collect(),
            generic_constraints: self.generic_constraints(generic_params)?,
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
                self.generic_params = previous_generics;
                self.generic_constraints = previous_constraints;

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
        self.generic_params = previous_generics;
        self.generic_constraints = previous_constraints;

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
        let class_generics = self
            .classes
            .get(class_name)
            .map(|info| info.generic_params.clone())
            .unwrap_or_default();
        let class_constraints = self
            .classes
            .get(class_name)
            .map(|info| info.generic_constraints.clone())
            .unwrap_or_default();
        let mut active_generics = class_generics;
        active_generics.extend(method.generic_params.iter().map(|p| p.name.clone()));
        let previous_generics = std::mem::replace(&mut self.generic_params, active_generics);

        let mut active_generic_constraints: HashMap<String, Vec<GenericConstraint>> =
            class_constraints.into_iter().collect();
        for (name, constraints) in self.generic_constraints(&method.generic_params)? {
            active_generic_constraints.insert(name, constraints);
        }

        let previous_constraints = std::mem::replace(
            &mut self.generic_constraints,
            active_generic_constraints,
        );

        self.push_scope();

        // Une méthode `static` n'a pas de `this` (elle n'est pas appelée sur
        // une instance) : on ne le déclare donc pas dans la portée vérifiée.
        // Une utilisation de `this` dans son corps échouera alors comme une
        // variable non déclarée — cohérent avec le vrai comportement du
        // compilateur (`compile_static_method` ne réserve pas ce slot).
        if !method.is_static {
            let this_type = self
                .classes
                .get(class_name)
                .filter(|info| !info.generic_params.is_empty())
                .map(|info| Type::Generic {
                    name: class_name.to_string(),
                    arguments: info
                        .generic_params
                        .iter()
                        .cloned()
                        .map(Type::TypeParam)
                        .collect(),
                })
                .unwrap_or_else(|| Type::Named(class_name.to_string()));
            self.declare(
                "this",
                Binding {
                    ty: this_type,
                    _mutable: true,
                    native: false,
                },
            )?;
        }

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

        self.current_return_type = method
            .return_type
            .as_ref()
            .map(|annotation| self.resolve_type(annotation));
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
                self.generic_params = previous_generics;
                self.generic_constraints = previous_constraints;

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
        self.generic_params = previous_generics;
        self.generic_constraints = previous_constraints;

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

                if let Some(class_name) = Self::type_name(&object_type) {
                    self.check_member_visibility(&class_name, name)?;

                    // `this.age = valeur` : la valeur doit respecter le type
                    // déclaré du champ (`let age: int`), y compris après
                    // substitution d'un paramètre générique de classe.
                    if let Some(expected) = self.find_field_for_type(&object_type, name) {
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
                    generic_params: Vec::new(),
                    generic_constraints: Vec::new(),
                    params: vec![Type::Dynamic; params.len()],
                    return_type: Box::new(return_type),
                }))
            }

            Expression::Call {
                callee,
                generic_args,
                arguments,
                ..
            } => {
                // Fonction globale SURCHARGÉE (`add(1)`, `add(1, 2)`) : la
                // signature est choisie par arité et par type, comme pour une
                // méthode.
                if let Expression::Variable(name) = callee.as_ref()
                    && let Some(signatures) = self.overloads_of(name)
                {
                    let signature = self.resolve_overload(
                        &signatures,
                        generic_args,
                        arguments,
                        name,
                    )?;

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

                    if let Some(class_name) = Self::type_name(&object_type) {
                        self.check_member_visibility(&class_name, name)?;

                        let signatures = self.find_methods_for_type(&object_type, name);

                        if !signatures.is_empty() {
                            let signature = self.resolve_overload(
                                &signatures,
                                generic_args,
                                arguments,
                                &format!("{class_name}.{name}"),
                            )?;

                            return Ok(*signature.return_type);
                        }

                        // Méthode STATIQUE (`NomClasse.membre(...)`, appelée
                        // sur la classe ou, comme les langages dont Kastel
                        // s'inspire, sur une instance) : même vérification
                        // d'arité/générique que pour une méthode d'instance.
                        let static_signatures = self.find_static_methods(&class_name, name);

                        if !static_signatures.is_empty() {
                            let signature = self.resolve_overload(
                                &static_signatures,
                                generic_args,
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
                        return self.check_call_signature(&signature, generic_args, arguments, name);
                    }
                }

                let callee_type = self.check_expression(callee)?;

                match callee_type {
                    Type::Function(signature) => {
                        let function_name = self.expression_name(callee);
                        self.check_call_signature(&signature, generic_args, arguments, &function_name)
                    }

                    // Fonction surchargée importée d'un module : signature
                    // choisie par arité et par type.
                    Type::Overloads(signatures) => {
                        let function_name = self.expression_name(callee);
                        let signature = self.resolve_overload(
                            &signatures,
                            generic_args,
                            arguments,
                            &function_name,
                        )?;

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
                generic_args,
                arguments,
                ..
            } => {
                let class_name = self.canonical_class_name(class_name);

                if self
                    .classes
                    .get(&class_name)
                    .is_some_and(|info| !info.enum_variants.is_empty())
                {
                    return Err(CompileError::TypeMismatch {
                        expected: "class".to_string(),
                        found: format!("enum {class_name}"),
                    });
                }

                // Constructeur `private` : `new` n'est permis que dans le
                // corps de la classe qui le déclare.
                self.check_member_visibility(&class_name, CONSTRUCTOR_NAME)?;

                let class_info = self.classes.get(&class_name).cloned();
                let class_generic_names = class_info
                    .as_ref()
                    .map(|info| info.generic_params.clone())
                    .unwrap_or_default();

                let mut class_arguments = if !generic_args.is_empty() {
                    if generic_args.len() != class_generic_names.len() {
                        return Err(CompileError::InvalidGenericArity {
                            name: class_name.clone(),
                            expected: class_generic_names.len(),
                            found: generic_args.len(),
                        });
                    }

                    generic_args
                        .iter()
                        .map(|argument| self.resolve_type(argument))
                        .collect::<Vec<_>>()
                } else {
                    vec![Type::Dynamic; class_generic_names.len()]
                };

                // Une classe générique peut déduire ses paramètres depuis son
                // constructeur : `new Box(42)` devient `Box<int>`.
                if !class_generic_names.is_empty() && generic_args.is_empty() {
                    if let Some(inferred) = self.infer_class_arguments_from_constructor(
                        &class_name,
                        arguments,
                        &class_generic_names,
                    )? {
                        class_arguments = inferred;
                    }
                }

                if let Some(info) = &class_info
                    && !info.generic_constraints.is_empty()
                {
                    let substitutions = class_generic_names
                        .iter()
                        .cloned()
                        .zip(class_arguments.iter().cloned())
                        .collect::<HashMap<_, _>>();
                    self.validate_constraint_set(
                        &info.generic_constraints,
                        &substitutions,
                        &class_name,
                    )?;
                }

                let instance_type = if class_generic_names.is_empty() {
                    Type::Named(class_name.clone())
                } else {
                    Type::Generic {
                        name: class_name.clone(),
                        arguments: class_arguments,
                    }
                };

                let signatures = self.find_methods_for_type(&instance_type, CONSTRUCTOR_NAME);

                if !signatures.is_empty() {
                    self.resolve_overload(
                        &signatures,
                        &[],
                        arguments,
                        &format!("{class_name}.{CONSTRUCTOR_NAME}"),
                    )?;
                } else {
                    for argument in arguments {
                        self.check_expression(argument)?;
                    }

                    if !arguments.is_empty() && self.hierarchy_is_known(&class_name) {
                        return Err(CompileError::WrongArgumentCount {
                            expected: 0,
                            found: arguments.len(),
                        });
                    }
                }

                Ok(instance_type)
            }

            Expression::This => {
                let Some(class_name) = self.current_class.as_ref() else {
                    return Ok(Type::Dynamic);
                };

                let Some(class_info) = self.classes.get(class_name) else {
                    return Ok(Type::Named(class_name.clone()));
                };

                if class_info.generic_params.is_empty() {
                    Ok(Type::Named(class_name.clone()))
                } else {
                    Ok(Type::Generic {
                        name: class_name.clone(),
                        arguments: class_info
                            .generic_params
                            .iter()
                            .cloned()
                            .map(Type::TypeParam)
                            .collect(),
                    })
                }
            },

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

        if let Some(capability) = Capability::from_operator(operator) {
            let left_is_generic = matches!(left_type, Type::TypeParam(_));
            let right_is_generic = matches!(right_type, Type::TypeParam(_));

            if left_is_generic || right_is_generic {
                let same_generic = matches!((&left_type, &right_type), (Type::TypeParam(left), Type::TypeParam(right)) if left == right);

                if !same_generic
                    || !self.type_supports_capability(&left_type, capability)
                    || !self.type_supports_capability(&right_type, capability)
                {
                    return Err(CompileError::InvalidBinaryOperation {
                        operator: binary_symbol(operator).to_string(),
                        left: left_type.to_string(),
                        right: right_type.to_string(),
                    });
                }

                return Ok(match operator {
                    BinaryOp::Add
                    | BinaryOp::Subtract
                    | BinaryOp::Multiply
                    | BinaryOp::Modulo => left_type,
                    BinaryOp::Divide => Type::Float,
                    BinaryOp::Equal
                    | BinaryOp::NotEqual
                    | BinaryOp::Is
                    | BinaryOp::Less
                    | BinaryOp::LessEqual
                    | BinaryOp::Greater
                    | BinaryOp::GreaterEqual => Type::Bool,
                    BinaryOp::BitAnd
                    | BinaryOp::BitOr
                    | BinaryOp::BitXor
                    | BinaryOp::ShiftLeft
                    | BinaryOp::ShiftRight => left_type,
                    BinaryOp::And | BinaryOp::Or => unreachable!(),
                });
            }
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
            Type::Named(class_name) | Type::Generic { name: class_name, .. } => {
                self.check_member_visibility(class_name, name)?;

                let signatures = self.find_methods_for_type(object_type, name);

                if signatures.len() == 1 {
                    return Ok(Type::Function(signatures[0].clone()));
                }

                if signatures.len() > 1 {
                    return Ok(Type::Dynamic);
                }

                // Variant d'enum : `Color.Red` retourne le type instancié de
                // l'enum, par exemple `Result<int, str>`.
                if let Some(info) = self.classes.get(class_name)
                    && info.enum_variants.contains(name)
                {
                    if info.generic_params.is_empty() {
                        return Ok(object_type.clone());
                    }

                    return Ok(match object_type {
                        Type::Generic { .. } => object_type.clone(),
                        Type::Named(_) => Type::Generic {
                            name: class_name.clone(),
                            arguments: info
                                .generic_params
                                .iter()
                                .map(|_| Type::Dynamic)
                                .collect(),
                        },
                        _ => unreachable!(),
                    });
                }

                if let Some(field_type) = self.find_field_for_type(object_type, name) {
                    return Ok(field_type);
                }

                // Membre STATIQUE (`NomClasse.membre` sans appel, ou accès
                // via une instance) : espace de noms séparé de `methods`/
                // `fields`, non hérité.
                let static_signatures = self.find_static_methods(class_name, name);

                if static_signatures.len() == 1 {
                    return Ok(Type::Function(static_signatures[0].clone()));
                }

                if static_signatures.len() > 1 {
                    return Ok(Type::Dynamic);
                }

                if let Some(field_type) = self.find_static_field(class_name, name) {
                    return Ok(field_type);
                }
            }

            Type::TypeParam(parameter) => {
                let Some(constraints) = self.generic_constraints.get(parameter) else {
                    return Err(CompileError::InvalidMemberAccess {
                        name: name.to_string(),
                    });
                };

                let mut candidate: Option<FunctionType> = None;

                for constraint in constraints {
                    let GenericConstraint::Interface(interface) = constraint else {
                        continue;
                    };

                    let signatures = self.find_methods_for_type(interface, name);
                    if signatures.len() > 1 {
                        return Ok(Type::Dynamic);
                    }
                    if let Some(signature) = signatures.into_iter().next() {
                        if candidate.is_some() {
                            return Ok(Type::Dynamic);
                        }
                        candidate = Some(signature);
                    }
                }

                let _ = candidate
                    .map(Type::Function)
                    .ok_or_else(|| CompileError::InvalidMemberAccess {
                        name: name.to_string(),
                    });
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

                return signatures
                    .filter(|signatures| signatures.len() > 1)
                    .cloned();
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
    /// (champ ou méthode), avec sa visibilité.
    fn find_member_declaration(
        &self,
        class_name: &str,
        member: &str,
    ) -> Option<(String, Visibility)> {
        let mut pending = vec![class_name.to_string()];
        let mut visited = HashSet::new();

        while let Some(name) = pending.pop() {
            if !visited.insert(name.clone()) {
                continue;
            }

            if let Some(class) = self.classes.get(&name) {
                // Le statique n'est pas hérité : on ne le cherche que sur la
                // classe visée elle-même, pas sur ses bases (`pending`).
                let is_member = class.fields.contains_key(member)
                    || class.methods.contains_key(member)
                    || (name == class_name
                        && (class.static_fields.contains_key(member)
                            || class.static_methods.contains_key(member)));

                if is_member {
                    let visibility = if class.private_members.contains(member) {
                        Visibility::Private
                    } else if class.protected_members.contains(member) {
                        Visibility::Protected
                    } else {
                        Visibility::Public
                    };

                    return Some((name, visibility));
                }

                pending.extend(class.bases.iter().cloned());
            }
        }

        None
    }

    /// `true` si `class_name` est la classe `ancestor` elle-même ou une de
    /// ses classes dérivées. La relation suit uniquement l'héritage de
    /// classes ; les interfaces n'autorisent pas l'accès à un membre protégé.
    fn is_same_or_derived(&self, class_name: &str, ancestor: &str) -> bool {
        let mut pending = vec![class_name.to_string()];
        let mut visited = HashSet::new();

        while let Some(name) = pending.pop() {
            if !visited.insert(name.clone()) {
                continue;
            }

            if name == ancestor {
                return true;
            }

            if let Some(class) = self.classes.get(&name) {
                for base in &class.bases {
                    if self.classes.contains_key(base) {
                        pending.push(base.clone());
                    }
                }
            }
        }

        false
    }

    /// Contrôle de visibilité statique. `private` est limité à la classe qui
    /// déclare le membre ; `protected` est accessible dans toute la
    /// hiérarchie descendante.
    fn check_member_visibility(&self, class_name: &str, member: &str) -> Result<(), CompileError> {
        match self.find_member_declaration(class_name, member) {
            Some((owner, Visibility::Private))
                if self.current_class.as_deref() != Some(owner.as_str()) =>
            {
                Err(CompileError::PrivateMemberAccess {
                    class_name: owner,
                    member: member.to_string(),
                })
            }

            Some((owner, Visibility::Protected))
                if !self
                    .current_class
                    .as_deref()
                    .is_some_and(|current| self.is_same_or_derived(current, &owner)) =>
            {
                Err(CompileError::ProtectedMemberAccess {
                    class_name: owner,
                    member: member.to_string(),
                })
            }

            _ => Ok(()),
        }
    }

    fn type_name(ty: &Type) -> Option<String> {
        match ty {
            Type::Named(name) | Type::Generic { name, .. } => Some(name.clone()),
            _ => None,
        }
    }

    fn type_arguments(ty: &Type) -> Vec<Type> {
        match ty {
            Type::Generic { arguments, .. } => arguments.clone(),
            _ => Vec::new(),
        }
    }

    /// Type déclaré d'un champ, en remontant la hiérarchie et en substituant
    /// les paramètres de type de chaque classe de base.
    fn find_field_for_type(&self, object_type: &Type, field: &str) -> Option<Type> {
        let mut pending = vec![object_type.clone()];
        let mut visited = HashSet::new();

        while let Some(current) = pending.pop() {
            let Some(name) = Self::type_name(&current) else {
                continue;
            };
            let key = current.to_string();
            if !visited.insert(key) {
                continue;
            }

            if let Some(class) = self.classes.get(&name) {
                let substitutions = class
                    .generic_params
                    .iter()
                    .cloned()
                    .zip(Self::type_arguments(&current))
                    .collect::<HashMap<_, _>>();

                if let Some(ty) = class.fields.get(field) {
                    return Some(Self::substitute_type(ty, &substitutions));
                }

                for base in &class.base_types {
                    pending.push(Self::substitute_type(base, &substitutions));
                }
            }
        }

        None
    }

    fn find_methods_for_type(&self, object_type: &Type, method_name: &str) -> Vec<FunctionType> {
        let mut pending = vec![object_type.clone()];
        let mut visited = HashSet::new();
        let mut seen_arities = HashSet::new();
        let mut result = Vec::new();

        while let Some(current) = pending.pop() {
            let Some(name) = Self::type_name(&current) else {
                continue;
            };
            let key = current.to_string();
            if !visited.insert(key) {
                continue;
            }

            if let Some(class) = self.classes.get(&name) {
                let substitutions = class
                    .generic_params
                    .iter()
                    .cloned()
                    .zip(Self::type_arguments(&current))
                    .collect::<HashMap<_, _>>();

                if let Some(overloads) = class.methods.get(method_name) {
                    for signature in overloads {
                        if seen_arities.insert(signature.params.len()) {
                            result.push(Self::substitute_function_signature(
                                signature,
                                &substitutions,
                            ));
                        }
                    }
                }

                for base in &class.base_types {
                    pending.push(Self::substitute_type(base, &substitutions));
                }
            }
        }

        result
    }

    fn find_methods(&self, class_name: &str, method_name: &str) -> Vec<FunctionType> {
        self.find_methods_for_type(&Type::Named(class_name.to_string()), method_name)
    }

    /// Méthodes STATIQUES de `class_name` nommées `method_name`
    /// (`NomClasse.membre(...)`). Contrairement à `find_methods_for_type`,
    /// ne remonte PAS la hiérarchie : le statique n'est pas hérité.
    fn find_static_methods(&self, class_name: &str, method_name: &str) -> Vec<FunctionType> {
        self.classes
            .get(class_name)
            .and_then(|class| class.static_methods.get(method_name))
            .cloned()
            .unwrap_or_default()
    }

    /// Champ STATIQUE de `class_name` nommé `field`. Comme
    /// `find_static_methods`, pas de remontée de hiérarchie.
    fn find_static_field(&self, class_name: &str, field: &str) -> Option<Type> {
        self.classes
            .get(class_name)
            .and_then(|class| class.static_fields.get(field))
            .cloned()
    }

    fn substitute_generic_constraint(
        constraint: &GenericConstraint,
        substitutions: &HashMap<String, Type>,
    ) -> GenericConstraint {
        match constraint {
            GenericConstraint::Capability(capability) => {
                GenericConstraint::Capability(*capability)
            }
            GenericConstraint::Interface(interface) => GenericConstraint::Interface(Box::new(
                Self::substitute_type(interface, substitutions),
            )),
        }
    }

    fn substitute_function_signature(
        signature: &FunctionType,
        substitutions: &HashMap<String, Type>,
    ) -> FunctionType {
        FunctionType {
            generic_params: signature.generic_params.clone(),
            generic_constraints: signature
                .generic_constraints
                .iter()
                .map(|(parameter, constraints)| {
                    (
                        parameter.clone(),
                        constraints
                            .iter()
                            .map(|constraint| {
                                Self::substitute_generic_constraint(
                                    constraint,
                                    substitutions,
                                )
                            })
                            .collect(),
                    )
                })
                .collect(),
            params: signature
                .params
                .iter()
                .map(|param| Self::substitute_type(param, substitutions))
                .collect(),
            return_type: Box::new(Self::substitute_type(&signature.return_type, substitutions)),
        }
    }

    fn infer_generic_bindings(
        expected: &Type,
        actual: &Type,
        bindings: &mut HashMap<String, Type>,
        generic_params: &[String],
    ) -> bool {
        match expected {
            Type::TypeParam(name) if generic_params.iter().any(|parameter| parameter == name) => {
                if let Some(previous) = bindings.get(name) {
                    previous == actual || previous.is_dynamic() || actual.is_dynamic()
                } else {
                    // Avec le typage gradué de Kastel, un argument Dynamic
                    // reste un cas d'inférence valide : le résultat est
                    // alors Dynamic plutôt qu'une erreur d'inférence.
                    bindings.insert(
                        name.clone(),
                        if actual.is_dynamic() {
                            Type::Dynamic
                        } else {
                            actual.clone()
                        },
                    );
                    true
                }
            }
            Type::Array(expected) => matches!(actual, Type::Array(actual) if Self::infer_generic_bindings(expected, actual, bindings, generic_params)),
            Type::Dict(expected_key, expected_value) => matches!(actual, Type::Dict(actual_key, actual_value) if Self::infer_generic_bindings(expected_key, actual_key, bindings, generic_params) && Self::infer_generic_bindings(expected_value, actual_value, bindings, generic_params)),
            Type::Set(expected) => matches!(actual, Type::Set(actual) if Self::infer_generic_bindings(expected, actual, bindings, generic_params)),
            Type::Tuple(expected) => matches!(actual, Type::Tuple(actual) if expected.len() == actual.len() && expected.iter().zip(actual).all(|(expected, actual)| Self::infer_generic_bindings(expected, actual, bindings, generic_params))),
            Type::Generic { name: expected_name, arguments: expected_arguments } => {
                match actual {
                    Type::Generic { name: actual_name, arguments: actual_arguments }
                        if expected_name == actual_name && expected_arguments.len() == actual_arguments.len() =>
                    {
                        expected_arguments.iter().zip(actual_arguments).all(|(expected, actual)| {
                            Self::infer_generic_bindings(expected, actual, bindings, generic_params)
                        })
                    }
                    _ => false,
                }
            }
            _ => true,
        }
    }

    fn instantiate_call_signature(
        &mut self,
        signature: &FunctionType,
        generic_args: &[TypeExpr],
        arguments: &[Expression],
        function_name: &str,
    ) -> Result<FunctionType, CompileError> {
        if signature.params.len() != arguments.len() {
            return Err(CompileError::WrongArgumentCount {
                expected: signature.params.len() as i32,
                found: arguments.len(),
            });
        }

        let actual_types = arguments
            .iter()
            .map(|argument| self.check_expression(argument))
            .collect::<Result<Vec<_>, _>>()?;

        let mut substitutions = HashMap::new();

        if !generic_args.is_empty() {
            if generic_args.len() != signature.generic_params.len() {
                return Err(CompileError::InvalidGenericArity {
                    name: function_name.to_string(),
                    expected: signature.generic_params.len(),
                    found: generic_args.len(),
                });
            }

            for (parameter, argument) in signature
                .generic_params
                .iter()
                .zip(generic_args.iter())
            {
                substitutions.insert(parameter.clone(), self.resolve_type(argument));
            }
        } else if !signature.generic_params.is_empty() {
            for (expected, actual) in signature.params.iter().zip(&actual_types) {
                Self::infer_generic_bindings(
                    expected,
                    actual,
                    &mut substitutions,
                    &signature.generic_params,
                );
            }

            for parameter in &signature.generic_params {
                if !substitutions.contains_key(parameter) {
                    return Err(CompileError::CannotInferGenericType {
                        parameter: parameter.clone(),
                        function: function_name.to_string(),
                    });
                }
            }
        } else if !generic_args.is_empty() {
            return Err(CompileError::InvalidGenericArity {
                name: function_name.to_string(),
                expected: 0,
                found: generic_args.len(),
            });
        }

        self.validate_generic_constraints(signature, &substitutions, function_name)?;

        let instantiated = Self::substitute_function_signature(signature, &substitutions);

        for (index, (actual, expected)) in actual_types
            .iter()
            .zip(&instantiated.params)
            .enumerate()
        {
            if !self.are_assignable(actual, expected) {
                return Err(CompileError::WrongArgumentType {
                    function: function_name.to_string(),
                    index: index + 1,
                    expected: expected.to_string(),
                    found: actual.to_string(),
                });
            }
        }

        Ok(instantiated)
    }

    fn check_call_signature(
        &mut self,
        signature: &FunctionType,
        generic_args: &[TypeExpr],
        arguments: &[Expression],
        function_name: &str,
    ) -> Result<Type, CompileError> {
        Ok(*self
            .instantiate_call_signature(signature, generic_args, arguments, function_name)?
            .return_type)
    }

    fn resolve_overload(
        &mut self,
        signatures: &[FunctionType],
        generic_args: &[TypeExpr],
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

        self.instantiate_call_signature(signature, generic_args, arguments, function_name)
    }

    fn infer_class_arguments_from_constructor(
        &mut self,
        class_name: &str,
        arguments: &[Expression],
        class_generic_names: &[String],
    ) -> Result<Option<Vec<Type>>, CompileError> {
        let signatures = self.find_methods(class_name, CONSTRUCTOR_NAME);
        let Some(signature) = signatures
            .iter()
            .find(|signature| signature.params.len() == arguments.len())
        else {
            return Ok(None);
        };

        let actual_types = arguments
            .iter()
            .map(|argument| self.check_expression(argument))
            .collect::<Result<Vec<_>, _>>()?;
        let mut bindings = HashMap::new();

        for (expected, actual) in signature.params.iter().zip(actual_types) {
            Self::infer_generic_bindings(expected, &actual, &mut bindings, class_generic_names);
        }

        if class_generic_names
            .iter()
            .all(|parameter| bindings.contains_key(parameter))
        {
            Ok(Some(
                class_generic_names
                    .iter()
                    .map(|parameter| bindings.get(parameter).cloned().unwrap_or(Type::Dynamic))
                    .collect(),
            ))
        } else {
            Ok(None)
        }
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
        if expected.is_dynamic() || actual == expected {
            return true;
        }

        if let Type::TypeParam(name) = actual
            && let Some(constraints) = self.generic_constraints.get(name)
            && matches!(expected, Type::Named(_) | Type::Generic { .. })
        {
            return constraints.iter().any(|constraint| {
                matches!(constraint, GenericConstraint::Interface(interface) if self.are_assignable(interface, expected))
            });
        }

        if self.is_generic_nominal_assignable(actual, expected) {
            return true;
        }

        let parents = |name: &str| self.parents.get(name).cloned().unwrap_or_default();
        actual.is_assignable_to(expected, &parents)
    }

    /// Vérifie la compatibilité nominale des classes/interfaces génériques en
    /// tenant compte des arguments de type propagés dans les bases.
    fn is_generic_nominal_assignable(&self, actual: &Type, expected: &Type) -> bool {
        let mut visited = HashSet::new();
        self.is_generic_nominal_assignable_inner(actual, expected, &mut visited)
    }

    fn is_generic_nominal_assignable_inner(
        &self,
        actual: &Type,
        expected: &Type,
        visited: &mut HashSet<(String, String)>,
    ) -> bool {
        if actual == expected {
            return true;
        }

        let key = (actual.to_string(), expected.to_string());
        if !visited.insert(key) {
            return false;
        }

        let actual_name = match actual {
            Type::Named(name) | Type::Generic { name, .. } => name.as_str(),
            _ => return false,
        };

        if let (
            Type::Generic {
                name: left_name,
                arguments: left_args,
            },
            Type::Generic {
                name: right_name,
                arguments: right_args,
            },
        ) = (actual, expected)
            && left_name == right_name
            && left_args.len() == right_args.len()
        {
            return left_args
                .iter()
                .zip(right_args)
                .all(|(left, right)| self.are_assignable(left, right));
        }

        if let Type::Named(expected_name) | Type::Generic {
            name: expected_name,
            ..
        } = expected
        {
            if actual_name == expected_name {
                if let Type::Generic {
                    arguments: actual_args,
                    ..
                } = actual
                    && let Type::Generic {
                        arguments: expected_args,
                        ..
                    } = expected
                    && actual_args.len() == expected_args.len()
                {
                    return actual_args
                        .iter()
                        .zip(expected_args)
                        .all(|(left, right)| self.are_assignable(left, right));
                }

                return matches!(expected, Type::Named(_));
            }
        }

        let Some(class) = self.classes.get(actual_name) else {
            return false;
        };

        let substitutions = class
            .generic_params
            .iter()
            .cloned()
            .zip(Self::type_arguments(actual))
            .collect::<HashMap<_, _>>();

        class.base_types.iter().any(|base| {
            let instantiated = Self::substitute_type(base, &substitutions);
            self.is_generic_nominal_assignable_inner(&instantiated, expected, visited)
        })
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
        let result =
            check("let m: Dict<str, List<int>> = { \"a\": [1, 2] };\nlet v: List<int>= [1, 2];");
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
        assert!(check("class A { func f(x) { return 1; } } let a = new A(); a.f(1, 2);").is_err());
        assert!(check("class B { func initialize(x) { this.x = x; } } let b = new B();").is_err());
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
        assert!(matches!(
            duplicate,
            Err(CompileError::DuplicateMethod { .. })
        ));

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
    fn protected_members_are_accessible_from_derived_classes() {
        let result = check(
            r#"
class Base {
    protected let value: int = 41;
    protected func read() -> int { return this.value; }
}

class Derived: Base {
    func readBase() -> int {
        return this.value + this.read();
    }
}

let d = new Derived();
let n: int = d.readBase();
"#,
        );
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn protected_members_are_forbidden_outside_the_hierarchy() {
        let result = check(
            r#"
class Base {
    protected let value: int = 41;
}

class Derived: Base {
    func ok() -> int { return this.value; }
}

let d = new Derived();
let a = d.value;
"#,
        );

        let mut cursor = match result {
            Ok(()) => panic!("les accès protected externes doivent être refusés"),
            Err(error) => error,
        };
        while let CompileError::WithLocation { source, .. } = cursor {
            cursor = *source;
        }
        assert!(matches!(&cursor, CompileError::ProtectedMemberAccess { .. }), "{:?}", cursor);
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
        assert!(
            check(&format!(
                "{source}\nlet a = new Point(); let b = new Point(1); let c = new Point(1, 2);"
            ))
            .is_ok()
        );
        assert!(check(&format!("{source}\nlet d = new Point(1, 2, 3);")).is_err());
        assert!(check(&format!("{source}\nlet e = new Point(\"x\");")).is_err());

        let duplicate = check("class P { func initialize(a) {} func initialize(b) {} }");
        assert!(matches!(
            duplicate,
            Err(CompileError::DuplicateMethod { .. })
        ));
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
        assert!(check(&format!("{prelude}let p: Person = {{ name: 1, age: 25 }};")).is_err());
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
        assert!(matches!(
            duplicate,
            Err(CompileError::DuplicateFunction { .. })
        ));

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
        assert!(check("class S { func initialize() { this.v = 1; } } let s = new S();").is_ok());
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
    #[test]
    fn generic_functions_support_explicit_arguments_and_inference() {
        let ok = check(
            r#"
func identity<T>(value: T) -> T {
    return value;
}

func first<T>(items: List<T>) -> T {
    return items[0];
}

let a: int = identity(42);
let b: str = identity<str>("kastel");
let c: int = first([1, 2, 3]);
let left = 1;
let right = 2;
let comparison: bool = left < right;
"#,
        );
        assert!(ok.is_ok(), "{:?}", ok.err());

        assert!(check("func identity<T>(value: T) -> T { return value; } let x = identity<int, str>(1);").is_err());
        assert!(check("func identity<T>(value: T) -> T { return value; } let x: int = identity(1.5);").is_err());
        assert!(check("func identity<T>(value: T) -> T { return value; } let d: dynamic = 1; let x: dynamic = identity(d);").is_ok());
    }

    #[test]
    fn generic_constraints_use_capabilities_and_interfaces() {
        let ok = check(
            r#"
interface Marker {}

class Number: Marker {}

func combine<T: Add + Eq>(a: T, b: T) -> T {
    let _same: bool = a == b;
    return a + b;
}

func identity_marker<T: Marker>(value: T) -> T {
    return value;
}

let a: int = combine(10, 20);
let number: Number = new Number();
let same: Number = identity_marker(number);
"#,
        );
        assert!(ok.is_ok(), "{:?}", ok.err());

        assert!(check(
            r#"
interface Marker {}

class Number: Marker {}

func add<T: Add>(a: T, b: T) -> T {
    return a + b;
}

let x: int = add(new Number(), new Number());
"#,
        )
        .is_err());

        assert!(check(
            r#"
interface Marker {}

func identity_marker<T: Marker>(value: T) -> T {
    return value;
}

let x: int = identity_marker(42);
"#,
        )
        .is_err());
    }

    #[test]
    fn generic_interface_constraints_expose_only_interface_members() {
        let ok = check(
            r#"
interface Printable {
    func render() -> str;
}

class Document: Printable {
    func render() -> str {
        return "doc";
    }
}

func render<T: Printable>(value: T) -> str {
    return value.render();
}

let document: Document = new Document();
let output: str = render(document);
"#,
        );
        assert!(ok.is_ok(), "{:?}", ok.err());

        let forward = check(
            r#"
func render<T: Printable>(value: T) -> str {
    return value.render();
}

interface Printable {
    func render() -> str;
}

class Document: Printable {
    func render() -> str {
        return "doc";
    }
}

let output: str = render(new Document());
"#,
        );
        assert!(forward.is_ok(), "{:?}", forward.err());

        assert!(check(
            r#"
interface Printable { func render() -> str; }
class Broken: Printable {}
"#,
        )
        .is_err());

        assert!(check(
            r#"
class NotAnInterface {}
func identity<T: NotAnInterface>(value: T) -> T {
    return value;
}
"#,
        )
        .is_err());
    }

    #[test]
    fn generic_operator_constraints_use_capabilities() {
        let ok = check(
            r#"
func add<T: Add>(a: T, b: T) -> T {
    return a + b;
}

func same<T: Eq>(a: T, b: T) -> bool {
    return a == b;
}

func less<T: Ord>(a: T, b: T) -> bool {
    return a < b;
}

func mask<T: BitAnd>(a: T, b: T) -> T {
    return a & b;
}

let a: int = add(45, 6);
let b: str = add<str>("foo", "bar");
let c: bool = same(1, 1);
let d: bool = less(1, 2);
let e: int = mask(7, 3);
"#,
        );
        assert!(ok.is_ok(), "{:?}", ok.err());

        assert!(check(
            r#"
func add<T: Add>(a: T, b: T) -> T {
    return a + b;
}
let x: int = add(true, false);
"#,
        )
        .is_err());

        assert!(check(
            r#"
func add<T>(a: T, b: T) -> T {
    return a + b;
}
"#,
        )
        .is_err());
    }

    #[test]
    fn generic_classes_infer_constructor_arguments_and_substitute_members() {
        let ok = check(
            r#"
class Box<T> {
    private let value: T;

    func initialize(value: T) {
        this.value = value;
    }

    func get() -> T {
        return this.value;
    }
}

let box: Box<int> = new Box(42);
let value: int = box.get();
let explicit: Box<str> = new Box<str>("kastel");
let text: str = explicit.get();
"#,
        );
        assert!(ok.is_ok(), "{:?}", ok.err());

        assert!(check("class Box<T> { func initialize(value: T) {} } let b = new Box<int, str>(1);").is_err());
    }

    #[test]
    fn generic_class_constraints_are_checked_and_available_in_methods() {
        let ok = check(
            r#"
class Box<T: Add> {
    func add(a: T, b: T) -> T {
        return a + b;
    }
}

let box: Box<int> = new Box<int>();
let value: int = box.add(1, 2);
"#,
        );
        assert!(ok.is_ok(), "{:?}", ok.err());

        assert!(check(
            r#"
class Box<T: Add> {
    func add(a: T, b: T) -> T {
        return a + b;
    }
}

let box = new Box<bool>();
"#,
        )
        .is_err());
    }

    #[test]
    fn generic_methods_support_inference_and_explicit_arguments() {
        let ok = check(
            r#"
class Holder {
    func echo<T>(value: T) -> T {
        return value;
    }
}

let holder = new Holder();
let a: int = holder.echo(10);
let b: str = holder.echo<str>("kastel");
"#,
        );
        assert!(ok.is_ok(), "{:?}", ok.err());

        assert!(check("class Holder { func echo<T>(value: T) -> T { return value; } } let x: int = new Holder().echo<int, str>(1);").is_err());
    }

    #[test]
    fn generic_aliases_can_be_instantiated() {
        let ok = check(
            r#"

type Pair<A, B> = { first: A, second: B };
let pair: Pair<int, str> = { first: 7, second: "kastel" };
let first: int = pair.first;
let second: str = pair.second;
"#,
        );
        assert!(ok.is_ok(), "{:?}", ok.err());

        assert!(check("type Pair<A, B> = { first: A, second: B }; let p: Pair<int> = { first: 1, second: \"x\" };").is_err());
    }

    #[test]
    fn generic_inheritance_preserves_base_type_arguments() {
        let ok = check(
            r#"
class Parent<T> {
    protected let value: T;

    func get() -> T {
        return this.value;
    }
}

class Child: Parent<int> {}

let child: Child = new Child();
let value: int = child.get();
let parent: Parent<int> = child;
"#,
        );
        assert!(ok.is_ok(), "{:?}", ok.err());

        assert!(check("class Parent<T> {} class Child: Parent<int> {} let bad: Parent<str> = new Child();").is_err());
    }

    #[test]
    fn generic_interfaces_and_enums_preserve_instantiated_types() {
        let ok = check(
            r#"
interface Comparable<T> {
    func compare(other: T) -> int;
}

class Number: Comparable<int> {
    func compare(other: int) -> int {
        return 0;
    }
}

let comparable: Comparable<int> = new Number();

 enum Result<T, E> {
    Ok,
    Error
}

let result: Result<int, str> = Result.Ok;
let other: Result<float, str> = Result.Ok;
"#,
        );
        assert!(ok.is_ok(), "{:?}", ok.err());

        assert!(check("interface Comparable<T> { func compare(other: T) -> int; } class Number: Comparable<int> { func compare(other: int) -> int { return 0; } } let bad: Comparable<str> = new Number();").is_err());
    }


}
