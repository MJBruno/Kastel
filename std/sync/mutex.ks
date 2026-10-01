// Mutex coopératif.
//
// Le runtime fournit le native `mutex()`. Ce module expose une API
// qualifiée et évite d'exposer directement les détails du runtime.

export func create() -> Mutex {
    return mutex();
}
