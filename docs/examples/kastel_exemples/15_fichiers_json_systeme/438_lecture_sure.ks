// ==================================================================
// Exemple 438 — Lecture sûre avec Result
// Catégorie : Fichiers, JSON, système
// ------------------------------------------------------------------
// Envelopper l'erreur de fichier dans un Result.
// ------------------------------------------------------------------
// Sortie attendue :
//   Err(impossible de lire absent_kastel.txt)
// ==================================================================

func lire(chemin: str) -> Result<str, str> {
    try {
        return Ok(file_read(chemin));
    } catch (e) {
        return Err("impossible de lire " + chemin);
    }
}

println(lire("absent_kastel.txt"));
