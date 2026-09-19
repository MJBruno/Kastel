//! Typage graduel de Kastel — note de conception.
//!
//! ```text
//!                   AST
//!                   │
//!           ┌───────┴────────┐
//!           │                │
//!     annotation        aucune annotation
//!           │                │
//!      Type explicite      inférence
//!           │                │
//!           └───────┬────────┘
//!                   │
//!               TypeChecker
//!                   │
//!         ┌─────────┴─────────┐
//!         │                   │
//!      type connu          Dynamic
//!         │                   │
//!         └─────────┬─────────┘
//!                   │
//!              bytecode / VM
//! ```
//!
//! # Où on en est (Phase 1 — ce fichier + le parseur)
//!
//! La grammaire est acceptée et conservée dans l'AST :
//!
//! ```text
//! let a = 10;
//! let b: int = 20;
//! let p: Personne = new Personne();
//! func add(a: int, b: int) -> int { return a + b; }
//! ```
//!
//! `Statement::Let.type_annotation`, `Statement::Function`/
//! `FunctionMethod.param_types`/`.return_type` portent le nom brut de
//! type (`Option<String>`), tel qu'écrit par l'utilisateur. **Rien ne
//! le vérifie encore** : le compilateur les ignore purement et
//! simplement (voir les commentaires "Phase 1" dans
//! `compiler/statements.rs`). Une annotation fausse ou correcte
//! compile et s'exécute exactement pareil aujourd'hui — c'est un
//! choix délibéré : faire accepter la syntaxe est une étape séparée,
//! sans risque, de la vérification elle-même.
//!
//! `Type` ci-dessous convertit ce `String` brut en une représentation
//! structurée, prête à être consommée par le TypeChecker de la
//! Phase 2 — mais n'est pour l'instant appelée nulle part dans le
//! pipeline de compilation.
//!
//! # Phase 2 — le TypeChecker (à faire)
//!
//! Un passage entre le parsing et la compilation bytecode :
//!
//! - **Annoté** (`let b: int = ...`) : vérifie que le type de la
//!   valeur (inféré depuis l'expression) est compatible avec
//!   l'annotation. Incompatible -> diagnostic de compilation (pas une
//!   `RuntimeError` — l'esprit du typage graduel est de détecter ça
//!   AVANT l'exécution).
//! - **Non annoté** (`let name = "Bruno";`) : inférence locale simple
//!   (littéraux, propagation à travers les opérateurs binaires,
//!   type de retour d'un appel de fonction connu). Si l'inférence
//!   n'aboutit pas (ex. valeur qui dépend d'un paramètre non
//!   annoté), le type reste `Type::Dynamic` — ce n'est PAS une
//!   erreur, juste une absence d'information statique, exactement
//!   comme `any` en TypeScript.
//! - **`Type::Dynamic`** : aucune vérification statique ; le contrôle
//!   redevient celui d'aujourd'hui — une `RuntimeError::TypeError` si
//!   l'opération réelle ne correspond pas, au moment où elle a lieu.
//!
//! C'est la partie qui répond à "type connu / Dynamic" du schéma :
//! les DEUX chemins existent toujours et convergent vers le MÊME
//! bytecode — le typage n'ajoute pas un second runtime, il ajoute une
//! passe de diagnostic *avant* l'existant.
//!
//! # Phase 3 — surcharge de fonctions (le point dur)
//!
//! ```text
//! func add(a: int, b: int) -> int { return a + b; }
//! func add(a: float, b: float) -> float { return a + b; }
//! func add(a: int, b: float) -> float { return a + b; }
//! ```
//!
//! Aujourd'hui, `predeclare_global_function`/`compile_function_statement`
//! rejettent une deuxième déclaration de `add` avec
//! `CompileError::VariableAlreadyDeclared` — un nom global = UNE
//! valeur. Le support de la surcharge change ce modèle : plusieurs
//! fonctions peuvent partager un nom, tant que leurs *signatures*
//! diffèrent.
//!
//! Conception proposée (deux mécanismes, pas un seul) :
//!
//! 1. **Résolution statique par mangling**, quand TOUS les arguments
//!    d'un appel ont un type connu au point d'appel (annoté, ou
//!    inféré par la Phase 2) : le compilateur choisit la surcharge
//!    exacte au moment de la compilation et compile un appel DIRECT
//!    vers elle — aucun coût à l'exécution, exactement la branche
//!    "type connu" du schéma. En interne, chaque surcharge obtient un
//!    nom mangled unique (ex. `add$int$int`, `add$float$float`,
//!    `add$int$float`) invisible depuis Kastel.
//!
//! 2. **Dispatch dynamique**, quand au moins un argument est
//!    `Type::Dynamic` au point d'appel (ex. `dynamicAdd` existe
//!    justement pour ce cas, ou un `add(x, y)` où `x`/`y` viennent
//!    d'un paramètre non annoté) : le compilateur émet un appel vers
//!    une petite fonction "dispatcher" synthétisée, qui inspecte
//!    `type(a)`/`type(b)` À L'EXÉCUTION et branche vers la bonne
//!    surcharge mangled — avec une `RuntimeError::TypeError` claire
//!    si aucune surcharge ne correspond. C'est la branche "Dynamic"
//!    du schéma, qui rejoint le même bytecode derrière.
//!
//! Ce que ça implique concrètement, pas encore fait :
//!   - `self.globals` (aujourd'hui `HashMap<String, Global>`, UNE
//!     entrée par nom) devient `HashMap<String, Vec<OverloadSignature>>`
//!     pour les noms qui ont plusieurs surcharges ;
//!   - chaque site d'appel (`Expression::Call`) doit essayer de
//!     déduire le type de chaque argument (Phase 2) avant de choisir
//!     entre mangling statique et dispatcher dynamique ;
//!   - erreurs à définir : signatures ambiguës (`add(int, int)` et
//!     `add(int, int)` deux fois), aucune surcharge ne correspondant
//!     aux types statiquement connus, etc.
//!
//! C'est un changement bien plus invasif que la Phase 1/2 — je
//! recommande de ne l'attaquer qu'une fois la Phase 2 solide et
//! testée, plutôt que les faire toutes les trois d'un bloc sans
//! pouvoir compiler entre chaque étape.

/// Représentation structurée d'une annotation de type, une fois
/// résolue depuis le nom brut porté par l'AST (`Option<String>`).
///
/// Pas encore branché dans le pipeline de compilation (Phase 1) —
/// prêt pour le TypeChecker de la Phase 2.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Type {
    Int,
    Float,
    Str,
    Bool,
    Array,
    Dict,
    Tuple,
    /// Une classe ou interface utilisateur, par son nom (ex.
    /// `Type::Named("Personne".to_string())`). Résolue par son nom
    /// uniquement pour l'instant — pas de vérification que la classe
    /// existe réellement (rôle du TypeChecker, Phase 2).
    Named(String),
    /// Aucune information statique : soit non annoté et non
    /// inférable, soit explicitement voulu dynamique. C'est la
    /// branche "Dynamic" du schéma — jamais une erreur en soi.
    Dynamic,
}

impl Type {
    /// Convertit le nom brut porté par l'AST (`Option<String>`,
    /// tel qu'écrit après `:` ou `->`) en `Type`. `None` (annotation
    /// absente) donne `Type::Dynamic`.
    #[allow(dead_code)]
    pub fn from_annotation(annotation: Option<&str>) -> Type {
        match annotation {
            None => Type::Dynamic,
            Some("int") => Type::Int,
            Some("float") => Type::Float,
            Some("str") => Type::Str,
            Some("bool") => Type::Bool,
            Some("array") => Type::Array,
            Some("dict") => Type::Dict,
            Some("tuple") => Type::Tuple,
            Some(name) => Type::Named(name.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_primitive_names() {
        assert_eq!(Type::from_annotation(Some("int")), Type::Int);
        assert_eq!(Type::from_annotation(Some("float")), Type::Float);
        assert_eq!(Type::from_annotation(Some("str")), Type::Str);
        assert_eq!(Type::from_annotation(Some("bool")), Type::Bool);
    }

    #[test]
    fn maps_missing_annotation_to_dynamic() {
        assert_eq!(Type::from_annotation(None), Type::Dynamic);
    }

    #[test]
    fn maps_unknown_name_to_named() {
        assert_eq!(
            Type::from_annotation(Some("Personne")),
            Type::Named("Personne".to_string())
        );
    }
}
