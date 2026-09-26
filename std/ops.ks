// std/ops.ks
//
// Equivalent Kastel de std::ops / std::cmp : une interface par operateur
// surchargeable. AUCUNE de ces interfaces n'est necessaire au
// fonctionnement du langage - Add, Sub, Eq, Index, etc. sont des
// "capabilities" INTRINSEQUES que le compilateur reconnait nativement,
// exactement comme std::ops::Add existe dans le prelude de Rust sans
// import. Ce module existe pour la MEME raison qu'importer std::ops en
// Rust : documentation explicite, decouverte (IDE/autocompletion), et
// un point de reference unique pour ecrire ses propres bornes generiques
// (`<T: ops.Addable>` par exemple, voir plus bas).
//
// Convention Kastel <-> Rust :
//   - "Rhs" (right-hand side) devient le type de l'argument de la methode.
//   - "Output" devient le type de retour de la methode.
//   - `Self` fonctionne comme en Rust : substitue par le type concret a
//     l'implementation.
//   - Kastel n'a PAS de references : `IndexMut` n'emprunte donc pas
//     `&mut Output`, elle recoit directement la valeur a ecrire.

// ================================================================
// ARITHMETIC OPERATORS
// ================================================================

// std::ops::Add -> x + y
export interface Add {
    func add(other: Self) -> Self;
}

// std::ops::Sub -> x - y
export interface Sub {
    func sub(other: Self) -> Self;
}

// std::ops::Mul -> x * y
export interface Mul {
    func mul(other: Self) -> Self;
}

// std::ops::Div -> x / y
export interface Div {
    func div(other: Self) -> Self;
}

// std::ops::Rem -> x % y
export interface Mod {
    func mod(other: Self) -> Self;
}

// ================================================================
// UNARY OPERATORS
// ================================================================

// std::ops::Neg -> -x
export interface Neg {
    func neg() -> Self;
}

// std::ops::Not (partie bitwise uniquement : Kastel separe `!` et `~`,
// voir NOTE plus bas) -> ~x
export interface BitNot {
    func bitnot() -> Self;
}

// ================================================================
// BITWISE OPERATORS
// ================================================================

// std::ops::BitAnd -> x & y
export interface BitAnd {
    func bitand(other: Self) -> Self;
}

// std::ops::BitOr -> x | y
export interface BitOr {
    func bitor(other: Self) -> Self;
}

// std::ops::BitXor -> x ^ y
export interface BitXor {
    func bitxor(other: Self) -> Self;
}

// std::ops::Shl -> x << y
export interface ShiftLeft {
    func shl(other: Self) -> Self;
}

// std::ops::Shr -> x >> y
export interface ShiftRight {
    func shr(other: Self) -> Self;
}

// ================================================================
// COMPARISON (std::cmp)
// ================================================================

// std::cmp::PartialEq -> x == y, x != y
// Sortie FIXEE a bool, quel que soit Rhs (voir ops_generic.ks pour la
// forme heterogene Eq<str> par exemple).
export interface Eq {
    func equals(other: Self) -> bool;
}

// std::cmp::PartialOrd -> x < y, x <= y, x > y, x >= y
// `compare` renvoie -1, 0 ou 1 (ordre a trois valeurs, comme
// std::cmp::Ordering) ; le compilateur traduit chaque operateur en
// comparaison sur ce resultat : `a < b` devient `a.compare(b) < 0`,
// `a >= b` devient `!(a.compare(b) < 0)`, etc. Les operateurs restent
// bien de type bool.
export interface Ord {
    func compare(other: Self) -> int;
}

// ================================================================
// INDEXING
// ================================================================

// std::ops::Index -> x[y], &x[y]
// Forme heterogene obligatoire : Index<Idx, Output>.
export interface Index<Idx, Output> {
    func index(key: Idx) -> Output;
}

// std::ops::IndexMut -> x[y] = z, &mut x[y]
// Adaptation Kastel : pas de reference mutable, `set_index` recoit
// directement la valeur a ecrire et ne renvoie rien.
export interface IndexMut<Idx, Output> {
    func set_index(key: Idx, value: Output) -> None;
}

// ================================================================
// COMPOUND ASSIGNMENT
// ================================================================
//
// std::ops::AddAssign, SubAssign, MulAssign, DivAssign, RemAssign,
// BitAndAssign, BitOrAssign, BitXorAssign, ShlAssign, ShrAssign
// (x += y, x -= y, ..., x &= y, x |= y, x ^= y, x <<= y, x >>= y)
//
// PAS d'interface dediee ici : contrairement a Rust, Kastel n'a pas de
// trait "*Assign" separe. `x += y` est du SUCRE SYNTAXIQUE compile en
// `x = x + y` (idem pour tous les autres), donc `+=` fonctionne des
// qu'une classe implemente `Add`, `-=` des qu'elle implemente `Sub`,
// `&=` des qu'elle implemente `BitAnd`, etc. Aucune methode
// supplementaire a ecrire.

// ================================================================
// `!x` (std::ops::Not, partie logique) : PAS surchargeable
// ================================================================
//
// NOTE : en Rust, `std::ops::Not` couvre A LA FOIS `!flag` (bool) et
// `!bits` (entier, bitwise). Kastel separe les deux operateurs :
//   - `~x` (bitwise)  -> capability `BitNot` ci-dessus, surchargeable.
//   - `!x` (logique)  -> TOUJOURS bool, jamais surchargeable, y compris
//     sur un type `dynamic`. C'est un choix de conception delibere : `!`
//     reste un operateur logique pur, jamais redefinissable par une
//     classe. Pas d'interface a declarer pour lui ici.
