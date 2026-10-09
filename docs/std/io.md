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

## `BufReader`

```ks
from std.io import buf_reader, SeekFrom

let reader = buf_reader("journal.txt")
let first = reader.read_line() // Option<str> : ligne ou None à EOF
let bytes = reader.read(16)    // List<int>, au plus 16 octets
let buffered = reader.buffer() // octets préchargés, sans consommation
reader.seek(0, SeekFrom.Start)
reader.close()
```

`read_line()` conserve le séparateur de ligne lorsqu'il existe et renvoie `None` à la fin du fichier. Les lignes sont limitées à 16 Mio. `read(size)` accepte de 0 à 16 Mio et peut renvoyer moins d'octets que demandé, notamment à la fin du fichier. `read_to_end()` et `read_to_string()` sont limités à 16 Mio par appel. Pour les fichiers ordinaires, la taille restante est vérifiée avant de consommer des octets ; si elle dépasse la limite, une erreur est renvoyée et la position de lecture reste inchangée. Pour les fichiers plus volumineux, utiliser `read(size)` par blocs. La capacité par défaut du buffer est de 8 Kio ; `buf_reader_with_capacity(path, capacity)` permet de la configurer entre 1 octet et 16 Mio.

Méthodes : `read(size)`, `read_line()`, `read_to_end()`, `read_to_string()`, `buffer()`, `seek(offset, origin)`, `stream_position()`, `buffer_capacity()`, `path()`, `is_closed()` et `close()`. `buffer()` copie les octets actuellement préchargés et ne les consomme pas ; cet appel peut remplir le buffer depuis le fichier sous-jacent.

## `BufWriter`

```ks
from std.io import buf_writer

let writer = buf_writer("rapport.txt") // crée ou tronque le fichier
writer.write_all("Début\n")
writer.write_all([65, 66, 67])
writer.flush()       // vide le buffer vers le système d'exploitation
writer.sync_all()    // flush, puis demande la synchronisation du fichier
writer.close()       // flush avant fermeture ; l'erreur est propagée
```

La capacité par défaut est de 8 Kio. `buf_writer_with_capacity(path, capacity)` accepte une capacité entre 1 octet et 16 Mio. Le constructeur `buf_writer(path)` ouvre en écriture et tronque un fichier préexistant. Pour préserver le contenu et ajouter à la fin, utiliser `buf_writer_append(path)` ou `buf_writer_append_with_capacity(path, capacity)` ; ces variantes créent le fichier s'il n'existe pas, sans le tronquer. `write(value)` accepte une chaîne, une `List<int>` ou un tuple d'octets (0–255) et renvoie le nombre d'octets acceptés par le buffer ou écrits. `write_all(value)` écrit tous les octets ou renvoie une erreur. Il faut traiter les erreurs de `flush()`, `sync_all()` et `close()` pour ne pas ignorer les échecs d'écriture.

Méthodes : `write(value)`, `write_all(value)`, `flush()`, `sync_all()`, `buffer_capacity()`, `path()`, `is_closed()` et `close()`.

## Ressources et limites

Les handles possèdent leur fichier et sont suivis par le runtime/GC. `BufWriter.close()` vide le buffer avant de fermer ; si le flush échoue, le handle reste ouvert pour permettre une nouvelle tentative. Le nettoyage implicite du GC ne peut pas remonter une erreur : pour des écritures importantes, appeler explicitement `flush()`, `sync_all()` ou `close()` et vérifier le résultat. Les capacités sont bornées entre 1 octet et 16 Mio, et les lectures tamponnées explicites ne dépassent pas 16 Mio par appel.
