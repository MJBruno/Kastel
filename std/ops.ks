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
// UNARY OPERATORS -- PAS surchargeables dans cette version de Kastel
// ================================================================
//
// CORRECTIF : contrairement a Add/Sub/Mul/.../Ord ci-dessus, `-x` et
// `~x` ne passent PAS par le systeme de capabilities. Le TypeChecker
// (compiler/type_checker.rs, verification de UnaryOp::Negate /
// UnaryOp::BitNot) exige directement un operande numerique/int et ne
// cherche jamais de methode `neg`/`bitnot` sur une classe utilisateur.
// Declarer une interface `Neg`/`BitNot` et une methode `neg()`/
// `bitnot()` sur une classe NE branche PAS `-instance`/`~instance` :
// ca compile (le nom de methode n'a rien de special), mais l'operateur
// unaire lui-meme echoue au typage des qu'on l'utilise sur cette
// classe. Le seul moyen de faire un "moins" ou un "bitnot" pour un
// type utilisateur aujourd'hui est d'appeler la methode explicitement
// (`valeur.neg()`), jamais via `-valeur`.
//
// Les interfaces ci-dessous restent une CONVENTION DE NOMMAGE utile
// (et une borne generique valide, `<T: Neg>`) si vous ecrivez ce genre
// de methode a la main -- mais elles ne font PAS ce que `Add`/`Sub`
// font plus haut. A retirer si un futur compilateur les branche
// reellement (verifier `Capability` dans compiler/capability.rs :
// tant que `Neg`/`BitNot` n'y figurent pas, cette limite tient).

// Convention pour -x, PAS branchee sur l'operateur.
export interface Neg {
    func neg() -> Self;
}

// Convention pour ~x, PAS branchee sur l'operateur (`!x`, lui, est
// TOUJOURS bool et jamais surchargeable -- voir plus bas).
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
// INDEXING -- PAS surchargeable non plus dans cette version de Kastel
// ================================================================
//
// CORRECTIF : `x[y]` (OpCode::GetIndex) et `x[y] = z` (OpCode::SetIndex)
// ne regardent le type concret QUE pour Array/Tuple/Dict
// (vm/machine/dispatch.rs) ; une instance de classe utilisateur tombe
// directement dans le cas `_ => Err(NotIndexable)`, sans jamais chercher
// de methode `index`/`set_index`. Meme constat que pour Neg/BitNot
// au-dessus : declarer ces interfaces et les implementer ne fait RIEN
// pour `instance[cle]`, qui echoue toujours a l'execution. Pour "indexer"
// un objet aujourd'hui, il faut une methode nommee explicitement
// (`.get(cle)`, comme le fait Dict lui-meme) et l'appeler telle quelle.
//
// std::ops::Index -> x[y], &x[y]. Forme heterogene : Index<Idx, Output>.
// Convention seulement -- voir le correctif ci-dessus.
export interface Index<Idx, Output> {
    func index(key: Idx) -> Output;
}

// std::ops::IndexMut -> x[y] = z, &mut x[y]
// Adaptation Kastel : pas de reference mutable, `set_index` recoit
// directement la valeur a ecrire et ne renvoie rien. Convention
// seulement -- voir le correctif ci-dessus.
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
//   - `~x` (bitwise)  -> interface `BitNot` ci-dessus, mais PAS branchee
//     par le compilateur actuel (voir le correctif plus haut) : `~x`
//     reste reserve aux entiers natifs pour l'instant.
//   - `!x` (logique)  -> TOUJOURS bool, jamais surchargeable, y compris
//     sur un type `dynamic`. C'est un choix de conception delibere : `!`
//     reste un operateur logique pur, jamais redefinissable par une
//     classe. Pas d'interface a declarer pour lui ici.
