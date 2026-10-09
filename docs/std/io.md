# `std.io` — handles de fichiers

Kastel possède deux formes d'accès aux fichiers :

- les helpers stateless `std.fs` (`file_read`, `file_write`, `file_read_bytes`, etc.) ;
- les handles persistants `File` et le builder `OpenOptions` exposés depuis `std.io`.

## Ouvrir un fichier

```ks
from std.io import open_options, SeekFrom

let file = open_options()
    .read(true)
    .write(true)
    .create(true)
    .truncate(true)
    .open("demo.txt");

file.write_all("Kastel\n");
file.seek(0, SeekFrom.Start);
let content = file.read_to_string();
println(content);
file.sync_all();
file.close();
```

Pour une lecture seule, utiliser `open_file(path)`. Pour créer ou tronquer un fichier en écriture seule, utiliser `create_file(path)`.

## Méthodes `File`

| Méthode | Contrat |
|---|---|
| `read(max_bytes)` | Lit au plus le nombre d'octets demandé et retourne `List<int>`; plafond de 16 MiB par appel |
| `read_to_end()` | Lit les octets restants et retourne `List<int>` |
| `read_to_string()` | Lit les octets restants et les décode en UTF-8 (`str`) |
| `write(data)` | Écrit tout ou partie de `str`, `List<int>` ou tuple d'octets et retourne le nombre d'octets écrits |
| `write_all(data)` | Écrit toutes les données ou renvoie une erreur |
| `flush()` | Vide les tampons associés au handle |
| `seek(offset, SeekFrom.Start/Current/End)` | Déplace la position et retourne la nouvelle position |
| `stream_position()` | Retourne la position courante |
| `set_len(length)` | Modifie la taille du fichier |
| `metadata()` | Record `size`, `is_file`, `is_dir`, `readonly` |
| `sync_all()` / `sync_data()` | Demande la synchronisation au système de fichiers |
| `try_clone()` | Crée un autre handle OS; les positions partagées suivent la sémantique du système |
| `close()` | Ferme ce handle; la fermeture est idempotente |
| `is_closed()` / `path()` | Inspectent l'état et le chemin conservé par le handle |

Les méthodes sont synchrones et peuvent bloquer pendant les appels système. Les handles ne contiennent pas de `Value` Kastel interne; le GC les traite comme des ressources opaques et leur destruction ferme les fichiers OS. Le mode d'accès (`read`, `write`, `append`, `truncate`, `create`, `create_new`) est défini par `OpenOptions` et validé par le système d'exploitation lors de `open()`.

`BufReader`, `BufWriter`, `read_line` sur handle, les streams asynchrones et le trait de type `Read`/`Write` ne sont pas inclus dans ce premier niveau.
