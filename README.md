<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.70%2B-orange?style=for-the-badge&logo=rust" alt="Rust Version">
  <img src="https://img.shields.io/badge/License-MIT-blue?style=for-the-badge" alt="License MIT">
  <img src="https://img.shields.io/badge/Status-En%20Développement-yellow?style=for-the-badge" alt="Status">
  <img src="https://img.shields.io/badge/Bytecode-VM-brightgreen?style=for-the-badge" alt="Bytecode VM">
</p>

![Architecture de Kastel](assets/logo/4.png)

# 🏰 Kastel

**Kastel** est un langage de programmation interprété, moderne et dynamique, conçu pour allier la simplicité syntaxique de JavaScript et la robustesse de Python. Entièrement écrit en **Rust** (sans dépendances externes), il repose sur une machine virtuelle stack-based et un compilateur bytecode, garantissant des performances solides et une gestion mémoire fiable grâce à un ramasse-miettes (Garbage Collector) intégré.

> *"Un langage pensé pour l'expressivité, construit pour la performance."*

---

## ✨ Fonctionnalités clés

| Domaine | Fonctionnalités |
| :--- | :--- |
| **Cœur du langage** | Typage dynamique, variables mutables (`let`) et constantes (`const`), fonctions pures, récursivité. |
| **Avancé** | Fermetures (Closures), Upvalues, tableaux dynamiques, objets natifs avec propriétés et indexation. |
| **Exécution** | Compilateur bytecode, VM basée sur une pile (Stack-based), Garbage Collector automatique. |
| **Écosystème** | système de modules avec `import` / `export`, désassembleur de bytecode. |
| **Gestion d'erreurs** | Système d'erreurs à plusieurs niveaux pour un debugging facilité. |

---

## 🚀 Démarrage rapide

### Prérequis
- [Rust](https://www.rust-lang.org/) (version 1.70 ou supérieure)
- Cargo (inclus avec Rust)

### Installation
Clonez le dépôt et compilez le projet en mode release :

```bash
git clone https://github.com/MJBruno/Kastel.git
cd Kastel
cargo build --release



