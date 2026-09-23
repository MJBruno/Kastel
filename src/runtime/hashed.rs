//! Conteneurs à recherche en O(1) pour `Dict` et `Set`.
//!
//! Les valeurs Kastel n'ont pas de hachage intrinsèque : ce module définit
//! deux conteneurs qui conservent l'ORDRE D'INSERTION (comme avant) et
//! ajoutent un index de hachage ; les recherches, insertions et
//! appartenances passent de O(n) à O(1) en moyenne.
//!
//! Contrat (voir `Value::key_hash` / `Value::key_equals`) : deux clés égales
//! ont le même hachage.
//!
//! * nombres : `1` et `1.0` sont la MÊME clé (égalité exacte, pas via `f64`
//!   au-delà de 2^53) ;
//! * chaînes : par contenu ; tuples : par contenu (ils sont immuables) ;
//! * autres objets (listes, dicts, records, ensembles, fonctions...) : par
//!   IDENTITÉ.
//!
//! Suppression : O(n) (décalage des positions), comme dans un `Vec`.
//!
//! Emprunts : le hachage d'une clé et la recherche par comparaison se font
//! TOUJOURS avant d'emprunter le conteneur en écriture (voir `Value::dict_set`
//! et `Value::set_add`) : comparer ou hacher sous un emprunt mutable ferait
//! paniquer `d[d] = 1` ou `s.add(s)`.

use std::collections::HashMap;
use std::ops::Deref;

use crate::runtime::value::Value;

/// Table de hachage -> positions dans un `Vec` (plusieurs positions par seau
/// en cas de collision).
#[derive(Debug, Clone, Default, PartialEq)]
struct HashIndex {
    buckets: HashMap<u64, Vec<usize>>,
}

impl HashIndex {
    fn insert(&mut self, hash: u64, position: usize) {
        self.buckets.entry(hash).or_default().push(position);
    }

    fn find(&self, hash: u64, mut matches: impl FnMut(usize) -> bool) -> Option<usize> {
        self.buckets
            .get(&hash)?
            .iter()
            .copied()
            .find(|position| matches(*position))
    }

    /// Retire `position` de son seau, puis décale d'un cran les positions
    /// supérieures (le `Vec` sous-jacent vient de perdre un élément).
    fn remove_at(&mut self, hash: u64, position: usize) {
        if let Some(bucket) = self.buckets.get_mut(&hash) {
            bucket.retain(|existing| *existing != position);

            if bucket.is_empty() {
                self.buckets.remove(&hash);
            }
        }

        for bucket in self.buckets.values_mut() {
            for existing in bucket.iter_mut() {
                if *existing > position {
                    *existing -= 1;
                }
            }
        }
    }

    fn clear(&mut self) {
        self.buckets.clear();
    }
}

// ============================================================
//                            DICT
// ============================================================

/// Entrées d'un `Dict` : paires (clé, valeur) dans l'ordre d'insertion.
///
/// Se lit comme une tranche (`Deref<Target = [(Value, Value)]>` : `iter()`,
/// `len()`, `get(i)`...) ; toute MODIFICATION passe par les méthodes ci-dessous
/// pour garder l'index cohérent.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DictEntries {
    entries: Vec<(Value, Value)>,

    /// Hachage de chaque clé, en parallèle de `entries` (évite de re-hacher
    /// une clé — et de l'emprunter — lors d'une suppression).
    hashes: Vec<u64>,

    index: HashIndex,
}

impl DictEntries {
    /// Construit à partir de paires ; une clé répétée écrase la valeur
    /// précédente (la position de première insertion est conservée).
    pub fn from_pairs(pairs: Vec<(Value, Value)>) -> Self {
        let mut dict = Self::default();

        for (key, value) in pairs {
            let hash = key.key_hash();

            match dict.position(&key, hash) {
                Some(position) => dict.set_value_at(position, value),
                None => dict.push_new(key, value, hash),
            }
        }

        dict
    }

    /// Position de `key` (de hachage `hash`), s'il est présent.
    pub fn position(&self, key: &Value, hash: u64) -> Option<usize> {
        self.index.find(hash, |position| {
            Value::key_equals(&self.entries[position].0, key)
        })
    }

    /// Ajoute une clé que l'appelant SAIT absente (aucune comparaison ici).
    pub fn push_new(&mut self, key: Value, value: Value, hash: u64) {
        let position = self.entries.len();

        self.entries.push((key, value));
        self.hashes.push(hash);
        self.index.insert(hash, position);
    }

    pub fn set_value_at(&mut self, position: usize, value: Value) {
        self.entries[position].1 = value;
    }

    /// Retire l'entrée à `position` et la renvoie.
    pub fn remove_at(&mut self, position: usize) -> (Value, Value) {
        let hash = self.hashes.remove(position);
        let pair = self.entries.remove(position);

        self.index.remove_at(hash, position);

        pair
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.hashes.clear();
        self.index.clear();
    }
}

impl Deref for DictEntries {
    type Target = [(Value, Value)];

    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}

impl<'a> IntoIterator for &'a DictEntries {
    type Item = &'a (Value, Value);
    type IntoIter = std::slice::Iter<'a, (Value, Value)>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}

// ============================================================
//                            SET
// ============================================================

/// Éléments d'un `Set` : uniques, dans l'ordre d'insertion (voir
/// `DictEntries` pour le contrat de lecture et de modification).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SetElements {
    items: Vec<Value>,
    hashes: Vec<u64>,
    index: HashIndex,
}

impl SetElements {
    /// Construit en éliminant les doublons (le premier exemplaire est gardé).
    pub fn from_values(values: Vec<Value>) -> Self {
        let mut set = Self::default();

        for value in values {
            let hash = value.key_hash();

            if set.position(&value, hash).is_none() {
                set.push_new(value, hash);
            }
        }

        set
    }

    /// Construit à partir d'éléments que l'appelant garantit DÉJÀ uniques
    /// (sous-ensemble ou copie d'un ensemble existant).
    pub fn from_unique(values: Vec<Value>) -> Self {
        let mut set = Self::default();

        for value in values {
            let hash = value.key_hash();

            set.push_new(value, hash);
        }

        set
    }

    pub fn position(&self, value: &Value, hash: u64) -> Option<usize> {
        self.index.find(hash, |position| {
            Value::key_equals(&self.items[position], value)
        })
    }

    /// Ajoute un élément que l'appelant SAIT absent (aucune comparaison ici).
    pub fn push_new(&mut self, value: Value, hash: u64) {
        let position = self.items.len();

        self.items.push(value);
        self.hashes.push(hash);
        self.index.insert(hash, position);
    }

    pub fn remove_at(&mut self, position: usize) -> Value {
        let hash = self.hashes.remove(position);
        let value = self.items.remove(position);

        self.index.remove_at(hash, position);

        value
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.hashes.clear();
        self.index.clear();
    }
}

impl Deref for SetElements {
    type Target = [Value];

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl<'a> IntoIterator for &'a SetElements {
    type Item = &'a Value;
    type IntoIter = std::slice::Iter<'a, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::object::Object;

    fn int(value: i64) -> Value {
        Value::Integer(value)
    }

    fn text(value: &str) -> Value {
        Value::new_string(value.to_string())
    }

    fn pair(a: i64, b: i64) -> Value {
        Value::new_tuple(vec![int(a), int(b)])
    }

    #[test]
    fn dict_keys_unify_equal_numbers_and_compare_strings_and_tuples_by_content() {
        let dict = Value::new_dict(vec![(int(1), text("un")), (text("k"), text("kay"))]);

        // 1 et 1.0 sont la même clé.
        assert_eq!(dict.dict_get(&Value::Float(1.0)).unwrap().to_string(), "un");
        assert!(dict.dict_contains(&text("k")).unwrap());

        dict.dict_set(&Value::Float(1.0), text("UN")).unwrap();
        assert_eq!(dict.dict_len().unwrap(), 2);
        assert_eq!(dict.dict_get(&int(1)).unwrap().to_string(), "UN");

        // Un tuple est une clé PAR CONTENU (deux objets distincts).
        dict.dict_set(&pair(1, 2), text("p")).unwrap();
        assert_eq!(dict.dict_get(&pair(1, 2)).unwrap().to_string(), "p");
        assert!(!dict.dict_contains(&pair(2, 1)).unwrap());
    }

    #[test]
    fn integer_and_float_equality_is_exact() {
        // 2^53 + 1 n'est pas représentable en f64 : il ne doit pas égaler 2^53.
        let big = (1i64 << 53) + 1;

        assert!(!Value::equals(int(big), Value::Float((1u64 << 53) as f64)));
        assert!(Value::equals(int(3), Value::Float(3.0)));
        assert!(!Value::equals(int(3), Value::Float(3.5)));

        assert_eq!(int(3).key_hash(), Value::Float(3.0).key_hash());
        assert_eq!(Value::Float(0.0).key_hash(), Value::Float(-0.0).key_hash());
    }

    #[test]
    fn lookups_stay_correct_after_removals_shift_the_positions() {
        let dict = Value::new_dict(Vec::new());
        let set = Value::new_set(Vec::new());

        for value in 0..2_000 {
            dict.dict_set(&int(value), int(value * 10)).unwrap();
            assert!(set.set_add(int(value)).unwrap());
        }

        // Retraits répartis : les positions suivantes se décalent.
        for value in (0..2_000).step_by(3) {
            dict.dict_remove(&int(value)).unwrap();
            assert!(set.set_remove(&int(value)).unwrap());
        }

        for value in 0..2_000 {
            let present = value % 3 != 0;

            assert_eq!(dict.dict_contains(&int(value)).unwrap(), present);
            assert_eq!(set.set_contains(&int(value)).unwrap(), present);

            if present {
                assert!(matches!(
                    dict.dict_get(&int(value)).unwrap(),
                    Value::Integer(found) if found == value * 10
                ));
            }
        }

        // L'ordre d'insertion est conservé.
        let keys = dict.dict_keys().unwrap();
        let shown = keys.to_string();

        assert!(shown.starts_with("[1, 2, 4, 5, 7"), "{shown}");
    }

    #[test]
    fn a_container_can_hold_itself_without_panicking() {
        let dict = Value::new_dict(Vec::new());

        dict.dict_set(&dict, int(1)).unwrap();
        assert!(dict.dict_contains(&dict).unwrap());

        let set = Value::new_set(Vec::new());

        assert!(set.set_add(set.clone()).unwrap());
        assert!(!set.set_add(set.clone()).unwrap());
    }

    #[test]
    fn large_containers_are_not_quadratic() {
        // 100 000 éléments : en O(n²) (400 M de comparaisons) ce test
        // prendrait de longues minutes ; en O(1) par opération, quelques
        // millisecondes.
        let set = Value::new_set(Vec::new());
        let dict = Value::new_dict(Vec::new());

        for value in 0..100_000 {
            set.set_add(int(value)).unwrap();
            dict.dict_set(&int(value), int(value)).unwrap();
        }

        assert_eq!(set.set_len().unwrap(), 100_000);
        assert!(set.set_contains(&int(99_999)).unwrap());
        assert!(dict.dict_contains(&int(50_000)).unwrap());

        // Les entrées restent lisibles comme tranche.
        if let Value::Object(handle) = &dict {
            if let Object::Dict(entries) = &*handle.borrow() {
                assert_eq!(entries.len(), 100_000);
                assert!(matches!(entries[0].0, Value::Integer(0)));
            }
        }
    }
}
