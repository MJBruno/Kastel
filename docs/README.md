# Kastel — exemples de la std 1.0

Ce dossier contient un exemple `.ks` isolé par module officiel de la bibliothèque standard Kastel.

## Test un par un

Depuis la racine du dépôt Kastel :

```powershell
powershell -ExecutionPolicy Bypass -File .\examples\std\tools\run_one.ps1 .\examples\std\01_core.ks
```

Ou, après extraction directe dans le dépôt, avec le script depuis ce dossier :

```powershell
powershell -ExecutionPolicy Bypass -File .\examples\std\tools\run_one.ps1 01_core.ks
```

Le script lance : `cargo run -q --bin kastel -- <fichier>`.

## Ordre

01 core
02 collections
03 string
04 math
05 datetime
06 io (interactif)
07 fs
08 path
09 os
10 process
11 json
12 csv
13 toml
14 yaml
15 regex
16 net TCP client
17 net TCP server (reste en écoute)
18 net UDP
19 http client
20 thread
21 statistics
22 testing
23 ops
24 sync.channel
25 sync.mutex
26 sync.rwlock
27 sync.semaphore
28 sync.event
29 sync.barrier
30 sync.wait_group
31 sync.condvar
32 version

## Cas particuliers

### `02_collections.ks`
Il couvre `first_opt`, `get_opt`, `zip`, `chunks`, `find`, `partition` et `flatten_options`. Le module doit être importable avec la correction actuelle du typage contextuel des dictionnaires vides (`Dict<K,V>`).

### `06_io.ks`
Le programme demande deux lignes sur la console.

### `10_process.ks`
L'exemple utilise `git --version`, car le dépôt Kastel est développé avec Git.

### `16/17 net TCP`
Lancer d'abord `17_net_tcp_server.ks`, puis dans un second terminal `16_net_tcp_client.ks`. Le serveur écoute sur `127.0.0.1:8080` et doit être arrêté avec `Ctrl+C`.

### `18_net_udp.ks`
Le test est autonome et utilise deux sockets UDP sur loopback avec un port éphémère.

### `19_http.ks`
La version actuelle de `std.http` est un client HTTP/1.1. L'exemple utilise `http://example.com/` et nécessite un accès réseau. Il ne teste pas un serveur HTTP, car `serve()` n'existe pas encore dans l'API stable actuelle.

### Synchronisation
Les exemples de `std.sync.*` exécutent les opérations dans une tâche lorsque le runtime exige un contexte de tâche.

## Nettoyage

Les exemples `fs`, `json`, `csv`, `toml` et `yaml` créent de petits fichiers temporaires dans le répertoire courant. Supprime-les après test s'ils restent présents.

## Surface officielle

Les anciens noms `std.strings`, `std.file` et `std.statistic` ne sont pas utilisés. Les collections utilisent `List`, `size()`, `add()` et `contains()`.
