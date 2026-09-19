//! Signatures statiques des fonctions natives exposées par le runtime.
//!
//! Cette table est volontairement indépendante des implémentations Rust.
//! Le runtime continue d'enregistrer les fonctions comme avant ; le
//! TypeChecker utilise uniquement leur contrat de type.

use std::collections::HashMap;

use super::types::{FunctionType, Type};

fn function(params: &[Type], return_type: Type) -> Type {
    Type::Function(FunctionType {
        params: params.to_vec(),
        return_type: Box::new(return_type),
    })
}

fn unary(argument: Type, result: Type) -> Type {
    function(&[argument], result)
}

fn binary(left: Type, right: Type, result: Type) -> Type {
    function(&[left, right], result)
}

pub fn all() -> HashMap<String, Type> {
    use Type::*;

    let mut types = HashMap::new();

    // I/O. `print`, `println` et `input` ont une arité dynamique/optionnelle
    // dans le runtime ; on ne leur attribue donc pas une fausse arité fixe.
    types.insert("print".into(), Dynamic);
    types.insert("println".into(), Dynamic);
    types.insert("input".into(), Dynamic);

    // Conversion / système
    types.insert("int".into(), unary(Dynamic, Int));
    types.insert("float".into(), unary(Dynamic, Float));
    types.insert("str".into(), unary(Dynamic, Str));
    types.insert("bool".into(), unary(Dynamic, Bool));
    types.insert("type".into(), unary(Dynamic, Str));
    types.insert("clock".into(), function(&[], Float));
    types.insert("cwd".into(), function(&[], Str));
    types.insert("env".into(), unary(Str, Dynamic));

    // Math natives. Le runtime accepte int ou float en entrée via
    // `expect_number`, mais les fonctions trigonométriques renvoient
    // toujours Float.
    types.insert("abs".into(), unary(Dynamic, Dynamic));
    types.insert("floor".into(), unary(Dynamic, Int));
    types.insert("ceil".into(), unary(Dynamic, Int));
    types.insert("round".into(), unary(Dynamic, Int));
    types.insert("sqrt".into(), unary(Float, Float));
    types.insert("pow".into(), binary(Dynamic, Dynamic, Dynamic));
    types.insert("min".into(), binary(Dynamic, Dynamic, Dynamic));
    types.insert("max".into(), binary(Dynamic, Dynamic, Dynamic));

    for name in ["sin", "cos", "tan", "asin", "acos", "atan", "exp", "log", "log10"] {
        types.insert(name.into(), unary(Float, Float));
    }

    types.insert("atan2".into(), binary(Float, Float, Float));
    types.insert("rand".into(), function(&[], Float));
    types.insert("rand_int".into(), unary(Dynamic, Int));
    // `rand_range(low, high)` : deux bornes entières, résultat entier dans
    // [low, high) (voir `native_rand_range`).
    types.insert("rand_range".into(), binary(Dynamic, Dynamic, Int));

    // Structures / utilitaires
    types.insert(
        "dict".into(),
        function(&[], Dict(Box::new(Dynamic), Box::new(Dynamic))),
    );
    // `range` accepte 1, 2 ou 3 arguments : arité variable non représentée
    // par FunctionType pour l'instant. Il reste donc dynamique jusqu'à
    // l'introduction d'un modèle d'arité optionnelle.
    types.insert("range".into(), Dynamic);
    types.insert(
        "list".into(),
        unary(Dynamic, ArrayDynamic),
    );

    // Debug
    types.insert("inspect".into(), unary(Dynamic, Str));
    types.insert("debug".into(), unary(Dynamic, Dynamic));
    types.insert("format".into(), Dynamic);

    // JSON
    types.insert("json_encode".into(), unary(Dynamic, Str));
    types.insert("json_decode".into(), unary(Str, Dynamic));

    // Fichiers
    types.insert("file_read".into(), unary(Str, Str));
    types.insert("file_read_lines".into(), unary(Str, Array(Box::new(Str))));
    types.insert("file_write".into(), function(&[Str, Str], None));
    types.insert("file_append".into(), function(&[Str, Str], None));
    types.insert("file_exists".into(), unary(Str, Bool));
    types.insert("file_delete".into(), unary(Str, None));
    types.insert("file_size".into(), unary(Str, Int));

    // Path
    types.insert("path_join".into(), unary(Dynamic, Str));
    types.insert("path_exists".into(), unary(Str, Bool));
    types.insert("path_is_dir".into(), unary(Str, Bool));
    types.insert("path_is_file".into(), unary(Str, Bool));
    types.insert("path_absolute".into(), unary(Str, Str));
    types.insert("path_basename".into(), unary(Str, Str));
    types.insert("path_dirname".into(), unary(Str, Str));
    types.insert("path_extension".into(), unary(Str, Str));
    types.insert("path_stem".into(), unary(Str, Str));

    // OS
    types.insert("os_name".into(), function(&[], Str));
    types.insert("os_arch".into(), function(&[], Str));
    types.insert("args".into(), function(&[], Array(Box::new(Str))));
    types.insert("exit".into(), Dynamic);

    types
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_math_signatures() {
        let types = all();

        assert_eq!(
            types.get("sin"),
            Some(&Type::Function(FunctionType {
                params: vec![Type::Float],
                return_type: Box::new(Type::Float),
            }))
        );

        assert_eq!(
            types.get("floor"),
            Some(&Type::Function(FunctionType {
                params: vec![Type::Dynamic],
                return_type: Box::new(Type::Int),
            }))
        );
    }
}
