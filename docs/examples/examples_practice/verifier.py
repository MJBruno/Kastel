#!/usr/bin/env python3
"""Exécute tous les exemples .ks et compare leur sortie à `attendu/`.

Usage :  python3 verifier.py [commande]     (défaut : kastel)
         python3 verifier.py kastel 09_exceptions      (un seul dossier)
"""
import glob, os, shlex, subprocess, sys

here = os.path.dirname(os.path.abspath(__file__))
cmd = shlex.split(sys.argv[1]) if len(sys.argv) > 1 else ["kastel"]
only = sys.argv[2] if len(sys.argv) > 2 else ""

ok = ko = skipped = 0
echecs = []
for path in sorted(glob.glob(os.path.join(here, "*", "*.ks"))):
    dossier = os.path.basename(os.path.dirname(path))
    if only and dossier != only:
        continue
    nom = os.path.splitext(os.path.basename(path))[0]
    attendu = os.path.join(here, "attendu", nom + ".txt")
    if not os.path.exists(attendu):
        skipped += 1
        continue
    with open(attendu, encoding="utf-8") as f:
        voulu = f.read().rstrip("\n")
    try:
        r = subprocess.run(cmd + [path], capture_output=True, text=True, timeout=30, cwd=here)
        obtenu = r.stdout.rstrip("\n")
        erreur = r.stderr.strip()
    except subprocess.TimeoutExpired:
        obtenu, erreur = "", "délai dépassé (30 s)"
    if obtenu == voulu:
        ok += 1
    else:
        ko += 1
        echecs.append((dossier, nom, voulu, obtenu, erreur))

for dossier, nom, voulu, obtenu, erreur in echecs:
    print("=" * 60)
    print("ÉCART : %s/%s" % (dossier, nom))
    print("--- attendu ---\n" + voulu)
    print("--- obtenu ---\n" + obtenu)
    if erreur:
        print("--- erreur ---\n" + erreur)

print("\n%d réussis, %d écarts, %d sans sortie de référence" % (ok, ko, skipped))
sys.exit(1 if ko else 0)
