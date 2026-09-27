// examples/shapes.ks
//
// Bibliothèque utilisée par examples/type_aliases_across_modules.ks.

export type Point = { x: int, y: int };
export type Number = int | float;

export func origin() -> Point {
    return { x: 0, y: 0 };
}

// Union en paramètre de fonction, EXPORTÉE : `Number` est déclaré plus loin
// dans ce même fichier (référence en avant) — couvre le même chemin de
// résolution qu'un usage depuis un autre module.
export type ScaledPoint = { x: Number, y: Number };

export func scale(p: Point, factor: Number) -> ScaledPoint {
    return { x: p.x * factor, y: p.y * factor };
}
