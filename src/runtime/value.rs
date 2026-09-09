use std::rc::Rc;

use crate::error::runtime_error::RuntimeError;
use crate::module::module::ModuleInstance;
use crate::stdlib::NativeFn;
use crate::runtime::function::Function;
use crate::runtime::gc_handle::Gc;
use crate::runtime::object::Object;

pub enum NumericOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

#[allow(dead_code)]
pub enum ComparisonOp {
    Equal,
    Greater,
    Less,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code, unpredictable_function_pointer_comparisons)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),

    /// Pointeur de fonction native : `Copy`, aucune allocation, ne peut
    /// jamais former de cycle — reste un Value primitif, hors du système
    /// `Object`/`Gc` (voir object.rs pour la justification complète).
    NativeFunction(NativeFn),

    /// Intervalle numérique léger et RÉUTILISABLE, produit par range().
    /// Aucune allocation sur le tas (juste 3 f64) — c'est ce qui rend
    /// range() paresseux. Reste hors du système `Object`/`Gc` pour la
    /// même raison que NativeFunction : l'y faire entrer ajouterait une
    /// allocation à chaque appel de range(), ce qui va à l'encontre du
    /// but recherché.
    Range {
        start: f64,
        stop: f64,
        step: f64,
    },

    /// TOUT ce qui est alloué sur le tas et suivi par le collecteur de
    /// cycles passe par cette seule variante : chaînes, tableaux, objets
    /// dynamiques, fonctions, closures, itérateurs à état, modules — voir
    /// object.rs pour le détail de chaque variante d'`Object`. Le GC n'a
    /// plus qu'UN SEUL registre à parcourir (voir gc.rs), au lieu d'un
    /// par type comme avant cette refonte.
    Object(Gc<Object>),

    Nil,
}

#[allow(dead_code)]
impl Value {
    // ============================================================
    // CONSTRUCTEURS DE COMMODITÉ
    // ============================================================
    //
    // Évitent d'avoir à écrire `Value::Object(Gc::new(Object::String(...)))`
    // en toutes lettres à chaque site d'appel — c'est LE point de passage
    // obligé pour créer chacun de ces types, exactement comme
    // `objet::new_closure`/`new_upvalue` l'étaient déjà avant cette
    // refonte pour Closure/ObjUpvalue.

    pub fn new_string(value: String) -> Self {
        Self::new_heap_object(Object::String(value))
    }

    pub fn new_array(elements: Vec<Value>) -> Self {
        Self::new_heap_object(Object::Array(elements))
    }

    /// Extrait le `Gc<Object>` sous-jacent si cette valeur en est un.
    /// Sert de point d'entrée générique pour tout code qui a besoin de
    /// travailler avec la poignée elle-même plutôt qu'un accès typé
    /// (ex. le marquage GC, ou le dispatch générique de propriétés).
    pub fn as_object(&self) -> Option<&Gc<Object>> {
        match self {
            Value::Object(handle) => Some(handle),
            _ => None,
        }
    }

    // ============================================================
    // STRING
    // ============================================================

    /// Retourne le contenu si cette valeur est une chaîne, sinon `None`.
    /// Clone le contenu (plutôt que d'exposer directement le `Ref` du
    /// `RefCell`) pour rester simple à l'usage : les appelants (native.rs,
    /// machine.rs) veulent presque toujours soit une copie immédiate, soit
    /// juste tester le type.
    pub fn as_string_value(&self) -> Option<String> {
        match self {
            Value::Object(handle) => match &*handle.borrow() {
                Object::String(s) => Some(s.clone()),
                _ => None,
            },

            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::Object(handle) if matches!(&*handle.borrow(), Object::String(_)))
    }

    // ============================================================
    // ARRAY
    // ============================================================

    pub fn array_get(&self, index: usize) -> Result<Value, RuntimeError> {
        self.with_array(|array| {
            array
                .get(index)
                .cloned()
                .ok_or(RuntimeError::IndexOutOfBounds)
        })
    }

    pub fn array_set(&self, index: usize, value: Value) -> Result<(), RuntimeError> {
        self.with_array_mut(|array| {
            let slot = array.get_mut(index).ok_or(RuntimeError::IndexOutOfBounds)?;
            *slot = value;
            Ok(())
        })
    }

    pub fn array_len(&self) -> Result<usize, RuntimeError> {
        self.with_array(|array| Ok(array.len()))
    }

    pub fn array_push(&self, value: Value) -> Result<usize, RuntimeError> {
        self.with_array_mut(|array| {
            array.push(value);
            Ok(array.len())
        })
    }

    pub fn array_pop(&self) -> Result<Value, RuntimeError> {
        self.with_array_mut(|array| {
            // Convention JS-like : pop() sur un tableau vide renvoie nil
            // plutôt que de lever une erreur.
            Ok(array.pop().unwrap_or(Value::Nil))
        })
    }

    pub fn array_insert(&self, index: usize, value: Value) -> Result<usize, RuntimeError> {
        self.with_array_mut(|array| {
            let length = array.len();

            if index > length {
                return Err(RuntimeError::ArrayIndexOutOfBounds { index, length });
            }

            array.insert(index, value);

            Ok(array.len())
        })
    }

    pub fn array_remove(&self, index: usize) -> Result<Value, RuntimeError> {
        self.with_array_mut(|array| {
            let length = array.len();

            if index >= length {
                return Err(RuntimeError::ArrayIndexOutOfBounds { index, length });
            }

            Ok(array.remove(index))
        })
    }

    pub fn array_clear(&self) -> Result<(), RuntimeError> {
        self.with_array_mut(|array| {
            array.clear();
            Ok(())
        })
    }

    pub fn array_contains(&self, value: &Value) -> Result<bool, RuntimeError> {
        self.with_array(|array| Ok(array.iter().any(|element| element == value)))
    }

    /// Accès en lecture au `Vec<Value>` sous-jacent si cette valeur est
    /// bien un tableau, sinon `TypeError`. Centralise le "déballage"
    /// `Value::Object -> Object::Array` commun à toutes les méthodes
    /// `array_*` ci-dessus.
    fn with_array<R>(
        &self,
        f: impl FnOnce(&Vec<Value>) -> Result<R, RuntimeError>,
    ) -> Result<R, RuntimeError> {
        match self {
            Value::Object(handle) => match &*handle.borrow() {
                Object::Array(array) => f(array),
                _ => Err(RuntimeError::NotIndexable),
            },

            _ => Err(RuntimeError::NotIndexable),
        }
    }

    fn with_array_mut<R>(
        &self,
        f: impl FnOnce(&mut Vec<Value>) -> Result<R, RuntimeError>,
    ) -> Result<R, RuntimeError> {
        match self {
            Value::Object(handle) => match &mut *handle.borrow_mut() {
                Object::Array(array) => f(array),
                _ => Err(RuntimeError::NotIndexable),
            },

            _ => Err(RuntimeError::NotIndexable),
        }
    }

    // ============================================================
    // FUNCTION
    // ============================================================

    /// Construit une valeur `Function` — utilisée par le compilateur pour
    /// peupler le pool de constantes (c'est CE constant qu'`OP_CLOSURE`
    /// retrouve ensuite à l'exécution pour fabriquer la closure elle-même).
    pub fn new_function(function: Rc<Function>) -> Self {
        Self::new_heap_object(Object::Function(function))
    }

    // ============================================================
    // MODULE
    // ============================================================

    pub fn new_module(module: Rc<ModuleInstance>) -> Self {
        Self::new_heap_object(Object::Module(module))
    }

    pub fn module_get(&self, name: &str) -> Result<Value, RuntimeError> {
        match self {
            Value::Object(handle) => match &*handle.borrow() {
                Object::Module(module) => module.get_export(name).cloned().ok_or_else(|| {
                    RuntimeError::ModuleError(format!(
                        "Module '{}' does not export '{}'",
                        module.name, name
                    ))
                }),

                _ => Err(RuntimeError::TypeError),
            },

            _ => Err(RuntimeError::TypeError),
        }
    }

   // ============================================================
//                         DICT
// ============================================================
//
// Dict = dictionnaire Kastel.
//
// Représentation interne :
//     Object::Dict(Vec<(String, Value)>)
//
// Les clés sont actuellement des chaînes. L'ordre d'insertion
// est conservé.
//
// Cette couche constitue l'API unique utilisée par la VM et la
// stdlib pour manipuler les dictionnaires.
//

pub fn new_dict(entries: Vec<(String, Value)>) -> Self {
    Self::new_heap_object(Object::Dict(entries))
}

pub fn dict_get(&self, key: &str) -> Result<Value, RuntimeError> {
    match self {
        Value::Object(handle) => match &*handle.borrow() {
            Object::Dict(entries) => entries
                .iter()
                .find(|(entry_key, _)| entry_key == key)
                .map(|(_, value)| value.clone())
                .ok_or_else(|| {
                    let suggestion = crate::error::suggest::closest_match(
                        key,
                        entries.iter().map(|(entry_key, _)| entry_key.as_str()),
                    );

                    RuntimeError::ObjectFieldNotFound {
                        name: key.to_string(),
                        suggestion,
                    }
                }),

            _ => Err(RuntimeError::TypeError),
        },

        _ => Err(RuntimeError::TypeError),
    }
}

pub fn dict_set(
    &self,
    key: &str,
    value: Value,
) -> Result<(), RuntimeError> {
    match self {
        Value::Object(handle) => {
            let mut object = handle.borrow_mut();

            match &mut *object {
                Object::Dict(entries) => {
                    if let Some((_, existing)) =
                        entries.iter_mut().find(|(entry_key, _)| entry_key == key)
                    {
                        *existing = value;
                    } else {
                        entries.push((key.to_string(), value));
                    }

                    Ok(())
                }

                _ => Err(RuntimeError::TypeError),
            }
        }

        _ => Err(RuntimeError::TypeError),
    }
}

pub fn dict_contains(
    &self,
    key: &str,
) -> Result<bool, RuntimeError> {
    match self {
        Value::Object(handle) => match &*handle.borrow() {
            Object::Dict(entries) => {
                Ok(entries.iter().any(|(entry_key, _)| entry_key == key))
            }

            _ => Err(RuntimeError::TypeError),
        },

        _ => Err(RuntimeError::TypeError),
    }
}

pub fn dict_remove(
    &self,
    key: &str,
) -> Result<Value, RuntimeError> {
    match self {
        Value::Object(handle) => {
            let mut object = handle.borrow_mut();

            match &mut *object {
                Object::Dict(entries) => {
                    let index = entries
                        .iter()
                        .position(|(entry_key, _)| entry_key == key)
                        .ok_or_else(|| {
                            let suggestion =
                                crate::error::suggest::closest_match(
                                    key,
                                    entries
                                        .iter()
                                        .map(|(entry_key, _)| entry_key.as_str()),
                                );

                            RuntimeError::ObjectFieldNotFound {
                                name: key.to_string(),
                                suggestion,
                            }
                        })?;

                    Ok(entries.remove(index).1)
                }

                _ => Err(RuntimeError::TypeError),
            }
        }

        _ => Err(RuntimeError::TypeError),
    }
}

pub fn dict_len(&self) -> Result<usize, RuntimeError> {
    match self {
        Value::Object(handle) => match &*handle.borrow() {
            Object::Dict(entries) => Ok(entries.len()),
            _ => Err(RuntimeError::TypeError),
        },

        _ => Err(RuntimeError::TypeError),
    }
}

pub fn dict_keys(&self) -> Result<Value, RuntimeError> {
    match self {
        Value::Object(handle) => match &*handle.borrow() {
            Object::Dict(entries) => {
                let keys = entries
                    .iter()
                    .map(|(key, _)| Value::new_string(key.clone()))
                    .collect();

                Ok(Value::new_array(keys))
            }

            _ => Err(RuntimeError::TypeError),
        },

        _ => Err(RuntimeError::TypeError),
    }
}

pub fn dict_values(&self) -> Result<Value, RuntimeError> {
    match self {
        Value::Object(handle) => match &*handle.borrow() {
            Object::Dict(entries) => {
                let values = entries
                    .iter()
                    .map(|(_, value)| value.clone())
                    .collect();

                Ok(Value::new_array(values))
            }

            _ => Err(RuntimeError::TypeError),
        },

        _ => Err(RuntimeError::TypeError),
    }
}

pub fn dict_items(&self) -> Result<Value, RuntimeError> {
    match self {
        Value::Object(handle) => match &*handle.borrow() {
            Object::Dict(entries) => {
                let items = entries
                    .iter()
                    .map(|(key, value)| {
                        Value::new_array(vec![
                            Value::new_string(key.clone()),
                            value.clone(),
                        ])
                    })
                    .collect();

                Ok(Value::new_array(items))
            }

            _ => Err(RuntimeError::TypeError),
        },

        _ => Err(RuntimeError::TypeError),
    }
}
    // ============================================================
    //                      ACCÈS UNIFIÉ AUX PROPRIÉTÉS
    // ============================================================
    //
    // Utilisé par les opcodes GetProperty/SetProperty : `user.name` et
    // `module.export` partagent la même syntaxe (Expression::Member), donc
    // la VM n'a pas besoin de savoir à la compilation lequel des deux
    // c'est — elle demande juste "récupère/assigne la propriété `name`" et
    // laisse le type réel de la valeur décider du comportement.

    pub fn get_property(&self, name: &str) -> Result<Value, RuntimeError> {
        match self {
            Value::Object(handle) => match &*handle.borrow() {
                Object::Module(_) => self.module_get(name),
                Object::Dict(_) => self.dict_get(name),
                _ => Err(RuntimeError::NotObject),
            },

            _ => Err(RuntimeError::NotObject),
        }
    }

    pub fn set_property(&self, name: &str, value: Value) -> Result<(), RuntimeError> {
        match self {
            // Les modules restent en lecture seule : leurs exports sont
            // figés à la compilation du module importé.
            Value::Object(handle) => {
                // ⚠️ L'emprunt de `matches!` doit être entièrement terminé
                // AVANT d'appeler `dict_set` (qui fait son propre
                // `borrow_mut()`) : le tenir plus longtemps ferait
                // paniquer avec "already borrowed" à chaque affectation.
                let is_dict = matches!(&*handle.borrow(), Object::Dict(_));

                if is_dict {
                   self.dict_set(name, value)
                } else {
                    Err(RuntimeError::NotObject)
                }
            }

            _ => Err(RuntimeError::NotObject),
        }
    }

    /// Construit un `Value::Object` autour de n'importe quelle variante
    /// `Object`, en l'enregistrant systématiquement auprès du GC. Unique
    /// point de passage PRIVÉ pour créer un `Gc<Object>` depuis `Value` —
    /// les constructeurs publics ci-dessus (`new_string`, `new_array`,
    /// `new_module`, `new_object`, `new_function`) passent tous par lui.
    fn new_heap_object(object: Object) -> Self {
        let handle = Gc::new(object);

        crate::runtime::gc::register_object(&handle);

        Value::Object(handle)
    }
}

#[allow(dead_code)]
impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(value) => {
                write!(f, "{value}")
            }

            Value::Float(value) => {
                // Toujours au moins un point décimal, même pour une valeur
                // entière (5.0, pas 5) : c'est ce qui permet de distinguer
                // visuellement un Float d'un Integer, comme repr() en
                // Python. NaN/Infinity gardent leur affichage naturel.
                if value.fract() == 0.0 && value.is_finite() {
                    write!(f, "{value:.1}")
                } else {
                    write!(f, "{value}")
                }
            }

            Value::Boolean(value) => {
                write!(f, "{value}")
            }

            Value::Nil => {
                write!(f, "null")
            }

            Value::NativeFunction(function) => {
                write!(f, "<nativeFn '{:?}'>", function)
            }

            Value::Range { start, stop, step } => {
                if *step == 1.0 {
                    write!(f, "range({start}, {stop})")
                } else {
                    write!(f, "range({start}, {stop}, {step})")
                }
            }

            Value::Object(handle) => match &*handle.borrow() {
                Object::String(value) => write!(f, "{value}"),

                Object::Array(array) => {
                    write!(f, "[")?;

                    for (index, value) in array.iter().enumerate() {
                        if index > 0 {
                            write!(f, ", ")?;
                        }

                        write!(f, "{value}")?;
                    }

                    write!(f, "]")
                }

                Object::Dict(fields) => {
                    write!(f, "{{")?;

                    for (index, (key, value)) in fields.iter().enumerate() {
                        if index > 0 {
                            write!(f, ", ")?;
                        }

                        write!(f, "{key}: {value}")?;
                    }

                    write!(f, "}}")
                }

                Object::Function(function) => {
                    write!(f, "<fun '{}'>", function.name)
                }

                Object::Closure(closure) => {
                    write!(f, "<closure '{}'>", closure.function.name)
                }

                Object::Iterator(_) => {
                    write!(f, "<iterator>")
                }

                Object::Module(module) => {
                    write!(f, "<module '{}'>", module.name)
                }
            },
        }
    }
}

#[allow(dead_code)]
impl Value {
    // ============================================================
    // TRUTHINESS
    // ============================================================

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Nil => false,

            Value::Boolean(value) => *value,

            Value::Integer(value) => *value != 0,

            Value::Float(value) => *value != 0.0 && !value.is_nan(),

            Value::Object(handle) => match &*handle.borrow() {
                Object::String(value) => !value.is_empty(),
                _ => true,
            },

            _ => true,
        }
    }

    // ============================================================
    // NUMERIC OPERATIONS
    // ============================================================

    pub fn binary_numeric_op(a: Value, b: Value, op: NumericOp) -> Result<Value, RuntimeError> {
        match (a, b) {
            // Int op Int -> Int (sauf division, toujours "vraie division"
            // façon Python 3 : 7 / 2 == 3.5, pas 3 — Kastel n'a pas
            // d'opérateur de division entière séparé).
            (Value::Integer(a), Value::Integer(b)) => match op {
                NumericOp::Add => Ok(Value::Integer(a.wrapping_add(b))),
                NumericOp::Subtract => Ok(Value::Integer(a.wrapping_sub(b))),
                NumericOp::Multiply => Ok(Value::Integer(a.wrapping_mul(b))),

                NumericOp::Divide => {
                    if b == 0 {
                        return Err(RuntimeError::DivisionByZero);
                    }

                    Ok(Value::Float(a as f64 / b as f64))
                }

                NumericOp::Modulo => {
                    if b == 0 {
                        return Err(RuntimeError::DivisionByZero);
                    }

                    if a == i64::MIN && b == -1 {
                        return Ok(Value::Integer(0));
                    }

                    Ok(Value::Integer(a % b))
                }
            },

            // Toute combinaison impliquant un Float est promue en Float.
            (Value::Integer(a), Value::Float(b)) => Self::number_op(a as f64, b, op),
            (Value::Float(a), Value::Integer(b)) => Self::number_op(a, b as f64, op),
            (Value::Float(a), Value::Float(b)) => Self::number_op(a, b, op),

            _ => Err(RuntimeError::TypeError),
        }
    }

    fn number_op(a: f64, b: f64, op: NumericOp) -> Result<Value, RuntimeError> {
        match op {
            NumericOp::Add => Ok(Value::Float(a + b)),

            NumericOp::Subtract => Ok(Value::Float(a - b)),

            NumericOp::Multiply => Ok(Value::Float(a * b)),

            NumericOp::Divide => {
                if b == 0.0 {
                    return Err(RuntimeError::DivisionByZero);
                }

                Ok(Value::Float(a / b))
            }

            NumericOp::Modulo => Ok(Value::Float(a % b)),
        }
    }

    pub fn negate_values(a: Value) -> Result<Value, RuntimeError> {
        match a {
            Value::Integer(a) => Ok(Value::Integer(a.wrapping_neg())),

            Value::Float(a) => Ok(Value::Float(-a)),

            _ => Err(RuntimeError::TypeError),
        }
    }

    // ============================================================
    // COMPARISON
    // ============================================================

    /// Compare deux valeurs numériques, Integer et Float mélangeables
    /// (5 < 5.5 doit fonctionner). Passe par f64 pour la comparaison
    /// inter-types — limite connue : au-delà de 2^53, deux i64 distincts
    /// peuvent devenir "égaux" une fois convertis en f64. Kastel n'a pas
    /// vocation à manipuler des entiers de cette taille pour l'instant.
    pub fn compare_numeric(a: Value, b: Value, op: ComparisonOp) -> Result<Value, RuntimeError> {
        match (a, b) {
            (Value::Integer(a), Value::Integer(b)) => {
                let result = match op {
                    ComparisonOp::Equal => a == b,
                    ComparisonOp::Greater => a > b,
                    ComparisonOp::Less => a < b,
                };

                Ok(Value::Boolean(result))
            }

            (Value::Integer(a), Value::Float(b)) => {
                Ok(Value::Boolean(Self::compare_number(a as f64, b, op)))
            }

            (Value::Float(a), Value::Integer(b)) => {
                Ok(Value::Boolean(Self::compare_number(a, b as f64, op)))
            }

            (Value::Float(a), Value::Float(b)) => {
                Ok(Value::Boolean(Self::compare_number(a, b, op)))
            }

            _ => Err(RuntimeError::TypeError),
        }
    }

    fn compare_number(a: f64, b: f64, op: ComparisonOp) -> bool {
        match op {
            ComparisonOp::Equal => a == b,
            ComparisonOp::Greater => a > b,
            ComparisonOp::Less => a < b,
        }
    }

    // ============================================================
    // EQUALITY
    // ============================================================

    pub fn equals(a: Value, b: Value) -> bool {
        match (a, b) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Integer(a), Value::Float(b)) => a as f64 == b,
            (Value::Float(a), Value::Integer(b)) => a == b as f64,
            (Value::Float(a), Value::Float(b)) => a == b,

            (Value::Boolean(a), Value::Boolean(b)) => a == b,

            // Seules les chaînes sont comparées PAR VALEUR ici, comme
            // avant cette refonte (tableaux/objets/closures n'étaient déjà
            // pas comparés structurellement par `==` : ce n'est pas une
            // régression, c'est la même limite qu'avant, juste préservée).
            (Value::Object(a), Value::Object(b)) => {
                if let (Object::String(a_str), Object::String(b_str)) = (&*a.borrow(), &*b.borrow())
                {
                    return a_str == b_str;
                }

                Gc::<Object>::ptr_eq(&a, &b)
            }

            (Value::Nil, Value::Nil) => true,

            _ => false,
        }
    }
}
