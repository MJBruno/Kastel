//! Passes de transformation de la HIM.
//!
//! Les passes travaillent uniquement sur la représentation intermédiaire
//! possédée (`him.rs`). Elles ne dépendent ni du parser ni de l'AST.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use super::him::{AssignmentTarget, BinaryOp, Expression, Literal, MatchArm, Statement};

/// Contrat commun d'une passe de transformation HIM.
pub(crate) trait HimPass {
    /// Transforme le module en place.
    fn run(&self, statements: &mut Vec<Statement>);
}

/// Exécute les passes HIM activées par défaut.
///
/// L'ordre est volontairement stable : les passes de simplification
/// produisent d'abord les littéraux et le contrôle de flux simplifié, puis
/// l'élimination des valeurs mortes profite de ces résultats.
pub(crate) fn run_default_passes(statements: &mut Vec<Statement>) {
    let passes: [&dyn HimPass; 5] = [
        &ConstantFolder,
        &ConstantPropagator,
        &DeadCodeEliminator,
        &DeadValueEliminator,
        &DeadStoreEliminator,
    ];

    for pass in passes {
        pass.run(statements);
    }
}

/// Replie les expressions entièrement constantes.
///
/// Cette passe est volontairement conservatrice :
/// - aucun appel, accès membre, allocation ou opération utilisateur n'est
///   transformé ;
/// - les divisions/modulos par zéro restent dans le bytecode ;
/// - les opérations entières ne sont pliées que si elles ne débordent pas ;
/// - les comparaisons entier/flottant suivent la règle exacte du VM.
pub(crate) struct ConstantFolder;

/// Propage les valeurs littérales des liaisons immuables et simplifie les
/// branches dont la condition est devenue constante.
///
/// La passe reste volontairement conservatrice :
/// - seules les liaisons `let` immuables sont propagées ;
/// - aucune valeur n’est propagée à travers une fonction/closure ;
/// - les corps de boucles sont analysés avec un environnement isolé ;
/// - après un `if`, seules les constantes qui existent avec la même valeur
///   dans les deux branches peuvent survivre ;
/// - une branche constante est remplacée par son corps ;
/// - `while false` est supprimé ; `while true` est conservé.
pub(crate) struct ConstantPropagator;

/// Supprime le code linéairement inaccessible après une instruction
/// terminatrice et applique la même analyse aux corps imbriqués.
///
/// Cette passe est volontairement conservatrice :
/// - elle ne supprime pas les déclarations ou expressions seulement
///   "inutilisées" ;
/// - elle ne considère un `if` comme terminant que lorsque les deux branches
///   terminent toujours ;
/// - elle ne supprime jamais le `finally`, qui participe à la sémantique de
///   contrôle même lorsqu'un `return` ou un `throw` précède.
pub(crate) struct DeadCodeEliminator;

/// Supprime les liaisons locales immuables dont la valeur est un littéral pur
/// et qui ne sont jamais lues dans leur fonction.
///
/// Cette passe reste volontairement très conservative :
/// - elle ne touche pas aux `let` du module principal ;
/// - elle ne supprime que les initialisateurs réduits à un littéral ;
/// - elle tient compte des lectures réalisées par des closures imbriquées ;
/// - elle ne supprime jamais une affectation mutable ;
/// - les champs de classes et les paramètres restent intacts.
pub(crate) struct DeadValueEliminator;

/// Supprime les écritures locales mortes à partir de l'analyse de vivacité.
///
/// Seules les affectations à des variables locales dont la valeur n'est pas
/// lue sur un chemin futur sont candidates. Les affectations aux membres,
/// aux index, aux globals et les initialiseurs avec effets de bord restent
/// conservés.
pub(crate) struct DeadStoreEliminator;

/// Analyse de vivacité basée sur un petit graphe de contrôle de flux HIM.
///
/// Le CFG modélise précisément les séquences, `if`, `while`, `for`,
/// `break` et `continue`. Les régions dont la sémantique de contrôle est
/// exceptionnelle ou dynamiquement observable (`match`, `try`, déclarations
/// de type/fonction) restent volontairement opaques pour la DSE.
pub(crate) struct LivenessAnalyzer;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct LivenessSummary {
    pub(crate) live_in: HashSet<String>,
}

#[derive(Debug, Clone)]
struct CfgNode {
    statement_id: Option<usize>,
    uses: HashSet<String>,
    defs: HashSet<String>,
    successors: Vec<usize>,
    removable_store: Option<String>,
}

#[derive(Debug, Clone)]
struct ControlFlowGraph {
    nodes: Vec<CfgNode>,
    entry: Option<usize>,
    exit: usize,
}

struct CfgBuilder<'a> {
    nodes: Vec<CfgNode>,
    next_statement_id: usize,
    locals: &'a HashSet<String>,
    exit: usize,
}

#[derive(Debug, Clone)]
struct CfgLiveness {
    live_in: Vec<HashSet<String>>,
    live_out: Vec<HashSet<String>>,
}

impl HimPass for ConstantFolder {
    fn run(&self, statements: &mut Vec<Statement>) {
        for statement in statements {
            fold_statement(statement);
        }
    }
}

impl HimPass for ConstantPropagator {
    fn run(&self, statements: &mut Vec<Statement>) {
        let mut environment = HashMap::new();
        propagate_block(statements, &mut environment);
    }
}

impl HimPass for DeadCodeEliminator {
    fn run(&self, statements: &mut Vec<Statement>) {
        eliminate_block(statements);
    }
}

impl HimPass for DeadValueEliminator {
    fn run(&self, statements: &mut Vec<Statement>) {
        eliminate_dead_values_in_nested_functions(statements);
    }
}

impl HimPass for DeadStoreEliminator {
    fn run(&self, statements: &mut Vec<Statement>) {
        eliminate_dead_stores_in_nested_functions(statements);
    }
}

impl LivenessAnalyzer {
    #[cfg(test)]
    pub(crate) fn analyze_block(&self, statements: &[Statement]) -> LivenessSummary {
        self.analyze_block_with_live_out(statements, &HashSet::new())
    }

    #[cfg(test)]
    pub(crate) fn analyze_block_with_live_out(
        &self,
        statements: &[Statement],
        live_out: &HashSet<String>,
    ) -> LivenessSummary {
        // L'analyse publique de test reçoit un bloc autonome, sans la table
        // des variables locales fournie normalement par le compilateur de
        // fonction. Dans ce contexte, une affectation simple constitue une
        // définition locale pour permettre à l'analyse de distinguer une
        // lecture réellement non liée d'une valeur définie précédemment.
        let mut locals = HashSet::new();
        collect_liveness_definition_names(statements, &mut locals);
        let cfg = ControlFlowGraph::build(statements, &locals);
        let analysis = self.solve(&cfg, live_out);
        let live_in = cfg
            .entry
            .and_then(|entry| analysis.live_in.get(entry).cloned())
            .unwrap_or_default();

        LivenessSummary { live_in }
    }

    fn analyze_function(
        &self,
        statements: &[Statement],
        locals: &HashSet<String>,
    ) -> (LivenessSummary, HashSet<usize>) {
        let cfg = ControlFlowGraph::build(statements, locals);
        let analysis = self.solve(&cfg, &HashSet::new());
        let live_in = cfg
            .entry
            .and_then(|entry| analysis.live_in.get(entry).cloned())
            .unwrap_or_default();
        let mut dead_stores = HashSet::new();

        for (node_id, node) in cfg.nodes.iter().enumerate() {
            let Some(name) = &node.removable_store else {
                continue;
            };

            if !analysis.live_out[node_id].contains(name) {
                if let Some(statement_id) = node.statement_id {
                    dead_stores.insert(statement_id);
                }
            }
        }

        (LivenessSummary { live_in }, dead_stores)
    }

    fn solve(&self, cfg: &ControlFlowGraph, exit_live_in: &HashSet<String>) -> CfgLiveness {
        let mut live_in = vec![HashSet::new(); cfg.nodes.len()];
        let mut live_out = vec![HashSet::new(); cfg.nodes.len()];

        if let Some(exit) = live_in.get_mut(cfg.exit) {
            *exit = exit_live_in.clone();
        }

        loop {
            let mut changed = false;

            for node_id in (0..cfg.nodes.len()).rev() {
                if node_id == cfg.exit {
                    continue;
                }

                let mut next_live_out = HashSet::new();
                for successor in &cfg.nodes[node_id].successors {
                    next_live_out.extend(live_in[*successor].iter().cloned());
                }

                let mut next_live_in = next_live_out.clone();
                for definition in &cfg.nodes[node_id].defs {
                    next_live_in.remove(definition);
                }
                next_live_in.extend(cfg.nodes[node_id].uses.iter().cloned());

                if next_live_out != live_out[node_id] {
                    live_out[node_id] = next_live_out;
                    changed = true;
                }
                if next_live_in != live_in[node_id] {
                    live_in[node_id] = next_live_in;
                    changed = true;
                }
            }

            if !changed {
                break;
            }
        }

        CfgLiveness { live_in, live_out }
    }
}

impl ControlFlowGraph {
    fn build(statements: &[Statement], locals: &HashSet<String>) -> Self {
        let mut builder = CfgBuilder {
            nodes: Vec::new(),
            next_statement_id: 0,
            locals,
            exit: 0,
        };
        let exit = builder.add_node(None, HashSet::new(), HashSet::new(), Vec::new(), None);
        builder.exit = exit;
        let entry = builder.build_sequence(statements, exit, None, None);

        Self {
            nodes: builder.nodes,
            entry,
            exit,
        }
    }
}

impl<'a> CfgBuilder<'a> {
    fn add_node(
        &mut self,
        statement_id: Option<usize>,
        uses: HashSet<String>,
        defs: HashSet<String>,
        successors: Vec<usize>,
        removable_store: Option<String>,
    ) -> usize {
        let id = self.nodes.len();
        self.nodes.push(CfgNode {
            statement_id,
            uses,
            defs,
            successors,
            removable_store,
        });
        id
    }

    fn reserve_node(&mut self, statement_id: usize) -> usize {
        self.add_node(
            Some(statement_id),
            HashSet::new(),
            HashSet::new(),
            Vec::new(),
            None,
        )
    }

    fn build_sequence(
        &mut self,
        statements: &[Statement],
        next: usize,
        break_target: Option<usize>,
        continue_target: Option<usize>,
    ) -> Option<usize> {
        let mut current = next;

        for statement in statements.iter().rev() {
            current = self.build_statement(
                statement,
                current,
                break_target,
                continue_target,
            );
        }

        (current != next).then_some(current)
    }

    fn build_statement(
        &mut self,
        statement: &Statement,
        next: usize,
        break_target: Option<usize>,
        continue_target: Option<usize>,
    ) -> usize {
        let statement_id = self.next_statement_id;
        self.next_statement_id += 1;
        self.build_statement_with_id(
            statement,
            statement_id,
            next,
            break_target,
            continue_target,
        )
    }

    fn build_statement_with_id(
        &mut self,
        statement: &Statement,
        statement_id: usize,
        next: usize,
        break_target: Option<usize>,
        continue_target: Option<usize>,
    ) -> usize {
        match statement {
            Statement::Positioned { statement, .. } => self.build_statement_with_id(
                statement,
                statement_id,
                next,
                break_target,
                continue_target,
            ),

            Statement::Export { statement } => self.build_statement_with_id(
                statement,
                statement_id,
                next,
                break_target,
                continue_target,
            ),

            Statement::Let { name, value, .. } => {
                let mut uses = HashSet::new();
                collect_used_variables_expression(value, &mut uses);
                let mut defs = HashSet::new();
                defs.insert(name.clone());
                self.add_node(
                    Some(statement_id),
                    uses,
                    defs,
                    vec![next],
                    None,
                )
            }

            Statement::Assignment { target, value } => {
                let mut uses = HashSet::new();
                collect_used_variables_expression(value, &mut uses);
                let mut defs = HashSet::new();
                let mut removable_store = None;

                match target {
                    AssignmentTarget::Variable(name) => {
                        if self.locals.contains(name) {
                            defs.insert(name.clone());
                            if matches!(value, Expression::Literal(_)) {
                                removable_store = Some(name.clone());
                            }
                        }
                    }
                    AssignmentTarget::Member { .. } | AssignmentTarget::Index { .. } => {
                        collect_used_variables_target(target, &mut uses);
                    }
                }

                self.add_node(
                    Some(statement_id),
                    uses,
                    defs,
                    vec![next],
                    removable_store,
                )
            }

            Statement::Expression { expression } | Statement::Throw { value: expression } => {
                let mut uses = HashSet::new();
                collect_used_variables_expression(expression, &mut uses);
                let successors = if matches!(statement, Statement::Throw { .. }) {
                    vec![self.exit]
                } else {
                    vec![next]
                };
                self.add_node(
                    Some(statement_id),
                    uses,
                    HashSet::new(),
                    successors,
                    None,
                )
            }

            Statement::Return { value } => {
                let mut uses = HashSet::new();
                if let Some(value) = value {
                    collect_used_variables_expression(value, &mut uses);
                }
                self.add_node(
                    Some(statement_id),
                    uses,
                    HashSet::new(),
                    vec![self.exit],
                    None,
                )
            }

            Statement::Break => self.add_node(
                Some(statement_id),
                HashSet::new(),
                HashSet::new(),
                vec![break_target.unwrap_or(self.exit)],
                None,
            ),

            Statement::Continue => self.add_node(
                Some(statement_id),
                HashSet::new(),
                HashSet::new(),
                vec![continue_target.unwrap_or(self.exit)],
                None,
            ),

            Statement::Block(body) => {
                let body_entry = self.build_sequence(
                    body,
                    next,
                    break_target,
                    continue_target,
                )
                .unwrap_or(next);
                self.add_node(
                    Some(statement_id),
                    HashSet::new(),
                    HashSet::new(),
                    vec![body_entry],
                    None,
                )
            }

            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let then_entry = self
                    .build_sequence(then_branch, next, break_target, continue_target)
                    .unwrap_or(next);
                let else_entry = else_branch
                    .as_ref()
                    .and_then(|branch| {
                        self.build_sequence(branch, next, break_target, continue_target)
                    })
                    .unwrap_or(next);

                let mut uses = HashSet::new();
                collect_used_variables_expression(condition, &mut uses);
                let mut successors = vec![then_entry];
                if else_entry != then_entry {
                    successors.push(else_entry);
                }
                self.add_node(
                    Some(statement_id),
                    uses,
                    HashSet::new(),
                    successors,
                    None,
                )
            }

            Statement::While { condition, body } => {
                let condition_node = self.reserve_node(statement_id);
                let body_entry = self
                    .build_sequence(body, condition_node, Some(next), Some(condition_node))
                    .unwrap_or(condition_node);

                let mut uses = HashSet::new();
                collect_used_variables_expression(condition, &mut uses);
                self.nodes[condition_node].uses = uses;
                self.nodes[condition_node].successors = vec![body_entry, next];
                condition_node
            }

            Statement::ForIn { variable, iterable, body } => {
                let iteration_node = self.reserve_node(statement_id);
                let body_entry = self
                    .build_sequence(body, iteration_node, Some(next), Some(iteration_node))
                    .unwrap_or(iteration_node);

                let mut uses = HashSet::new();
                collect_used_variables_expression(iterable, &mut uses);
                let mut defs = HashSet::new();
                defs.insert(variable.clone());
                self.nodes[iteration_node].uses = uses;
                self.nodes[iteration_node].defs = defs;
                self.nodes[iteration_node].successors = vec![body_entry, next];
                iteration_node
            }

            // Les effets de contrôle de `match` dépendent des patterns et des
            // gardes, et peuvent aussi introduire des bindings. Tant que nous
            // n'avons pas leur CFG dédié, on garde toute la région opaque.
            Statement::Match { value, arms } => {
                let mut uses = HashSet::new();
                collect_used_variables_expression(value, &mut uses);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        collect_used_variables_expression(guard, &mut uses);
                    }
                    collect_used_variables_statements(&arm.body, &mut uses);
                }
                self.add_node(
                    Some(statement_id),
                    uses,
                    HashSet::new(),
                    vec![next],
                    None,
                )
            }

            // Une région `try` peut transférer le contrôle vers catch/finally
            // depuis pratiquement n'importe quel point. Elle reste donc opaque
            // jusqu'à l'ajout d'un véritable graphe d'exceptions HIM.
            Statement::Try {
                try_body,
                catch_body,
                finally_body,
                ..
            } => {
                let mut uses = HashSet::new();
                collect_used_variables_statements(try_body, &mut uses);
                if let Some(body) = catch_body {
                    collect_used_variables_statements(body, &mut uses);
                }
                if let Some(body) = finally_body {
                    collect_used_variables_statements(body, &mut uses);
                }
                self.add_node(
                    Some(statement_id),
                    uses,
                    HashSet::new(),
                    vec![next],
                    None,
                )
            }

            // Les déclarations exécutées dans une fonction peuvent capturer
            // des variables du scope courant. Elles restent donc des barrières
            // pour la DSE locale.
            Statement::Function { body, .. } => {
                let mut uses = HashSet::new();
                collect_used_variables_statements(body, &mut uses);
                self.add_node(
                    Some(statement_id),
                    uses,
                    HashSet::new(),
                    vec![next],
                    None,
                )
            }

            Statement::Class { fields, methods, .. } => {
                let mut uses = HashSet::new();
                for field in fields {
                    if let Some(initializer) = &field.initializer {
                        collect_used_variables_expression(initializer, &mut uses);
                    }
                }
                for method in methods {
                    collect_used_variables_statements(&method.body, &mut uses);
                }
                self.add_node(
                    Some(statement_id),
                    uses,
                    HashSet::new(),
                    vec![next],
                    None,
                )
            }

            Statement::Enum { methods, .. } => {
                let mut uses = HashSet::new();
                for method in methods {
                    collect_used_variables_statements(&method.body, &mut uses);
                }
                self.add_node(
                    Some(statement_id),
                    uses,
                    HashSet::new(),
                    vec![next],
                    None,
                )
            }

            Statement::Import { .. }
            | Statement::FromImport { .. }
            | Statement::TypeAlias { .. }
            | Statement::Interface { .. } => self.add_node(
                Some(statement_id),
                HashSet::new(),
                HashSet::new(),
                vec![next],
                None,
            ),
        }
    }
}

fn eliminate_dead_stores_in_nested_functions(statements: &mut [Statement]) {
    for statement in statements {
        discover_and_optimize_function(statement);
    }
}

fn discover_and_optimize_function(statement: &mut Statement) {
    match statement {
        Statement::Positioned { statement, .. } => discover_and_optimize_function(statement),
        Statement::Block(body) => {
            for statement in body {
                discover_and_optimize_function(statement);
            }
        }
        Statement::If {
            condition,
            then_branch,
            else_branch,
        } => {
            discover_and_optimize_nested_functions_in_expression(condition);
            for statement in then_branch {
                discover_and_optimize_function(statement);
            }
            if let Some(branch) = else_branch {
                for statement in branch {
                    discover_and_optimize_function(statement);
                }
            }
        }
        Statement::While { condition, body } => {
            discover_and_optimize_nested_functions_in_expression(condition);
            for statement in body {
                discover_and_optimize_function(statement);
            }
        }
        Statement::ForIn { iterable, body, .. } => {
            discover_and_optimize_nested_functions_in_expression(iterable);
            for statement in body {
                discover_and_optimize_function(statement);
            }
        }
        Statement::Match { value, arms } => {
            discover_and_optimize_nested_functions_in_expression(value);
            for arm in arms {
                if let Some(guard) = &mut arm.guard {
                    discover_and_optimize_nested_functions_in_expression(guard);
                }
                for statement in &mut arm.body {
                    discover_and_optimize_function(statement);
                }
            }
        }
        Statement::Try {
            try_body,
            catch_body,
            finally_body,
            ..
        } => {
            for statement in try_body {
                discover_and_optimize_function(statement);
            }
            if let Some(body) = catch_body {
                for statement in body {
                    discover_and_optimize_function(statement);
                }
            }
            if let Some(body) = finally_body {
                for statement in body {
                    discover_and_optimize_function(statement);
                }
            }
        }
        Statement::Function { body, params, .. } => {
            let locals = collect_function_local_bindings(params, body);
            eliminate_dead_stores_in_block(body, &locals);
        }
        Statement::Class { fields, methods, .. } => {
            for field in fields {
                if let Some(initializer) = &mut field.initializer {
                    discover_and_optimize_nested_functions_in_expression(initializer);
                }
            }
            for method in methods {
                let locals = collect_function_local_bindings(&method.params, &method.body);
                eliminate_dead_stores_in_block(&mut method.body, &locals);
            }
        }
        Statement::Enum { methods, .. } => {
            for method in methods {
                let locals = collect_function_local_bindings(&method.params, &method.body);
                eliminate_dead_stores_in_block(&mut method.body, &locals);
            }
        }
        Statement::Export { statement } => discover_and_optimize_function(statement),
        Statement::Let { value, .. }
        | Statement::Expression { expression: value }
        | Statement::Throw { value }
        | Statement::Return {
            value: Some(value),
        } => discover_and_optimize_nested_functions_in_expression(value),
        Statement::Assignment { target, value } => {
            discover_and_optimize_nested_functions_in_assignment_target(target);
            discover_and_optimize_nested_functions_in_expression(value);
        }
        Statement::Return { value: None }
        | Statement::Import { .. }
        | Statement::FromImport { .. }
        | Statement::TypeAlias { .. }
        | Statement::Interface { .. }
        | Statement::Break
        | Statement::Continue => {}
    }
}

fn discover_and_optimize_nested_functions_in_assignment_target(target: &mut AssignmentTarget) {
    match target {
        AssignmentTarget::Variable(_) => {}
        AssignmentTarget::Index { object, index } => {
            discover_and_optimize_nested_functions_in_expression(object);
            discover_and_optimize_nested_functions_in_expression(index);
        }
        AssignmentTarget::Member { object, .. } => {
            discover_and_optimize_nested_functions_in_expression(object);
        }
    }
}

fn discover_and_optimize_nested_functions_in_expression(expression: &mut Expression) {
    match expression {
        Expression::Function { params, body } => {
            let locals = collect_function_local_bindings(params, body);
            eliminate_dead_stores_in_block(body, &locals);
        }
        Expression::Unary { right, .. } => {
            discover_and_optimize_nested_functions_in_expression(right);
        }
        Expression::Binary { left, right, .. } => {
            discover_and_optimize_nested_functions_in_expression(left);
            discover_and_optimize_nested_functions_in_expression(right);
        }
        Expression::Call { callee, arguments, .. } => {
            discover_and_optimize_nested_functions_in_expression(callee);
            for argument in arguments {
                discover_and_optimize_nested_functions_in_expression(argument);
            }
        }
        Expression::Member { object, .. } => {
            discover_and_optimize_nested_functions_in_expression(object);
        }
        Expression::Index { object, index, .. } => {
            discover_and_optimize_nested_functions_in_expression(object);
            discover_and_optimize_nested_functions_in_expression(index);
        }
        Expression::New { arguments, .. } => {
            for argument in arguments {
                discover_and_optimize_nested_functions_in_expression(argument);
            }
        }
        Expression::Array(elements) | Expression::Tuple(elements) => {
            for element in elements {
                discover_and_optimize_nested_functions_in_expression(element);
            }
        }
        Expression::Dict(fields) | Expression::Record(fields) => {
            for (_, value) in fields {
                discover_and_optimize_nested_functions_in_expression(value);
            }
        }
        Expression::Try(expression) | Expression::Await(expression) => {
            discover_and_optimize_nested_functions_in_expression(expression);
        }
        Expression::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            discover_and_optimize_nested_functions_in_expression(condition);
            discover_and_optimize_nested_functions_in_expression(then_expr);
            discover_and_optimize_nested_functions_in_expression(else_expr);
        }
        Expression::Literal(_) | Expression::Variable(_) | Expression::SelfValue => {}
    }
}

fn collect_function_local_bindings(
    params: &[String],
    statements: &[Statement],
) -> HashSet<String> {
    let mut locals = params.iter().cloned().collect();
    collect_local_bindings_from_statements(statements, &mut locals);
    locals
}

#[cfg(test)]
fn collect_liveness_definition_names(
    statements: &[Statement],
    locals: &mut HashSet<String>,
) {
    for statement in statements {
        match statement {
            Statement::Positioned { statement, .. }
            | Statement::Export { statement } => {
                collect_liveness_definition_names(std::slice::from_ref(statement), locals);
            }
            Statement::Let { name, .. } => {
                locals.insert(name.clone());
            }
            Statement::Assignment {
                target: AssignmentTarget::Variable(name),
                ..
            } => {
                locals.insert(name.clone());
            }
            Statement::If {
                then_branch,
                else_branch,
                ..
            } => {
                collect_liveness_definition_names(then_branch, locals);
                if let Some(branch) = else_branch {
                    collect_liveness_definition_names(branch, locals);
                }
            }
            Statement::While { body, .. }
            | Statement::ForIn { body, .. }
            | Statement::Block(body) => {
                collect_liveness_definition_names(body, locals);
            }
            Statement::Match { arms, .. } => {
                for arm in arms {
                    collect_pattern_bindings(&arm.pattern, locals);
                    collect_liveness_definition_names(&arm.body, locals);
                }
            }
            Statement::Try {
                try_body,
                catch_name,
                catch_body,
                finally_body,
                ..
            } => {
                collect_liveness_definition_names(try_body, locals);
                if let Some(name) = catch_name {
                    locals.insert(name.clone());
                }
                if let Some(body) = catch_body {
                    collect_liveness_definition_names(body, locals);
                }
                if let Some(body) = finally_body {
                    collect_liveness_definition_names(body, locals);
                }
            }
            Statement::Function { .. }
            | Statement::Class { .. }
            | Statement::Enum { .. }
            | Statement::TypeAlias { .. }
            | Statement::Interface { .. }
            | Statement::Import { .. }
            | Statement::FromImport { .. }
            | Statement::Assignment { .. }
            | Statement::Expression { .. }
            | Statement::Return { .. }
            | Statement::Throw { .. }
            | Statement::Break
            | Statement::Continue => {}
        }
    }
}

fn collect_local_bindings_from_statements(
    statements: &[Statement],
    locals: &mut HashSet<String>,
) {
    for statement in statements {
        match statement {
            Statement::Positioned { statement, .. } => {
                collect_local_bindings_from_statements(std::slice::from_ref(statement), locals);
            }
            Statement::Let { name, .. } => {
                locals.insert(name.clone());
            }
            Statement::ForIn { variable, body, .. } => {
                locals.insert(variable.clone());
                collect_local_bindings_from_statements(body, locals);
            }
            Statement::If {
                then_branch,
                else_branch,
                ..
            } => {
                collect_local_bindings_from_statements(then_branch, locals);
                if let Some(branch) = else_branch {
                    collect_local_bindings_from_statements(branch, locals);
                }
            }
            Statement::While { body, .. } | Statement::Block(body) => {
                collect_local_bindings_from_statements(body, locals);
            }
            Statement::Match { arms, .. } => {
                for arm in arms {
                    collect_pattern_bindings(&arm.pattern, locals);
                    collect_local_bindings_from_statements(&arm.body, locals);
                }
            }
            Statement::Try {
                try_body,
                catch_name,
                catch_body,
                finally_body,
                ..
            } => {
                collect_local_bindings_from_statements(try_body, locals);
                if let Some(name) = catch_name {
                    locals.insert(name.clone());
                }
                if let Some(body) = catch_body {
                    collect_local_bindings_from_statements(body, locals);
                }
                if let Some(body) = finally_body {
                    collect_local_bindings_from_statements(body, locals);
                }
            }
            Statement::Function { .. } => {}
            Statement::Class { .. }
            | Statement::Enum { .. }
            | Statement::TypeAlias { .. }
            | Statement::Interface { .. }
            | Statement::Import { .. }
            | Statement::FromImport { .. }
            | Statement::Export { .. }
            | Statement::Assignment { .. }
            | Statement::Expression { .. }
            | Statement::Return { .. }
            | Statement::Throw { .. }
            | Statement::Break
            | Statement::Continue => {}
        }
    }
}

fn collect_pattern_bindings(pattern: &super::him::Pattern, locals: &mut HashSet<String>) {
    match pattern {
        super::him::Pattern::Binding(name) => {
            locals.insert(name.clone());
        }
        super::him::Pattern::Or(patterns)
        | super::him::Pattern::Array(patterns)
        | super::him::Pattern::ArrayRest(patterns)
        | super::him::Pattern::Tuple(patterns) => {
            for pattern in patterns {
                collect_pattern_bindings(pattern, locals);
            }
        }
        super::him::Pattern::Range { start, end, .. } => {
            collect_pattern_bindings(start, locals);
            collect_pattern_bindings(end, locals);
        }
        super::him::Pattern::OptionSome(pattern)
        | super::him::Pattern::ResultOk(pattern)
        | super::him::Pattern::ResultErr(pattern) => {
            collect_pattern_bindings(pattern, locals);
        }
        super::him::Pattern::Wildcard
        | super::him::Pattern::Literal(_)
        | super::him::Pattern::EnumVariant { .. } => {}
    }
}

fn eliminate_dead_stores_in_block(
    statements: &mut Vec<Statement>,
    locals: &HashSet<String>,
) {
    for statement in statements.iter_mut() {
        discover_and_optimize_function(statement);
    }

    let analyzer = LivenessAnalyzer;
    let (_, dead_stores) = analyzer.analyze_function(statements, locals);
    if dead_stores.is_empty() {
        return;
    }

    let mut statement_id = 0usize;
    remove_dead_store_nodes(statements, &dead_stores, &mut statement_id);
}

fn remove_dead_store_nodes(
    statements: &mut Vec<Statement>,
    dead_stores: &HashSet<usize>,
    statement_id: &mut usize,
) {
    for index in (0..statements.len()).rev() {
        let id = *statement_id;
        *statement_id += 1;

        if dead_stores.contains(&id) {
            statements.remove(index);
            continue;
        }

        match &mut statements[index] {
            Statement::Positioned { statement, .. }
            | Statement::Export { statement } => {
                remove_dead_store_in_statement(statement, dead_stores, statement_id);
            }
            Statement::Block(body) => {
                remove_dead_store_nodes(body, dead_stores, statement_id);
            }
            Statement::If {
                then_branch,
                else_branch,
                ..
            } => {
                remove_dead_store_nodes(then_branch, dead_stores, statement_id);
                if let Some(branch) = else_branch {
                    remove_dead_store_nodes(branch, dead_stores, statement_id);
                }
            }
            Statement::While { body, .. } | Statement::ForIn { body, .. } => {
                remove_dead_store_nodes(body, dead_stores, statement_id);
            }
            // Le CFG reste opaque pour `match`, `try` et les déclarations.
            Statement::Match { .. }
            | Statement::Try { .. }
            | Statement::Function { .. }
            | Statement::Class { .. }
            | Statement::Enum { .. }
            | Statement::Let { .. }
            | Statement::Assignment { .. }
            | Statement::Expression { .. }
            | Statement::Return { .. }
            | Statement::Throw { .. }
            | Statement::Import { .. }
            | Statement::FromImport { .. }
            | Statement::TypeAlias { .. }
            | Statement::Interface { .. }
            | Statement::Break
            | Statement::Continue => {}
        }
    }
}

fn remove_dead_store_in_statement(
    statement: &mut Box<Statement>,
    dead_stores: &HashSet<usize>,
    statement_id: &mut usize,
) {
    let id = *statement_id;
    if dead_stores.contains(&id) {
        // Unreachable for a wrapper node: a removable assignment is represented
        // by the wrapper's own id and is removed by the parent vector.
        return;
    }

    match statement.as_mut() {
        Statement::Positioned { statement, .. }
        | Statement::Export { statement } => {
            remove_dead_store_in_statement(statement, dead_stores, statement_id);
        }
        Statement::Block(body) => remove_dead_store_nodes(body, dead_stores, statement_id),
        Statement::If {
            then_branch,
            else_branch,
            ..
        } => {
            remove_dead_store_nodes(then_branch, dead_stores, statement_id);
            if let Some(branch) = else_branch {
                remove_dead_store_nodes(branch, dead_stores, statement_id);
            }
        }
        Statement::While { body, .. } | Statement::ForIn { body, .. } => {
            remove_dead_store_nodes(body, dead_stores, statement_id);
        }
        _ => {}
    }
}

type ConstantEnvironment = HashMap<String, Literal>;

fn eliminate_dead_values_in_nested_functions(statements: &mut [Statement]) {
    for statement in statements {
        match statement {
            Statement::Positioned { statement, .. } => {
                eliminate_dead_values_in_nested_functions(std::slice::from_mut(statement.as_mut()));
            }
            Statement::Function { body, .. } => {
                eliminate_dead_values_in_block(body);
            }
            Statement::Class { methods, .. } => {
                for method in methods {
                    eliminate_dead_values_in_block(&mut method.body);
                }
            }
            Statement::Enum { methods, .. } => {
                for method in methods {
                    eliminate_dead_values_in_block(&mut method.body);
                }
            }
            Statement::Export { statement } => {
                eliminate_dead_values_in_nested_functions(std::slice::from_mut(statement.as_mut()));
            }
            Statement::Let { .. }
            | Statement::Assignment { .. }
            | Statement::Expression { .. }
            | Statement::Block(_)
            | Statement::If { .. }
            | Statement::While { .. }
            | Statement::ForIn { .. }
            | Statement::Match { .. }
            | Statement::Throw { .. }
            | Statement::Try { .. }
            | Statement::Return { .. }
            | Statement::Import { .. }
            | Statement::FromImport { .. }
            | Statement::TypeAlias { .. }
            | Statement::Interface { .. }
            | Statement::Break
            | Statement::Continue => {}
        }
    }
}

fn eliminate_dead_values_in_block(statements: &mut Vec<Statement>) {
    for statement in statements.iter_mut() {
        recurse_dead_value_pass(statement);
    }

    let mut used = HashSet::new();
    for statement in statements.iter() {
        collect_used_variables(statement, &mut used);
    }

    let original = std::mem::take(statements);
    let mut transformed = Vec::with_capacity(original.len());

    for statement in original {
        if is_dead_literal_let(&statement, &used) {
            continue;
        }
        transformed.push(statement);
    }

    *statements = transformed;
}

fn recurse_dead_value_pass(statement: &mut Statement) {
    match statement {
        Statement::Positioned { statement, .. } => recurse_dead_value_pass(statement),
        Statement::Block(body) => eliminate_dead_values_in_block(body),
        Statement::If {
            then_branch,
            else_branch,
            ..
        } => {
            eliminate_dead_values_in_block(then_branch);
            if let Some(branch) = else_branch {
                eliminate_dead_values_in_block(branch);
            }
        }
        Statement::While { body, .. } | Statement::ForIn { body, .. } => {
            eliminate_dead_values_in_block(body);
        }
        Statement::Match { arms, .. } => {
            for arm in arms {
                eliminate_dead_values_in_block(&mut arm.body);
            }
        }
        Statement::Try {
            try_body,
            catch_body,
            finally_body,
            ..
        } => {
            eliminate_dead_values_in_block(try_body);
            if let Some(body) = catch_body {
                eliminate_dead_values_in_block(body);
            }
            if let Some(body) = finally_body {
                eliminate_dead_values_in_block(body);
            }
        }
        Statement::Function { body, .. } => eliminate_dead_values_in_block(body),
        Statement::Class { methods, .. } => {
            for method in methods {
                eliminate_dead_values_in_block(&mut method.body);
            }
        }
        Statement::Enum { methods, .. } => {
            for method in methods {
                eliminate_dead_values_in_block(&mut method.body);
            }
        }
        Statement::Export { statement } => recurse_dead_value_pass(statement),
        Statement::Let { .. }
        | Statement::Assignment { .. }
        | Statement::Expression { .. }
        | Statement::Throw { .. }
        | Statement::Return { .. }
        | Statement::Import { .. }
        | Statement::FromImport { .. }
        | Statement::TypeAlias { .. }
        | Statement::Interface { .. }
        | Statement::Break
        | Statement::Continue => {}
    }
}

fn is_dead_literal_let(statement: &Statement, used: &HashSet<String>) -> bool {
    let statement = match statement {
        Statement::Positioned { statement, .. } => statement.as_ref(),
        other => other,
    };

    matches!(
        statement,
        Statement::Let {
            name,
            value: Expression::Literal(_),
            mutable: false,
            ..
        } if !used.contains(name)
    )
}

fn collect_used_variables_statements(statements: &[Statement], used: &mut HashSet<String>) {
    for statement in statements {
        collect_used_variables(statement, used);
    }
}

fn collect_used_variables(statement: &Statement, used: &mut HashSet<String>) {
    match statement {
        Statement::Positioned { statement, .. } => collect_used_variables(statement, used),
        Statement::Let { value, .. } => collect_used_variables_expression(value, used),
        Statement::Assignment { target, value } => {
            collect_used_variables_target(target, used);
            collect_used_variables_expression(value, used);
        }
        Statement::Expression { expression } | Statement::Throw { value: expression } => {
            collect_used_variables_expression(expression, used);
        }
        Statement::Block(body) => {
            for statement in body {
                collect_used_variables(statement, used);
            }
        }
        Statement::If {
            condition,
            then_branch,
            else_branch,
        } => {
            collect_used_variables_expression(condition, used);
            for statement in then_branch {
                collect_used_variables(statement, used);
            }
            if let Some(branch) = else_branch {
                for statement in branch {
                    collect_used_variables(statement, used);
                }
            }
        }
        Statement::While { condition, body } => {
            collect_used_variables_expression(condition, used);
            for statement in body {
                collect_used_variables(statement, used);
            }
        }
        Statement::ForIn { iterable, body, .. } => {
            collect_used_variables_expression(iterable, used);
            for statement in body {
                collect_used_variables(statement, used);
            }
        }
        Statement::Match { value, arms } => {
            collect_used_variables_expression(value, used);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_used_variables_expression(guard, used);
                }
                for statement in &arm.body {
                    collect_used_variables(statement, used);
                }
            }
        }
        Statement::Try {
            try_body,
            catch_body,
            finally_body,
            ..
        } => {
            for statement in try_body {
                collect_used_variables(statement, used);
            }
            if let Some(body) = catch_body {
                for statement in body {
                    collect_used_variables(statement, used);
                }
            }
            if let Some(body) = finally_body {
                for statement in body {
                    collect_used_variables(statement, used);
                }
            }
        }
        Statement::Function { body, .. } => {
            for statement in body {
                collect_used_variables(statement, used);
            }
        }
        Statement::Return { value } => {
            if let Some(value) = value {
                collect_used_variables_expression(value, used);
            }
        }
        Statement::Export { statement } => collect_used_variables(statement, used),
        Statement::Class { fields, methods, .. } => {
            for field in fields {
                if let Some(initializer) = &field.initializer {
                    collect_used_variables_expression(initializer, used);
                }
            }
            for method in methods {
                for statement in &method.body {
                    collect_used_variables(statement, used);
                }
            }
        }
        Statement::Enum { methods, .. } => {
            for method in methods {
                for statement in &method.body {
                    collect_used_variables(statement, used);
                }
            }
        }
        Statement::Import { .. }
        | Statement::FromImport { .. }
        | Statement::TypeAlias { .. }
        | Statement::Interface { .. }
        | Statement::Break
        | Statement::Continue => {}
    }
}

fn collect_used_variables_target(target: &AssignmentTarget, used: &mut HashSet<String>) {
    match target {
        AssignmentTarget::Variable(_) => {}
        AssignmentTarget::Index { object, index } => {
            collect_used_variables_expression(object, used);
            collect_used_variables_expression(index, used);
        }
        AssignmentTarget::Member { object, .. } => collect_used_variables_expression(object, used),
    }
}

fn collect_used_variables_expression(expression: &Expression, used: &mut HashSet<String>) {
    match expression {
        Expression::Variable(name) => {
            used.insert(name.clone());
        }
        Expression::Unary { right, .. } => collect_used_variables_expression(right, used),
        Expression::Binary { left, right, .. } => {
            collect_used_variables_expression(left, used);
            collect_used_variables_expression(right, used);
        }
        Expression::Function { body, .. } => {
            for statement in body {
                collect_used_variables(statement, used);
            }
        }
        Expression::Call { callee, arguments, .. } => {
            collect_used_variables_expression(callee, used);
            for argument in arguments {
                collect_used_variables_expression(argument, used);
            }
        }
        Expression::Member { object, .. } => collect_used_variables_expression(object, used),
        Expression::Index { object, index, .. } => {
            collect_used_variables_expression(object, used);
            collect_used_variables_expression(index, used);
        }
        Expression::New { arguments, .. } => {
            for argument in arguments {
                collect_used_variables_expression(argument, used);
            }
        }
        Expression::Array(elements) | Expression::Tuple(elements) => {
            for element in elements {
                collect_used_variables_expression(element, used);
            }
        }
        Expression::Dict(fields) | Expression::Record(fields) => {
            for (_, value) in fields {
                collect_used_variables_expression(value, used);
            }
        }
        Expression::Try(expression) | Expression::Await(expression) => {
            collect_used_variables_expression(expression, used);
        }
        Expression::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            collect_used_variables_expression(condition, used);
            collect_used_variables_expression(then_expr, used);
            collect_used_variables_expression(else_expr, used);
        }
        Expression::Literal(_) | Expression::SelfValue => {}
    }
}


fn propagate_block(statements: &mut Vec<Statement>, environment: &mut ConstantEnvironment) {
    let original = std::mem::take(statements);
    let mut transformed = Vec::with_capacity(original.len());

    for mut statement in original {
        propagate_statement(&mut statement, environment, &mut transformed);
    }

    *statements = transformed;
}

/// Réduit un bloc à sa partie atteignable.
fn eliminate_block(statements: &mut Vec<Statement>) {
    let original = std::mem::take(statements);
    let mut reachable = Vec::with_capacity(original.len());

    for mut statement in original {
        eliminate_statement(&mut statement);
        let terminates = statement_terminates(&statement);
        reachable.push(statement);

        if terminates {
            break;
        }
    }

    *statements = reachable;
}

fn eliminate_statement(statement: &mut Statement) {
    match statement {
        Statement::Positioned { statement, .. } => eliminate_statement(statement),

        Statement::Let { value, .. } => eliminate_expression(value),
        Statement::Assignment { target, value } => {
            eliminate_assignment_target(target);
            eliminate_expression(value);
        }
        Statement::Expression { expression } | Statement::Throw { value: expression } => {
            eliminate_expression(expression);
        }
        Statement::Block(statements) => eliminate_block(statements),

        Statement::If {
            condition,
            then_branch,
            else_branch,
        } => {
            eliminate_expression(condition);
            eliminate_block(then_branch);
            if let Some(branch) = else_branch {
                eliminate_block(branch);
            }
        }

        Statement::While { condition, body } => {
            eliminate_expression(condition);
            eliminate_block(body);
        }

        Statement::ForIn { iterable, body, .. } => {
            eliminate_expression(iterable);
            eliminate_block(body);
        }

        Statement::Match { value, arms } => {
            eliminate_expression(value);
            for arm in arms {
                if let Some(guard) = &mut arm.guard {
                    eliminate_expression(guard);
                }
                eliminate_block(&mut arm.body);
            }
        }

        Statement::Try {
            try_body,
            catch_body,
            finally_body,
            ..
        } => {
            eliminate_block(try_body);
            if let Some(body) = catch_body {
                eliminate_block(body);
            }
            if let Some(body) = finally_body {
                eliminate_block(body);
            }
        }

        Statement::Function { body, .. } => eliminate_block(body),

        Statement::Return { value } => {
            if let Some(value) = value {
                eliminate_expression(value);
            }
        }

        Statement::Export { statement } => eliminate_statement(statement),

        Statement::Class { fields, methods, .. } => {
            for field in fields {
                if let Some(initializer) = &mut field.initializer {
                    eliminate_expression(initializer);
                }
            }
            for method in methods {
                eliminate_block(&mut method.body);
            }
        }

        Statement::Enum { methods, .. } => {
            for method in methods {
                eliminate_block(&mut method.body);
            }
        }

        Statement::Import { .. }
        | Statement::FromImport { .. }
        | Statement::TypeAlias { .. }
        | Statement::Interface { .. }
        | Statement::Break
        | Statement::Continue => {}
    }
}

fn statement_terminates(statement: &Statement) -> bool {
    match statement {
        Statement::Positioned { statement, .. } => statement_terminates(statement),
        Statement::Return { .. } | Statement::Throw { .. } | Statement::Break | Statement::Continue => true,
        Statement::Block(statements) => statements.last().is_some_and(statement_terminates),
        Statement::If {
            then_branch,
            else_branch: Some(else_branch),
            ..
        } => {
            statements_terminate(then_branch) && statements_terminate(else_branch)
        }
        Statement::While {
            condition: Expression::Literal(Literal::Bool(true)),
            body,
        } => loop_body_terminates(body),
        Statement::Export { statement } => statement_terminates(statement),
        _ => false,
    }
}

fn statements_terminate(statements: &[Statement]) -> bool {
    statements.last().is_some_and(statement_terminates)
}

/// Vérifie qu'une boucle infinie ne peut pas atteindre son point de sortie
/// normal. `break` et `continue` ne comptent donc pas comme terminaison :
/// ils quittent ou poursuivent la boucle au lieu de quitter la fonction.
fn loop_body_terminates(statements: &[Statement]) -> bool {
    let Some(statement) = statements.last() else {
        return false;
    };

    match statement {
        Statement::Positioned { statement, .. } => loop_statement_terminates(statement),
        Statement::Return { .. } | Statement::Throw { .. } => true,
        Statement::Block(statements) => loop_body_terminates(statements),
        Statement::If {
            then_branch,
            else_branch: Some(else_branch),
            ..
        } => loop_body_terminates(then_branch) && loop_body_terminates(else_branch),
        Statement::Export { statement } => loop_statement_terminates(statement),
        _ => false,
    }
}

fn loop_statement_terminates(statement: &Statement) -> bool {
    match statement {
        Statement::Positioned { statement, .. } => loop_statement_terminates(statement),
        Statement::Return { .. } | Statement::Throw { .. } => true,
        Statement::Block(statements) => loop_body_terminates(statements),
        Statement::If {
            then_branch,
            else_branch: Some(else_branch),
            ..
        } => loop_body_terminates(then_branch) && loop_body_terminates(else_branch),
        Statement::Export { statement } => loop_statement_terminates(statement),
        _ => false,
    }
}

fn eliminate_assignment_target(target: &mut AssignmentTarget) {
    match target {
        AssignmentTarget::Variable(_) => {}
        AssignmentTarget::Index { object, index } => {
            eliminate_expression(object);
            eliminate_expression(index);
        }
        AssignmentTarget::Member { object, .. } => eliminate_expression(object),
    }
}

fn eliminate_expression(expression: &mut Expression) {
    match expression {
        Expression::Literal(_) | Expression::Variable(_) | Expression::SelfValue => {}

        Expression::Unary { right, .. } => eliminate_expression(right),
        Expression::Binary { left, right, .. } => {
            eliminate_expression(left);
            eliminate_expression(right);
        }
        Expression::Function { body, .. } => eliminate_block(body),
        Expression::Call { callee, arguments, .. } => {
            eliminate_expression(callee);
            for argument in arguments {
                eliminate_expression(argument);
            }
        }
        Expression::Member { object, .. } => eliminate_expression(object),
        Expression::Index { object, index, .. } => {
            eliminate_expression(object);
            eliminate_expression(index);
        }
        Expression::New { arguments, .. } => {
            for argument in arguments {
                eliminate_expression(argument);
            }
        }
        Expression::Array(elements) | Expression::Tuple(elements) => {
            for element in elements {
                eliminate_expression(element);
            }
        }
        Expression::Dict(fields) | Expression::Record(fields) => {
            for (_, value) in fields {
                eliminate_expression(value);
            }
        }
        Expression::Try(expression) | Expression::Await(expression) => eliminate_expression(expression),
        Expression::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            eliminate_expression(condition);
            eliminate_expression(then_expr);
            eliminate_expression(else_expr);
        }
    }
}

fn propagate_statement(
    statement: &mut Statement,
    environment: &mut ConstantEnvironment,
    output: &mut Vec<Statement>,
) {
    match statement {
        Statement::Positioned {
            line,
            column,
            statement: inner,
        } => {
            let mut nested_output = Vec::new();
            propagate_statement(inner, environment, &mut nested_output);

            if nested_output.len() == 1 {
                output.push(Statement::Positioned {
                    line: *line,
                    column: *column,
                    statement: Box::new(nested_output.remove(0)),
                });
            } else {
                output.extend(nested_output);
            }
        }

        Statement::Let {
            name,
            value,
            mutable,
            ..
        } => {
            propagate_expression(value, environment);

            if !*mutable {
                if let Expression::Literal(literal) = value {
                    environment.insert(name.clone(), literal.clone());
                } else {
                    environment.remove(name);
                }
            } else {
                environment.remove(name);
            }

            output.push(statement.clone());
        }

        Statement::Assignment { target, value } => {
            propagate_assignment_target(target, environment);
            propagate_expression(value, environment);

            if let AssignmentTarget::Variable(name) = target {
                environment.remove(name);
            }

            output.push(statement.clone());
        }

        Statement::Expression { expression } | Statement::Throw { value: expression } => {
            propagate_expression(expression, environment);
            output.push(statement.clone());
        }

        Statement::Return { value } => {
            if let Some(value) = value {
                propagate_expression(value, environment);
            }
            output.push(statement.clone());
        }

        Statement::Block(body) => {
            let mut nested_environment = environment.clone();
            propagate_block(body, &mut nested_environment);
            output.push(statement.clone());
        }

        Statement::If {
            condition,
            then_branch,
            else_branch,
        } => {
            propagate_expression(condition, environment);

            let incoming = environment.clone();
            let mut then_environment = incoming.clone();
            propagate_block(then_branch, &mut then_environment);

            let mut else_environment = incoming.clone();
            if let Some(branch) = else_branch {
                propagate_block(branch, &mut else_environment);
            }

            if let Expression::Literal(literal) = condition {
                let selected = if literal_truthy(literal) {
                    std::mem::take(then_branch)
                } else {
                    else_branch.take().unwrap_or_default()
                };

                // Une branche `if` possède sa propre portée. On conserve
                // donc le bloc lors de la simplification et on ne laisse pas
                // ses liaisons locales contaminer l'environnement extérieur.
                *environment = incoming;
                output.push(Statement::Block(selected));
            } else {
                *environment = merge_environments(&incoming, &then_environment, &else_environment, else_branch.is_some());
                output.push(statement.clone());
            }
        }

        Statement::While { condition, body } => {
            propagate_expression(condition, environment);

            let mut body_environment = environment.clone();
            propagate_block(body, &mut body_environment);

            if matches!(condition, Expression::Literal(literal) if !literal_truthy(literal)) {
                return;
            }

            output.push(statement.clone());
        }

        Statement::ForIn { variable, iterable, body } => {
            propagate_expression(iterable, environment);

            let mut body_environment = environment.clone();
            body_environment.remove(variable);
            propagate_block(body, &mut body_environment);

            output.push(statement.clone());
        }

        Statement::Match { value, arms } => {
            propagate_expression(value, environment);
            for arm in arms {
                let mut arm_environment = environment.clone();
                remove_pattern_bindings(&arm.pattern, &mut arm_environment);
                if let Some(guard) = &mut arm.guard {
                    propagate_expression(guard, &arm_environment);
                }
                propagate_block(&mut arm.body, &mut arm_environment);
            }
            output.push(statement.clone());
        }

        Statement::Try {
            try_body,
            catch_name,
            catch_body,
            finally_body,
            ..
        } => {
            let mut try_environment = environment.clone();
            propagate_block(try_body, &mut try_environment);

            let mut catch_environment = environment.clone();
            if let Some(name) = catch_name {
                catch_environment.remove(name);
            }
            if let Some(body) = catch_body {
                propagate_block(body, &mut catch_environment);
            }

            let mut finally_environment = environment.clone();
            if let Some(body) = finally_body {
                propagate_block(body, &mut finally_environment);
            }

            output.push(statement.clone());
        }

        Statement::Function { body, params, .. } => {
            let mut function_environment = ConstantEnvironment::new();
            for parameter in params {
                function_environment.remove(parameter);
            }
            propagate_block(body, &mut function_environment);
            output.push(statement.clone());
        }

        Statement::Export { statement: inner } => {
            let mut nested_output = Vec::new();
            let mut nested_environment = environment.clone();
            propagate_statement(inner, &mut nested_environment, &mut nested_output);
            if nested_output.len() == 1 {
                *inner = Box::new(nested_output.remove(0));
            }
            output.push(statement.clone());
        }

        Statement::Class { fields, methods, .. } => {
            for field in fields {
                if let Some(initializer) = &mut field.initializer {
                    let isolated_environment = ConstantEnvironment::new();
                    propagate_expression(initializer, &isolated_environment);
                }
            }
            for method in methods {
                let mut method_environment = ConstantEnvironment::new();
                for parameter in &method.params {
                    method_environment.remove(parameter);
                }
                propagate_block(&mut method.body, &mut method_environment);
            }
            output.push(statement.clone());
        }

        Statement::Enum { methods, .. } => {
            for method in methods {
                let mut method_environment = ConstantEnvironment::new();
                for parameter in &method.params {
                    method_environment.remove(parameter);
                }
                propagate_block(&mut method.body, &mut method_environment);
            }
            output.push(statement.clone());
        }

        Statement::Import { .. }
        | Statement::FromImport { .. }
        | Statement::TypeAlias { .. }
        | Statement::Interface { .. }
        | Statement::Break
        | Statement::Continue => {
            output.push(statement.clone());
        }
    }
}

fn merge_environments(
    incoming: &ConstantEnvironment,
    then_environment: &ConstantEnvironment,
    else_environment: &ConstantEnvironment,
    has_else: bool,
) -> ConstantEnvironment {
    let mut merged = ConstantEnvironment::new();

    for (name, value) in incoming {
        let same_then = then_environment.get(name).is_some_and(|candidate| literal_equals(value, candidate));
        let same_else = if has_else {
            else_environment.get(name).is_some_and(|candidate| literal_equals(value, candidate))
        } else {
            true
        };

        if same_then && same_else {
            merged.insert(name.clone(), value.clone());
        }
    }

    merged
}

fn propagate_assignment_target(
    target: &mut AssignmentTarget,
    environment: &ConstantEnvironment,
) {
    match target {
        AssignmentTarget::Variable(_) => {}
        AssignmentTarget::Index { object, index } => {
            propagate_expression(object, environment);
            propagate_expression(index, environment);
        }
        AssignmentTarget::Member { object, .. } => {
            propagate_expression(object, environment);
        }
    }
}

fn remove_pattern_bindings(pattern: &super::him::Pattern, environment: &mut ConstantEnvironment) {
    match pattern {
        super::him::Pattern::Binding(name) => {
            environment.remove(name);
        }
        super::him::Pattern::Or(patterns)
        | super::him::Pattern::Array(patterns)
        | super::him::Pattern::ArrayRest(patterns)
        | super::him::Pattern::Tuple(patterns) => {
            for pattern in patterns {
                remove_pattern_bindings(pattern, environment);
            }
        }
        super::him::Pattern::Range { start, end, .. } => {
            remove_pattern_bindings(start, environment);
            remove_pattern_bindings(end, environment);
        }
        super::him::Pattern::OptionSome(pattern)
        | super::him::Pattern::ResultOk(pattern)
        | super::him::Pattern::ResultErr(pattern) => {
            remove_pattern_bindings(pattern, environment);
        }
        super::him::Pattern::Wildcard
        | super::him::Pattern::Literal(_)
        | super::him::Pattern::EnumVariant { .. } => {}
    }
}

fn propagate_expression(
    expression: &mut Expression,
    environment: &ConstantEnvironment,
) {
    match expression {
        Expression::Variable(name) => {
            if let Some(value) = environment.get(name) {
                *expression = Expression::Literal(value.clone());
            }
        }

        Expression::Unary { right, .. } => {
            propagate_expression(right, environment);
            fold_expression(expression);
        }

        Expression::Binary { left, right, .. } => {
            propagate_expression(left, environment);
            propagate_expression(right, environment);
            fold_expression(expression);
        }

        Expression::Function { body, params } => {
            let mut isolated_environment = ConstantEnvironment::new();
            for parameter in params {
                isolated_environment.remove(parameter);
            }
            propagate_block(body, &mut isolated_environment);
        }

        Expression::Call { callee, arguments, .. } => {
            propagate_expression(callee, environment);
            for argument in arguments {
                propagate_expression(argument, environment);
            }
        }

        Expression::Member { object, .. } => {
            propagate_expression(object, environment);
        }

        Expression::Index { object, index, .. } => {
            propagate_expression(object, environment);
            propagate_expression(index, environment);
        }

        Expression::New { arguments, .. } => {
            for argument in arguments {
                propagate_expression(argument, environment);
            }
        }

        Expression::Array(elements) | Expression::Tuple(elements) => {
            for element in elements {
                propagate_expression(element, environment);
            }
        }

        Expression::Dict(fields) | Expression::Record(fields) => {
            for (_, value) in fields {
                propagate_expression(value, environment);
            }
        }

        Expression::Try(expression) | Expression::Await(expression) => {
            propagate_expression(expression, environment);
        }

        Expression::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            propagate_expression(condition, environment);
            propagate_expression(then_expr, environment);
            propagate_expression(else_expr, environment);
            fold_expression(expression);
        }

        Expression::Literal(_) | Expression::SelfValue => {}
    }
}

fn fold_statement(statement: &mut Statement) {
    match statement {
        Statement::Positioned { statement, .. } => fold_statement(statement),

        Statement::Let { value, .. } => fold_expression(value),
        Statement::Assignment { target, value } => {
            fold_assignment_target(target);
            fold_expression(value);
        }
        Statement::Expression { expression } | Statement::Throw { value: expression } => {
            fold_expression(expression);
        }

        Statement::Block(statements) => {
            for statement in statements {
                fold_statement(statement);
            }
        }

        Statement::If {
            condition,
            then_branch,
            else_branch,
        } => {
            fold_expression(condition);
            for statement in then_branch {
                fold_statement(statement);
            }
            if let Some(branch) = else_branch {
                for statement in branch {
                    fold_statement(statement);
                }
            }
        }

        Statement::While { condition, body } => {
            fold_expression(condition);
            for statement in body {
                fold_statement(statement);
            }
        }

        Statement::ForIn { iterable, body, .. } => {
            fold_expression(iterable);
            for statement in body {
                fold_statement(statement);
            }
        }

        Statement::Match { value, arms } => {
            fold_expression(value);
            for arm in arms {
                fold_match_arm(arm);
            }
        }

        Statement::Try {
            try_body,
            catch_body,
            finally_body,
            ..
        } => {
            for statement in try_body {
                fold_statement(statement);
            }
            if let Some(body) = catch_body {
                for statement in body {
                    fold_statement(statement);
                }
            }
            if let Some(body) = finally_body {
                for statement in body {
                    fold_statement(statement);
                }
            }
        }

        Statement::Function { body, .. } => {
            for statement in body {
                fold_statement(statement);
            }
        }

        Statement::Return { value } => {
            if let Some(value) = value {
                fold_expression(value);
            }
        }

        Statement::Export { statement } => fold_statement(statement),

        Statement::Class { fields, methods, .. } => {
            for field in fields {
                if let Some(initializer) = &mut field.initializer {
                    fold_expression(initializer);
                }
            }
            for method in methods {
                for statement in &mut method.body {
                    fold_statement(statement);
                }
            }
        }

        Statement::Enum { methods, .. } => {
            for method in methods {
                for statement in &mut method.body {
                    fold_statement(statement);
                }
            }
        }

        Statement::Import { .. }
        | Statement::FromImport { .. }
        | Statement::TypeAlias { .. }
        | Statement::Interface { .. }
        | Statement::Break
        | Statement::Continue => {}
    }
}

fn fold_assignment_target(target: &mut AssignmentTarget) {
    match target {
        AssignmentTarget::Variable(_) => {}
        AssignmentTarget::Index { object, index } => {
            fold_expression(object);
            fold_expression(index);
        }
        AssignmentTarget::Member { object, .. } => fold_expression(object),
    }
}

fn fold_match_arm(arm: &mut MatchArm) {
    if let Some(guard) = &mut arm.guard {
        fold_expression(guard);
    }
    for statement in &mut arm.body {
        fold_statement(statement);
    }
}

fn fold_expression(expression: &mut Expression) {
    match expression {
        Expression::Literal(_)
        | Expression::Variable(_)
        | Expression::SelfValue => {}

        Expression::Unary { operator, right, .. } => {
            fold_expression(right);

            let replacement = match right.as_ref() {
                Expression::Literal(literal) => fold_unary(*operator, literal),
                _ => None,
            };

            if let Some(result) = replacement {
                *expression = Expression::Literal(result);
            }
        }

        Expression::Binary {
            left,
            operator,
            right,
            ..
        } => {
            fold_expression(left);
            fold_expression(right);

            let replacement = match (left.as_ref(), right.as_ref()) {
                (Expression::Literal(left), Expression::Literal(right)) => {
                    fold_binary(*operator, left, right)
                }
                _ => None,
            };

            if let Some(result) = replacement {
                *expression = Expression::Literal(result);
                return;
            }

            // `&&` / `||` renvoient l'un de leurs opérandes. Lorsque gauche
            // est constant et décide déjà le résultat, droite peut être
            // éliminée sans changer l'ordre d'évaluation.
            let replacement = match (operator, left.as_ref()) {
                (BinaryOp::And, Expression::Literal(left)) if !literal_truthy(left) => {
                    Some(left.clone())
                }
                (BinaryOp::Or, Expression::Literal(left)) if literal_truthy(left) => {
                    Some(left.clone())
                }
                _ => None,
            };

            if let Some(result) = replacement {
                *expression = Expression::Literal(result);
            }
        }

        Expression::Function { body, .. } => {
            for statement in body {
                fold_statement(statement);
            }
        }

        Expression::Call {
            callee, arguments, ..
        } => {
            fold_expression(callee);
            for argument in arguments {
                fold_expression(argument);
            }
        }

        Expression::Member { object, .. } => fold_expression(object),

        Expression::Index { object, index, .. } => {
            fold_expression(object);
            fold_expression(index);
        }

        Expression::New { arguments, .. } => {
            for argument in arguments {
                fold_expression(argument);
            }
        }

        Expression::Array(elements) | Expression::Tuple(elements) => {
            for element in elements {
                fold_expression(element);
            }
        }

        Expression::Dict(fields) | Expression::Record(fields) => {
            for (_, value) in fields {
                fold_expression(value);
            }
        }

        Expression::Try(expression) | Expression::Await(expression) => {
            fold_expression(expression);
        }

        Expression::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            fold_expression(condition);
            fold_expression(then_expr);
            fold_expression(else_expr);

            let replacement = match condition.as_ref() {
                Expression::Literal(condition) if literal_truthy(condition) => {
                    Some((**then_expr).clone())
                }
                Expression::Literal(_) => Some((**else_expr).clone()),
                _ => None,
            };

            if let Some(result) = replacement {
                *expression = result;
            }
        }
    }
}

fn fold_unary(operator: super::him::UnaryOp, literal: &Literal) -> Option<Literal> {
    match operator {
        super::him::UnaryOp::Negate => match literal {
            Literal::Integer(value) => value.checked_neg().map(Literal::Integer),
            Literal::Float(value) => Some(Literal::Float(-value)),
            _ => None,
        },
        super::him::UnaryOp::Not => Some(Literal::Bool(!literal_truthy(literal))),
        super::him::UnaryOp::BitNot => match literal {
            Literal::Integer(value) => Some(Literal::Integer(!value)),
            _ => None,
        },
    }
}

fn fold_binary(operator: BinaryOp, left: &Literal, right: &Literal) -> Option<Literal> {
    match operator {
        BinaryOp::Add => fold_add(left, right),
        BinaryOp::Subtract => fold_numeric(left, right, NumericFold::Subtract),
        BinaryOp::Multiply => fold_numeric(left, right, NumericFold::Multiply),
        BinaryOp::Divide => fold_divide(left, right),
        BinaryOp::Modulo => fold_modulo(left, right),

        BinaryOp::Equal => Some(Literal::Bool(literal_equals(left, right))),
        BinaryOp::NotEqual => Some(Literal::Bool(!literal_equals(left, right))),

        BinaryOp::Less => fold_compare(left, right, |ordering| ordering == Ordering::Less),
        BinaryOp::LessEqual => fold_compare(left, right, |ordering| ordering != Ordering::Greater),
        BinaryOp::Greater => {
            fold_compare(left, right, |ordering| ordering == Ordering::Greater)
        }
        BinaryOp::GreaterEqual => fold_compare(left, right, |ordering| ordering != Ordering::Less),

        BinaryOp::And => {
            if literal_truthy(left) {
                Some(right.clone())
            } else {
                Some(left.clone())
            }
        }
        BinaryOp::Or => {
            if literal_truthy(left) {
                Some(left.clone())
            } else {
                Some(right.clone())
            }
        }

        BinaryOp::BitAnd => fold_bitwise(left, right, |a, b| a & b),
        BinaryOp::BitOr => fold_bitwise(left, right, |a, b| a | b),
        BinaryOp::BitXor => fold_bitwise(left, right, |a, b| a ^ b),
        BinaryOp::ShiftLeft => fold_shift(left, right, true),
        BinaryOp::ShiftRight => fold_shift(left, right, false),

        // `is` dépend d'une valeur runtime de classe/interface/type.
        BinaryOp::Is => None,
    }
}

fn fold_add(left: &Literal, right: &Literal) -> Option<Literal> {
    match (left, right) {
        (Literal::String(a), Literal::String(b)) => Some(Literal::String(format!("{a}{b}"))),
        _ => fold_numeric(left, right, NumericFold::Add),
    }
}

#[derive(Clone, Copy)]
enum NumericFold {
    Add,
    Subtract,
    Multiply,
}

fn fold_numeric(left: &Literal, right: &Literal, operation: NumericFold) -> Option<Literal> {
    match (left, right) {
        (Literal::Integer(a), Literal::Integer(b)) => {
            let result = match operation {
                NumericFold::Add => a.checked_add(*b),
                NumericFold::Subtract => a.checked_sub(*b),
                NumericFold::Multiply => a.checked_mul(*b),
            }?;
            Some(Literal::Integer(result))
        }

        (Literal::Integer(a), Literal::Float(b)) => {
            Some(Literal::Float(apply_float(*a as f64, *b, operation)))
        }
        (Literal::Float(a), Literal::Integer(b)) => {
            Some(Literal::Float(apply_float(*a, *b as f64, operation)))
        }
        (Literal::Float(a), Literal::Float(b)) => {
            Some(Literal::Float(apply_float(*a, *b, operation)))
        }

        _ => None,
    }
}

fn apply_float(a: f64, b: f64, operation: NumericFold) -> f64 {
    match operation {
        NumericFold::Add => a + b,
        NumericFold::Subtract => a - b,
        NumericFold::Multiply => a * b,
    }
}

fn fold_divide(left: &Literal, right: &Literal) -> Option<Literal> {
    match (left, right) {
        (Literal::Integer(a), Literal::Integer(b)) if *b != 0 => {
            Some(Literal::Float(*a as f64 / *b as f64))
        }
        (Literal::Integer(a), Literal::Float(b)) if *b != 0.0 => {
            Some(Literal::Float(*a as f64 / *b))
        }
        (Literal::Float(a), Literal::Integer(b)) if *b != 0 => {
            Some(Literal::Float(*a / *b as f64))
        }
        (Literal::Float(a), Literal::Float(b)) if *b != 0.0 => {
            Some(Literal::Float(*a / *b))
        }
        _ => None,
    }
}

fn fold_modulo(left: &Literal, right: &Literal) -> Option<Literal> {
    match (left, right) {
        (Literal::Integer(a), Literal::Integer(b)) if *b != 0 => {
            if *a == i64::MIN && *b == -1 {
                Some(Literal::Integer(0))
            } else {
                Some(Literal::Integer(*a % *b))
            }
        }
        (Literal::Integer(a), Literal::Float(b)) if *b != 0.0 => {
            Some(Literal::Float((*a as f64) % *b))
        }
        (Literal::Float(a), Literal::Integer(b)) if *b != 0 => {
            Some(Literal::Float(*a % *b as f64))
        }
        (Literal::Float(a), Literal::Float(b)) if *b != 0.0 => {
            Some(Literal::Float(*a % *b))
        }
        _ => None,
    }
}

fn fold_bitwise(
    left: &Literal,
    right: &Literal,
    operation: impl FnOnce(i64, i64) -> i64,
) -> Option<Literal> {
    match (left, right) {
        (Literal::Integer(a), Literal::Integer(b)) => {
            Some(Literal::Integer(operation(*a, *b)))
        }
        _ => None,
    }
}

fn fold_shift(left: &Literal, right: &Literal, left_shift: bool) -> Option<Literal> {
    let (Literal::Integer(value), Literal::Integer(amount)) = (left, right) else {
        return None;
    };

    if !(0..64).contains(amount) {
        // Le VM produira InvalidShiftAmount à l'exécution.
        return None;
    }

    let amount = *amount as u32;
    let result = if left_shift {
        value.wrapping_shl(amount)
    } else {
        value.wrapping_shr(amount)
    };

    Some(Literal::Integer(result))
}

fn fold_compare(
    left: &Literal,
    right: &Literal,
    predicate: impl FnOnce(Ordering) -> bool,
) -> Option<Literal> {
    let ordering = numeric_ordering(left, right)?;
    Some(Literal::Bool(ordering.map(predicate).unwrap_or(false)))
}

fn numeric_ordering(left: &Literal, right: &Literal) -> Option<Option<Ordering>> {
    match (left, right) {
        (Literal::Integer(a), Literal::Integer(b)) => Some(Some(a.cmp(b))),
        (Literal::Float(a), Literal::Float(b)) => Some(a.partial_cmp(b)),
        (Literal::Integer(a), Literal::Float(b)) => Some(compare_integer_float(*a, *b)),
        (Literal::Float(a), Literal::Integer(b)) => {
            Some(compare_integer_float(*b, *a).map(Ordering::reverse))
        }
        _ => None,
    }
}

/// Même comparaison exacte entier/flottant que `Value::compare_integer_float`.
fn compare_integer_float(integer: i64, float: f64) -> Option<Ordering> {
    const I64_MAX_EXCLUSIVE: f64 = 9_223_372_036_854_775_808.0;

    if float.is_nan() {
        return None;
    }

    if float >= I64_MAX_EXCLUSIVE {
        return Some(Ordering::Less);
    }

    if float < -I64_MAX_EXCLUSIVE {
        return Some(Ordering::Greater);
    }

    let truncated = float.trunc();

    match integer.cmp(&(truncated as i64)) {
        Ordering::Equal => {
            let fraction = float - truncated;
            if fraction > 0.0 {
                Some(Ordering::Less)
            } else if fraction < 0.0 {
                Some(Ordering::Greater)
            } else {
                Some(Ordering::Equal)
            }
        }
        other => Some(other),
    }
}

fn literal_equals(left: &Literal, right: &Literal) -> bool {
    match (left, right) {
        (Literal::Integer(a), Literal::Integer(b)) => a == b,
        (Literal::Integer(a), Literal::Float(b)) => integer_equals_float(*a, *b),
        (Literal::Float(a), Literal::Integer(b)) => integer_equals_float(*b, *a),
        (Literal::Float(a), Literal::Float(b)) => a == b,
        (Literal::String(a), Literal::String(b)) => a == b,
        (Literal::Bool(a), Literal::Bool(b)) => a == b,
        (Literal::None, Literal::None) => true,
        _ => false,
    }
}

fn integer_equals_float(integer: i64, float: f64) -> bool {
    const I64_MAX_EXCLUSIVE: f64 = 9_223_372_036_854_775_808.0;

    float.fract() == 0.0
        && float >= i64::MIN as f64
        && float < I64_MAX_EXCLUSIVE
        && (float as i64) == integer
}

fn literal_truthy(literal: &Literal) -> bool {
    match literal {
        Literal::None => false,
        Literal::Bool(value) => *value,
        Literal::Integer(value) => *value != 0,
        Literal::Float(value) => *value != 0.0 && !value.is_nan(),
        Literal::String(value) => !value.is_empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn literal(expression: Expression) -> Literal {
        match expression {
            Expression::Literal(value) => value,
            other => panic!("literal attendue, reçu: {other:?}"),
        }
    }

    #[test]
    fn folds_nested_integer_arithmetic() {
        let mut expression = Expression::Binary {
            left: Box::new(Expression::Literal(Literal::Integer(2))),
            operator: BinaryOp::Multiply,
            right: Box::new(Expression::Binary {
                left: Box::new(Expression::Literal(Literal::Integer(3))),
                operator: BinaryOp::Add,
                right: Box::new(Expression::Literal(Literal::Integer(4))),
                line: 1,
                column: 5,
            }),
            line: 1,
            column: 1,
        };

        fold_expression(&mut expression);

        assert!(matches!(expression, Expression::Literal(Literal::Integer(14))));
    }

    #[test]
    fn folds_string_concatenation() {
        let result = fold_binary(
            BinaryOp::Add,
            &Literal::String("Ka".into()),
            &Literal::String("stel".into()),
        )
        .expect("concaténation attendue");

        assert_eq!(literal(Expression::Literal(result)), Literal::String("Kastel".into()));
    }

    #[test]
    fn never_folds_division_by_zero() {
        assert!(fold_binary(
            BinaryOp::Divide,
            &Literal::Integer(10),
            &Literal::Integer(0),
        )
        .is_none());
    }

    #[test]
    fn folds_logical_literals_with_operand_semantics() {
        let and_result = fold_binary(
            BinaryOp::And,
            &Literal::Integer(0),
            &Literal::String("non atteint".into()),
        )
        .expect("résultat constant attendu");
        assert_eq!(and_result, Literal::Integer(0));

        let or_result = fold_binary(
            BinaryOp::Or,
            &Literal::Integer(1),
            &Literal::String("non atteint".into()),
        )
        .expect("résultat constant attendu");
        assert_eq!(or_result, Literal::Integer(1));
    }

    #[test]
    fn folds_exact_integer_float_equality() {
        assert!(literal_equals(
            &Literal::Integer(9_007_199_254_740_992),
            &Literal::Float(9_007_199_254_740_992.0),
        ));
        assert!(!literal_equals(
            &Literal::Integer(9_007_199_254_740_993),
            &Literal::Float(9_007_199_254_740_992.0),
        ));
    }

    #[test]
    fn folds_nan_comparisons_to_false() {
        let result = fold_binary(
            BinaryOp::LessEqual,
            &Literal::Float(f64::NAN),
            &Literal::Float(1.0),
        )
        .expect("comparaison numérique attendue");

        assert_eq!(result, Literal::Bool(false));
    }

    #[test]
    fn folds_ternary_after_folding_condition() {
        let mut expression = Expression::Ternary {
            condition: Box::new(Expression::Binary {
                left: Box::new(Expression::Literal(Literal::Integer(2))),
                operator: BinaryOp::Less,
                right: Box::new(Expression::Literal(Literal::Integer(3))),
                line: 1,
                column: 1,
            }),
            then_expr: Box::new(Expression::Literal(Literal::String("yes".into()))),
            else_expr: Box::new(Expression::Literal(Literal::String("no".into()))),
        };

        fold_expression(&mut expression);

        assert_eq!(literal(expression), Literal::String("yes".into()));
    }

    #[test]
    fn propagates_immutable_literal_bindings() {
        let mut statements = vec![
            Statement::Let {
                name: "x".into(),
                value: Expression::Literal(Literal::Integer(21)),
                mutable: false,
                type_annotation: None,
            },
            Statement::Let {
                name: "y".into(),
                value: Expression::Binary {
                    left: Box::new(Expression::Variable("x".into())),
                    operator: BinaryOp::Multiply,
                    right: Box::new(Expression::Literal(Literal::Integer(2))),
                    line: 1,
                    column: 1,
                },
                mutable: false,
                type_annotation: None,
            },
        ];

        ConstantPropagator.run(&mut statements);

        match &statements[1] {
            Statement::Let { value, .. } => {
                assert_eq!(value_literal(value), Literal::Integer(42));
            }
            other => panic!("let attendu, reçu: {other:?}"),
        }
    }

    #[test]
    fn does_not_propagate_mutable_bindings() {
        let mut statements = vec![
            Statement::Let {
                name: "x".into(),
                value: Expression::Literal(Literal::Integer(21)),
                mutable: true,
                type_annotation: None,
            },
            Statement::Let {
                name: "y".into(),
                value: Expression::Variable("x".into()),
                mutable: false,
                type_annotation: None,
            },
        ];

        ConstantPropagator.run(&mut statements);

        match &statements[1] {
            Statement::Let { value, .. } => {
                assert!(matches!(value, Expression::Variable(name) if name == "x"));
            }
            other => panic!("let attendu, reçu: {other:?}"),
        }
    }

    #[test]
    fn simplifies_constant_if_and_keeps_selected_branch() {
        let mut statements = vec![Statement::If {
            condition: Expression::Literal(Literal::Bool(true)),
            then_branch: vec![Statement::Expression {
                expression: Expression::Literal(Literal::Integer(7)),
            }],
            else_branch: Some(vec![Statement::Expression {
                expression: Expression::Literal(Literal::Integer(9)),
            }]),
        }];

        ConstantPropagator.run(&mut statements);

        assert_eq!(statements.len(), 1);
        assert!(matches!(statements[0], Statement::Block(_)));
    }

    #[test]
    fn removes_while_false() {
        let mut statements = vec![Statement::While {
            condition: Expression::Literal(Literal::Bool(false)),
            body: vec![Statement::Break],
        }];

        ConstantPropagator.run(&mut statements);

        assert!(statements.is_empty());
    }

    #[test]
    fn preserves_constants_around_nonconstant_branches_when_both_paths_agree() {
        let mut statements = vec![
            Statement::Let {
                name: "x".into(),
                value: Expression::Literal(Literal::Integer(10)),
                mutable: false,
                type_annotation: None,
            },
            Statement::If {
                condition: Expression::Variable("flag".into()),
                then_branch: vec![Statement::Expression {
                    expression: Expression::Literal(Literal::Integer(1)),
                }],
                else_branch: Some(vec![Statement::Expression {
                    expression: Expression::Literal(Literal::Integer(2)),
                }]),
            },
            Statement::Expression {
                expression: Expression::Binary {
                    left: Box::new(Expression::Variable("x".into())),
                    operator: BinaryOp::Add,
                    right: Box::new(Expression::Literal(Literal::Integer(1))),
                    line: 1,
                    column: 1,
                },
            },
        ];

        ConstantPropagator.run(&mut statements);

        match &statements[2] {
            Statement::Expression { expression } => {
                assert_eq!(value_literal(expression), Literal::Integer(11));
            }
            other => panic!("expression attendue, reçu: {other:?}"),
        }
    }

    #[test]
    fn constant_if_keeps_branch_scope_and_does_not_leak_locals() {
        let mut statements = vec![
            Statement::If {
                condition: Expression::Literal(Literal::Bool(true)),
                then_branch: vec![Statement::Let {
                    name: "inner".into(),
                    value: Expression::Literal(Literal::Integer(2)),
                    mutable: false,
                    type_annotation: None,
                }],
                else_branch: None,
            },
            Statement::Expression {
                expression: Expression::Variable("inner".into()),
            },
        ];

        ConstantPropagator.run(&mut statements);

        assert!(matches!(statements[0], Statement::Block(_)));
        assert!(matches!(
            &statements[1],
            Statement::Expression {
                expression: Expression::Variable(name)
            } if name == "inner"
        ));
    }

    #[test]
    fn preserves_positioned_statements_during_propagation() {
        let mut statements = vec![Statement::Positioned {
            line: 12,
            column: 4,
            statement: Box::new(Statement::Let {
                name: "x".into(),
                value: Expression::Literal(Literal::Integer(7)),
                mutable: false,
                type_annotation: None,
            }),
        }];

        ConstantPropagator.run(&mut statements);

        assert!(matches!(
            &statements[0],
            Statement::Positioned {
                line: 12,
                column: 4,
                statement
            } if matches!(statement.as_ref(), Statement::Let { .. })
        ));
    }

    fn value_literal(expression: &Expression) -> Literal {
        match expression {
            Expression::Literal(value) => value.clone(),
            other => panic!("littéral attendu, reçu: {other:?}"),
        }
    }


    #[test]
    fn removes_statements_after_return() {
        let mut statements = vec![
            Statement::Return { value: None },
            Statement::Expression {
                expression: Expression::Literal(Literal::Integer(1)),
            },
        ];

        DeadCodeEliminator.run(&mut statements);

        assert_eq!(statements.len(), 1);
        assert!(matches!(statements[0], Statement::Return { .. }));
    }

    #[test]
    fn removes_statements_after_throw() {
        let mut statements = vec![
            Statement::Throw {
                value: Expression::Literal(Literal::String("error".into())),
            },
            Statement::Expression {
                expression: Expression::Literal(Literal::Integer(1)),
            },
        ];

        DeadCodeEliminator.run(&mut statements);

        assert_eq!(statements.len(), 1);
        assert!(matches!(statements[0], Statement::Throw { .. }));
    }

    #[test]
    fn removes_statements_after_loop_control() {
        let mut break_body = vec![
            Statement::Break,
            Statement::Expression {
                expression: Expression::Literal(Literal::Integer(1)),
            },
        ];
        DeadCodeEliminator.run(&mut break_body);
        assert_eq!(break_body.len(), 1);

        let mut continue_body = vec![
            Statement::Continue,
            Statement::Expression {
                expression: Expression::Literal(Literal::Integer(2)),
            },
        ];
        DeadCodeEliminator.run(&mut continue_body);
        assert_eq!(continue_body.len(), 1);
    }

    #[test]
    fn removes_statements_after_if_when_both_branches_terminate() {
        let mut statements = vec![
            Statement::If {
                condition: Expression::Variable("flag".into()),
                then_branch: vec![Statement::Return { value: None }],
                else_branch: Some(vec![Statement::Throw {
                    value: Expression::Literal(Literal::String("boom".into())),
                }]),
            },
            Statement::Expression {
                expression: Expression::Literal(Literal::Integer(99)),
            },
        ];

        DeadCodeEliminator.run(&mut statements);

        assert_eq!(statements.len(), 1);
    }

    #[test]
    fn keeps_statements_after_if_when_one_branch_falls_through() {
        let mut statements = vec![
            Statement::If {
                condition: Expression::Variable("flag".into()),
                then_branch: vec![Statement::Return { value: None }],
                else_branch: Some(vec![Statement::Expression {
                    expression: Expression::Literal(Literal::Integer(1)),
                }]),
            },
            Statement::Expression {
                expression: Expression::Literal(Literal::Integer(2)),
            },
        ];

        DeadCodeEliminator.run(&mut statements);

        assert_eq!(statements.len(), 2);
    }

    #[test]
    fn keeps_code_after_while_true_with_break() {
        let mut statements = vec![
            Statement::While {
                condition: Expression::Literal(Literal::Bool(true)),
                body: vec![Statement::Break],
            },
            Statement::Expression {
                expression: Expression::Literal(Literal::Integer(7)),
            },
        ];

        DeadCodeEliminator.run(&mut statements);

        assert_eq!(statements.len(), 2);
    }

    #[test]
    fn removes_code_after_while_true_with_return() {
        let mut statements = vec![
            Statement::While {
                condition: Expression::Literal(Literal::Bool(true)),
                body: vec![Statement::Return {
                    value: Some(Expression::Literal(Literal::Integer(7))),
                }],
            },
            Statement::Expression {
                expression: Expression::Literal(Literal::Integer(8)),
            },
        ];

        DeadCodeEliminator.run(&mut statements);

        assert_eq!(statements.len(), 1);
    }

    #[test]
    fn removes_unused_literal_let_inside_function() {
        let mut statements = vec![Statement::Function {
            name: "f".into(),
            generic_params: vec![],
            params: vec![],
            param_types: vec![],
            return_type: None,
            body: vec![
                Statement::Let {
                    name: "dead".into(),
                    value: Expression::Literal(Literal::Integer(1)),
                    mutable: false,
                    type_annotation: None,
                },
                Statement::Return { value: None },
            ],
            is_async: false,
        }];

        DeadValueEliminator.run(&mut statements);

        let Statement::Function { body, .. } = &statements[0] else {
            panic!("fonction attendue");
        };
        assert_eq!(body.len(), 1);
        assert!(matches!(body[0], Statement::Return { .. }));
    }

    #[test]
    fn keeps_used_literal_let_inside_function() {
        let mut statements = vec![Statement::Function {
            name: "f".into(),
            generic_params: vec![],
            params: vec![],
            param_types: vec![],
            return_type: None,
            body: vec![
                Statement::Let {
                    name: "x".into(),
                    value: Expression::Literal(Literal::Integer(1)),
                    mutable: false,
                    type_annotation: None,
                },
                Statement::Return {
                    value: Some(Expression::Variable("x".into())),
                },
            ],
            is_async: false,
        }];

        DeadValueEliminator.run(&mut statements);

        let Statement::Function { body, .. } = &statements[0] else {
            panic!("fonction attendue");
        };
        assert_eq!(body.len(), 2);
    }

    #[test]
    fn keeps_literal_let_captured_by_nested_function() {
        let mut statements = vec![Statement::Function {
            name: "outer".into(),
            generic_params: vec![],
            params: vec![],
            param_types: vec![],
            return_type: None,
            body: vec![
                Statement::Let {
                    name: "captured".into(),
                    value: Expression::Literal(Literal::Integer(7)),
                    mutable: false,
                    type_annotation: None,
                },
                Statement::Expression {
                    expression: Expression::Function {
                        params: vec![],
                        body: vec![Statement::Return {
                            value: Some(Expression::Variable("captured".into())),
                        }],
                    },
                },
            ],
            is_async: false,
        }];

        DeadValueEliminator.run(&mut statements);

        let Statement::Function { body, .. } = &statements[0] else {
            panic!("fonction attendue");
        };
        assert!(matches!(body[0], Statement::Let { .. }));
    }

    #[test]
    fn keeps_nonliteral_initializer_even_when_binding_is_unused() {
        let mut statements = vec![Statement::Function {
            name: "f".into(),
            generic_params: vec![],
            params: vec![],
            param_types: vec![],
            return_type: None,
            body: vec![
                Statement::Let {
                    name: "x".into(),
                    value: Expression::Call {
                        callee: Box::new(Expression::Variable("make".into())),
                        generic_args: vec![],
                        arguments: vec![],
                        line: 1,
                        column: 1,
                    },
                    mutable: false,
                    type_annotation: None,
                },
                Statement::Return { value: None },
            ],
            is_async: false,
        }];

        DeadValueEliminator.run(&mut statements);

        let Statement::Function { body, .. } = &statements[0] else {
            panic!("fonction attendue");
        };
        assert_eq!(body.len(), 2);
    }

    #[test]
    fn leaves_module_level_literal_bindings_intact() {
        let mut statements = vec![Statement::Let {
            name: "module_value".into(),
            value: Expression::Literal(Literal::Integer(1)),
            mutable: false,
            type_annotation: None,
        }];

        DeadValueEliminator.run(&mut statements);

        assert_eq!(statements.len(), 1);
    }

    #[test]
    fn removes_constant_value_after_default_pipeline() {
        let mut statements = vec![Statement::Function {
            name: "f".into(),
            generic_params: vec![],
            params: vec![],
            param_types: vec![],
            return_type: None,
            body: vec![
                Statement::Let {
                    name: "dead".into(),
                    value: Expression::Binary {
                        left: Box::new(Expression::Literal(Literal::Integer(2))),
                        operator: BinaryOp::Multiply,
                        right: Box::new(Expression::Literal(Literal::Integer(3))),
                        line: 1,
                        column: 1,
                    },
                    mutable: false,
                    type_annotation: None,
                },
                Statement::Return { value: None },
            ],
            is_async: false,
        }];

        run_default_passes(&mut statements);

        let Statement::Function { body, .. } = &statements[0] else {
            panic!("fonction attendue");
        };
        assert_eq!(body.len(), 1);
    }

    #[test]
    fn liveness_marks_unbound_read_as_live() {
        let statements = vec![Statement::Return {
            value: Some(Expression::Variable("x".into())),
        }];

        let summary = LivenessAnalyzer.analyze_block(&statements);

        assert!(summary.live_in.contains("x"));
    }

    #[test]
    fn liveness_accounts_for_definitions_before_reads() {
        let statements = vec![
            Statement::Assignment {
                target: AssignmentTarget::Variable("x".into()),
                value: Expression::Literal(Literal::Integer(1)),
            },
            Statement::Return {
                value: Some(Expression::Variable("x".into())),
            },
        ];

        let live_out = HashSet::new();
        let summary = LivenessAnalyzer.analyze_block_with_live_out(&statements, &live_out);

        assert!(!summary.live_in.contains("x"));
    }

    #[test]
    fn cfg_liveness_merges_if_branches() {
        let statements = vec![
            Statement::If {
                condition: Expression::Variable("cond".into()),
                then_branch: vec![Statement::Assignment {
                    target: AssignmentTarget::Variable("x".into()),
                    value: Expression::Literal(Literal::Integer(1)),
                }],
                else_branch: Some(vec![Statement::Assignment {
                    target: AssignmentTarget::Variable("x".into()),
                    value: Expression::Literal(Literal::Integer(2)),
                }]),
            },
            Statement::Return {
                value: Some(Expression::Variable("x".into())),
            },
        ];

        let summary = LivenessAnalyzer.analyze_block(&statements);

        assert!(summary.live_in.contains("cond"));
        assert!(!summary.live_in.contains("x"));
    }

    #[test]
    fn dse_removes_dead_store_inside_if_branch() {
        let mut statements = vec![Statement::Function {
            name: "f".into(),
            generic_params: vec![],
            params: vec!["cond".into(), "x".into()],
            param_types: vec![None, None],
            return_type: None,
            body: vec![
                Statement::If {
                    condition: Expression::Variable("cond".into()),
                    then_branch: vec![Statement::Assignment {
                        target: AssignmentTarget::Variable("x".into()),
                        value: Expression::Literal(Literal::Integer(1)),
                    }],
                    else_branch: Some(vec![]),
                },
                Statement::Return { value: None },
            ],
            is_async: false,
        }];

        DeadStoreEliminator.run(&mut statements);

        let Statement::Function { body, .. } = &statements[0] else {
            panic!("fonction attendue");
        };
        let Statement::If { then_branch, .. } = &body[0] else {
            panic!("if attendu");
        };
        assert!(then_branch.is_empty());
    }

    #[test]
    fn dse_keeps_store_read_after_if() {
        let mut statements = vec![Statement::Function {
            name: "f".into(),
            generic_params: vec![],
            params: vec!["cond".into(), "x".into()],
            param_types: vec![None, None],
            return_type: None,
            body: vec![
                Statement::If {
                    condition: Expression::Variable("cond".into()),
                    then_branch: vec![Statement::Assignment {
                        target: AssignmentTarget::Variable("x".into()),
                        value: Expression::Literal(Literal::Integer(1)),
                    }],
                    else_branch: Some(vec![Statement::Assignment {
                        target: AssignmentTarget::Variable("x".into()),
                        value: Expression::Literal(Literal::Integer(2)),
                    }]),
                },
                Statement::Return {
                    value: Some(Expression::Variable("x".into())),
                },
            ],
            is_async: false,
        }];

        DeadStoreEliminator.run(&mut statements);

        let Statement::Function { body, .. } = &statements[0] else {
            panic!("fonction attendue");
        };
        let Statement::If {
            then_branch,
            else_branch,
            ..
        } = &body[0]
        else {
            panic!("if attendu");
        };
        assert_eq!(then_branch.len(), 1);
        assert_eq!(else_branch.as_ref().map(Vec::len), Some(1));
    }

    #[test]
    fn dse_removes_dead_store_from_loop_body() {
        let mut statements = vec![Statement::Function {
            name: "f".into(),
            generic_params: vec![],
            params: vec!["cond".into(), "x".into()],
            param_types: vec![None, None],
            return_type: None,
            body: vec![
                Statement::While {
                    condition: Expression::Variable("cond".into()),
                    body: vec![Statement::Assignment {
                        target: AssignmentTarget::Variable("x".into()),
                        value: Expression::Literal(Literal::Integer(1)),
                    }],
                },
                Statement::Return { value: None },
            ],
            is_async: false,
        }];

        DeadStoreEliminator.run(&mut statements);

        let Statement::Function { body, .. } = &statements[0] else {
            panic!("fonction attendue");
        };
        let Statement::While { body: loop_body, .. } = &body[0] else {
            panic!("while attendu");
        };
        assert!(loop_body.is_empty());
    }

    #[test]
    fn dse_keeps_loop_store_used_by_next_condition() {
        let mut statements = vec![Statement::Function {
            name: "f".into(),
            generic_params: vec![],
            params: vec!["cond".into(), "x".into()],
            param_types: vec![None, None],
            return_type: None,
            body: vec![Statement::While {
                condition: Expression::Variable("x".into()),
                body: vec![Statement::Assignment {
                    target: AssignmentTarget::Variable("x".into()),
                    value: Expression::Literal(Literal::Integer(1)),
                }],
            }],
            is_async: false,
        }];

        DeadStoreEliminator.run(&mut statements);

        let Statement::Function { body, .. } = &statements[0] else {
            panic!("fonction attendue");
        };
        let Statement::While { body: loop_body, .. } = &body[0] else {
            panic!("while attendu");
        };
        assert_eq!(loop_body.len(), 1);
    }

    #[test]
    fn dse_removes_dead_store_before_break() {
        let mut statements = vec![Statement::Function {
            name: "f".into(),
            generic_params: vec![],
            params: vec!["cond".into(), "x".into()],
            param_types: vec![None, None],
            return_type: None,
            body: vec![Statement::While {
                condition: Expression::Variable("cond".into()),
                body: vec![
                    Statement::Assignment {
                        target: AssignmentTarget::Variable("x".into()),
                        value: Expression::Literal(Literal::Integer(1)),
                    },
                    Statement::Break,
                ],
            }],
            is_async: false,
        }];

        DeadStoreEliminator.run(&mut statements);

        let Statement::Function { body, .. } = &statements[0] else {
            panic!("fonction attendue");
        };
        let Statement::While { body: loop_body, .. } = &body[0] else {
            panic!("while attendu");
        };
        assert_eq!(loop_body.len(), 1);
        assert!(matches!(loop_body[0], Statement::Break));
    }

    #[test]
    fn dse_keeps_loop_store_used_before_continue_reaches_condition() {
        let mut statements = vec![Statement::Function {
            name: "f".into(),
            generic_params: vec![],
            params: vec!["x".into()],
            param_types: vec![None],
            return_type: None,
            body: vec![Statement::While {
                condition: Expression::Variable("x".into()),
                body: vec![
                    Statement::Assignment {
                        target: AssignmentTarget::Variable("x".into()),
                        value: Expression::Literal(Literal::Integer(1)),
                    },
                    Statement::Continue,
                ],
            }],
            is_async: false,
        }];

        DeadStoreEliminator.run(&mut statements);

        let Statement::Function { body, .. } = &statements[0] else {
            panic!("fonction attendue");
        };
        let Statement::While { body: loop_body, .. } = &body[0] else {
            panic!("while attendu");
        };
        assert_eq!(loop_body.len(), 2);
    }

    #[test]
    fn keeps_dead_store_inside_match_arm_until_cfg_liveness_is_available() {
        let mut statements = vec![Statement::Function {
            name: "f".into(),
            generic_params: vec![],
            params: vec!["value".into()],
            param_types: vec![None],
            return_type: None,
            body: vec![
                Statement::Let {
                    name: "result".into(),
                    value: Expression::Literal(Literal::Integer(0)),
                    mutable: false,
                    type_annotation: None,
                },
                Statement::Match {
                    value: Expression::Variable("value".into()),
                    arms: vec![MatchArm {
                        pattern: super::super::him::Pattern::Wildcard,
                        guard: None,
                        body: vec![Statement::Assignment {
                            target: AssignmentTarget::Variable("result".into()),
                            value: Expression::Literal(Literal::Integer(1)),
                        }],
                    }],
                },
                Statement::Return {
                    value: Some(Expression::Variable("result".into())),
                },
            ],
            is_async: false,
        }];

        DeadStoreEliminator.run(&mut statements);

        let Statement::Function { body, .. } = &statements[0] else {
            panic!("fonction attendue");
        };
        let Statement::Match { arms, .. } = &body[1] else {
            panic!("match attendu");
        };
        assert!(matches!(
            arms[0].body.first(),
            Some(Statement::Assignment {
                target: AssignmentTarget::Variable(name),
                value: Expression::Literal(Literal::Integer(1)),
            }) if name == "result"
        ));
    }

    #[test]
    fn keeps_loop_catch_store_until_cfg_liveness_is_available() {
        let mut statements = vec![Statement::Function {
            name: "f".into(),
            generic_params: vec![],
            params: vec![],
            param_types: vec![],
            return_type: None,
            body: vec![Statement::While {
                condition: Expression::Variable("running".into()),
                body: vec![Statement::Try {
                    try_body: vec![Statement::Expression {
                        expression: Expression::Variable("op".into()),
                    }],
                    catch_name: Some("e".into()),
                    catch_type: None,
                    catch_body: Some(vec![Statement::Assignment {
                        target: AssignmentTarget::Variable("running".into()),
                        value: Expression::Literal(Literal::Bool(false)),
                    }]),
                    finally_body: None,
                }],
            }],
            is_async: false,
        }];

        DeadStoreEliminator.run(&mut statements);

        let Statement::Function { body, .. } = &statements[0] else {
            panic!("fonction attendue");
        };
        let Statement::While { body, .. } = &body[0] else {
            panic!("while attendu");
        };
        let Statement::Try { catch_body: Some(catch_body), .. } = &body[0] else {
            panic!("try attendu");
        };
        assert!(matches!(
            catch_body.first(),
            Some(Statement::Assignment {
                target: AssignmentTarget::Variable(name),
                value: Expression::Literal(Literal::Bool(false)),
            }) if name == "running"
        ));
    }

    #[test]
    fn removes_dead_literal_assignment_to_local() {
        let mut statements = vec![Statement::Function {
            name: "f".into(),
            generic_params: vec![],
            params: vec!["x".into()],
            param_types: vec![None],
            return_type: None,
            body: vec![
                Statement::Assignment {
                    target: AssignmentTarget::Variable("x".into()),
                    value: Expression::Literal(Literal::Integer(1)),
                },
                Statement::Return {
                    value: Some(Expression::Literal(Literal::Integer(0))),
                },
            ],
            is_async: false,
        }];

        DeadStoreEliminator.run(&mut statements);

        let Statement::Function { body, .. } = &statements[0] else {
            panic!("fonction attendue");
        };
        assert_eq!(body.len(), 1);
        assert!(matches!(body[0], Statement::Return { .. }));
    }

    #[test]
    fn keeps_assignment_when_value_is_read_later() {
        let mut statements = vec![Statement::Function {
            name: "f".into(),
            generic_params: vec![],
            params: vec!["x".into()],
            param_types: vec![None],
            return_type: None,
            body: vec![
                Statement::Assignment {
                    target: AssignmentTarget::Variable("x".into()),
                    value: Expression::Literal(Literal::Integer(1)),
                },
                Statement::Return {
                    value: Some(Expression::Variable("x".into())),
                },
            ],
            is_async: false,
        }];

        DeadStoreEliminator.run(&mut statements);

        let Statement::Function { body, .. } = &statements[0] else {
            panic!("fonction attendue");
        };
        assert_eq!(body.len(), 2);
    }

    #[test]
    fn removes_only_the_overwritten_dead_store() {
        let mut statements = vec![Statement::Function {
            name: "f".into(),
            generic_params: vec![],
            params: vec!["x".into()],
            param_types: vec![None],
            return_type: None,
            body: vec![
                Statement::Assignment {
                    target: AssignmentTarget::Variable("x".into()),
                    value: Expression::Literal(Literal::Integer(1)),
                },
                Statement::Assignment {
                    target: AssignmentTarget::Variable("x".into()),
                    value: Expression::Literal(Literal::Integer(2)),
                },
                Statement::Return {
                    value: Some(Expression::Variable("x".into())),
                },
            ],
            is_async: false,
        }];

        DeadStoreEliminator.run(&mut statements);

        let Statement::Function { body, .. } = &statements[0] else {
            panic!("fonction attendue");
        };
        assert_eq!(body.len(), 2);
        assert!(matches!(body[0], Statement::Assignment { .. }));
    }

    #[test]
    fn keeps_dead_store_when_closure_can_read_it() {
        let mut statements = vec![Statement::Function {
            name: "outer".into(),
            generic_params: vec![],
            params: vec!["x".into()],
            param_types: vec![None],
            return_type: None,
            body: vec![
                Statement::Assignment {
                    target: AssignmentTarget::Variable("x".into()),
                    value: Expression::Literal(Literal::Integer(1)),
                },
                Statement::Expression {
                    expression: Expression::Function {
                        params: vec![],
                        body: vec![Statement::Return {
                            value: Some(Expression::Variable("x".into())),
                        }],
                    },
                },
            ],
            is_async: false,
        }];

        DeadStoreEliminator.run(&mut statements);

        let Statement::Function { body, .. } = &statements[0] else {
            panic!("fonction attendue");
        };
        assert_eq!(body.len(), 2);
    }

    #[test]
    fn leaves_module_level_assignment_intact() {
        let mut statements = vec![
            Statement::Let {
                name: "module_value".into(),
                value: Expression::Literal(Literal::Integer(0)),
                mutable: true,
                type_annotation: None,
            },
            Statement::Assignment {
                target: AssignmentTarget::Variable("module_value".into()),
                value: Expression::Literal(Literal::Integer(1)),
            },
        ];

        DeadStoreEliminator.run(&mut statements);

        assert_eq!(statements.len(), 2);
    }

    #[test]
    fn keeps_member_store_even_when_value_is_not_read_as_a_local() {
        let mut statements = vec![Statement::Function {
            name: "f".into(),
            generic_params: vec![],
            params: vec!["obj".into()],
            param_types: vec![None],
            return_type: None,
            body: vec![
                Statement::Assignment {
                    target: AssignmentTarget::Member {
                        object: Box::new(Expression::Variable("obj".into())),
                        name: "value".into(),
                    },
                    value: Expression::Literal(Literal::Integer(1)),
                },
                Statement::Return { value: None },
            ],
            is_async: false,
        }];

        DeadStoreEliminator.run(&mut statements);

        let Statement::Function { body, .. } = &statements[0] else {
            panic!("fonction attendue");
        };
        assert_eq!(body.len(), 2);
    }

    #[test]
    fn default_pipeline_runs_dead_store_elimination() {
        let mut statements = vec![Statement::Function {
            name: "f".into(),
            generic_params: vec![],
            params: vec!["x".into()],
            param_types: vec![None],
            return_type: None,
            body: vec![
                Statement::Assignment {
                    target: AssignmentTarget::Variable("x".into()),
                    value: Expression::Literal(Literal::Integer(7)),
                },
                Statement::Return { value: None },
            ],
            is_async: false,
        }];

        run_default_passes(&mut statements);

        let Statement::Function { body, .. } = &statements[0] else {
            panic!("fonction attendue");
        };
        assert_eq!(body.len(), 1);
    }

    #[test]
    fn keeps_assignment_used_from_finally() {
        let mut statements = vec![Statement::Function {
            name: "f".into(),
            generic_params: vec![],
            params: vec!["x".into()],
            param_types: vec![None],
            return_type: None,
            body: vec![
                Statement::Assignment {
                    target: AssignmentTarget::Variable("x".into()),
                    value: Expression::Literal(Literal::Integer(1)),
                },
                Statement::Try {
                    try_body: vec![Statement::Return { value: None }],
                    catch_name: None,
                    catch_type: None,
                    catch_body: None,
                    finally_body: Some(vec![Statement::Expression {
                        expression: Expression::Call {
                            callee: Box::new(Expression::Variable("use_value".into())),
                            generic_args: vec![],
                            arguments: vec![Expression::Variable("x".into())],
                            line: 1,
                            column: 1,
                        },
                    }]),
                },
            ],
            is_async: false,
        }];

        DeadStoreEliminator.run(&mut statements);

        let Statement::Function { body, .. } = &statements[0] else {
            panic!("fonction attendue");
        };
        assert_eq!(body.len(), 2);
    }

    #[test]
    fn preserves_finally_after_terminating_try_body() {
        let mut statements = vec![Statement::Try {
            try_body: vec![
                Statement::Return { value: None },
                Statement::Expression {
                    expression: Expression::Literal(Literal::Integer(1)),
                },
            ],
            catch_name: None,
            catch_type: None,
            catch_body: None,
            finally_body: Some(vec![
                Statement::Expression {
                    expression: Expression::Literal(Literal::Integer(2)),
                },
                Statement::Return { value: None },
            ]),
        }];

        DeadCodeEliminator.run(&mut statements);

        assert_eq!(statements.len(), 1);
        let Statement::Try {
            try_body,
            finally_body,
            ..
        } = &statements[0]
        else {
            panic!("try attendu");
        };
        assert_eq!(try_body.len(), 1);
        assert_eq!(finally_body.as_ref().expect("finally attendu").len(), 2);
    }

}
