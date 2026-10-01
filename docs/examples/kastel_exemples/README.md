# Exemples Kastel

663 exemples (dont 200 mini-projets dans les dossiers 17 à 20), un fichier `.ks` par exemple, classés par thème.

## Lancer un exemple

```
kastel 01_bases/001_hello_world.ks
```

## Vérifier tous les exemples d'un coup

```
python3 verifier.py            # utilise la commande « kastel »
python3 verifier.py "cargo run --quiet --"   # ou une autre commande
```

Le script exécute chaque exemple, compare la sortie au fichier du dossier `attendu/` et affiche un résumé.

> **Important** : ces exemples ont été écrits à la main à partir du code source de Kastel,
> sans pouvoir les exécuter (pas de compilateur Rust disponible pendant leur écriture).
> Les « sorties attendues » ont été calculées à la main. Si `verifier.py` signale un écart,
> c'est soit une erreur dans l'exemple, soit un vrai bug de Kastel : les deux sont utiles à savoir.

## Notes de syntaxe utiles

- `print` n'ajoute pas de retour à la ligne, `println` oui ; pas d'interpolation : `println("{} a {} ans", nom, age)`.
- `/` renvoie toujours un `float` ; `idiv(a, b)` donne la division entière.
- Collections : `size()`, `add()`, `contains()`, `copy()` ; `length()`, `push()`, `has()`, `items()` n'existent plus.
- Les tâches (`spawn`) n'accèdent qu'aux variables globales : les données passent par les arguments.

## Sommaire

### 01_bases — Bases du langage (37)

| N° | Fichier | Sujet |
|---|---|---|
| 001 | `01_bases/001_hello_world.ks` | Bonjour, Kastel ! |
| 002 | `01_bases/002_print_vs_println.ks` | print() contre println() |
| 003 | `01_bases/003_variables.ks` | Déclarer des variables avec let |
| 004 | `01_bases/004_annotations_de_type.ks` | Annotations de type |
| 005 | `01_bases/005_constantes.ks` | Constantes avec const |
| 006 | `01_bases/006_reaffectation.ks` | Modifier une variable |
| 007 | `01_bases/007_arithmetique.ks` | Les quatre opérations et le modulo |
| 008 | `01_bases/008_division_flottante.ks` | La division donne un flottant |
| 009 | `01_bases/009_division_entiere.ks` | Division entière avec idiv |
| 010 | `01_bases/010_affectation_composee.ks` | Opérateurs += -= *= %= |
| 011 | `01_bases/011_comparaisons.ks` | Opérateurs de comparaison |
| 012 | `01_bases/012_logique.ks` | Opérateurs logiques && || ! |
| 013 | `01_bases/013_ternaire.ks` | L'expression conditionnelle ? : |
| 014 | `01_bases/014_melange_int_float.ks` | Mélanger entiers et flottants |
| 015 | `01_bases/015_litteraux_numeriques.ks` | Littéraux numériques |
| 016 | `01_bases/016_bits.ks` | Opérateurs bit à bit |
| 017 | `01_bases/017_type_de_valeur.ks` | Connaître le type d'une valeur |
| 018 | `01_bases/018_conversions.ks` | Convertir avec int(), float(), str(), bool() |
| 019 | `01_bases/019_concatenation.ks` | Concaténer du texte avec + |
| 020 | `01_bases/020_format_simple.ks` | Formater avec {} |
| 021 | `01_bases/021_format_indexes.ks` | Champs indexés {0} {1} |
| 022 | `01_bases/022_format_precision.ks` | Nombre de décimales |
| 023 | `01_bases/023_format_largeur.ks` | Largeur et alignement |
| 024 | `01_bases/024_format_bases.ks` | Afficher en hexadécimal et en binaire |
| 025 | `01_bases/025_format_milliers.ks` | Séparateur de milliers |
| 026 | `01_bases/026_format_accolades.ks` | Afficher des accolades littérales |
| 027 | `01_bases/027_format_fonction.ks` | La fonction format() |
| 028 | `01_bases/028_arithmetique_cyclique.ks` | wrapping_add : dépassement volontaire |
| 029 | `01_bases/029_depassement_entier.ks` | Détecter un dépassement d'entier |
| 030 | `01_bases/030_min_max.ks` | min et max |
| 031 | `01_bases/031_abs_arrondis.ks` | abs, floor, ceil, round |
| 032 | `01_bases/032_portee_de_bloc.ks` | Portée d'un bloc { } |
| 033 | `01_bases/033_commentaires.ks` | Commentaires |
| 034 | `01_bases/034_valeurs_vraies_fausses.ks` | Valeurs « vraies » et « fausses » |
| 035 | `01_bases/035_inspecter.ks` | inspect() : représentation lisible |
| 036 | `01_bases/036_horloge.ks` | clock() : mesurer le temps |
| 037 | `01_bases/037_aleatoire.ks` | Nombres aléatoires |

### 02_chaines — Chaînes de caractères (35)

| N° | Fichier | Sujet |
|---|---|---|
| 038 | `02_chaines/038_taille.ks` | Longueur d'une chaîne |
| 039 | `02_chaines/039_majuscules_minuscules.ks` | upper() et lower() |
| 040 | `02_chaines/040_trim.ks` | Supprimer les espaces avec trim() |
| 041 | `02_chaines/041_contient.ks` | contains, starts_with, ends_with |
| 042 | `02_chaines/042_index_of.ks` | Chercher la position d'un texte |
| 043 | `02_chaines/043_slice.ks` | Extraire avec slice(debut, fin) |
| 044 | `02_chaines/044_substring.ks` | Extraire avec substring(debut, longueur) |
| 045 | `02_chaines/045_remplacer.ks` | replace et replace_all |
| 046 | `02_chaines/046_decouper.ks` | Découper avec split() |
| 047 | `02_chaines/047_assembler.ks` | Assembler avec join() |
| 048 | `02_chaines/048_repeter.ks` | Répéter une chaîne |
| 049 | `02_chaines/049_caractere_a.ks` | Lire un caractère |
| 050 | `02_chaines/050_parcourir_caracteres.ks` | Parcourir les caractères |
| 051 | `02_chaines/051_vers_entier.ks` | Convertir du texte en nombre |
| 052 | `02_chaines/052_erreur_conversion.ks` | Conversion invalide |
| 053 | `02_chaines/053_est_vide.ks` | is_empty() |
| 054 | `02_chaines/054_classes_de_caracteres.ks` | is_digit, is_alpha, is_alphanumeric |
| 055 | `02_chaines/055_inverser.ks` | Inverser une chaîne |
| 056 | `02_chaines/056_palindrome.ks` | Tester un palindrome |
| 057 | `02_chaines/057_compter_voyelles.ks` | Compter les voyelles |
| 058 | `02_chaines/058_compter_mots.ks` | Compter les mots |
| 059 | `02_chaines/059_capitaliser.ks` | Mettre la première lettre en majuscule |
| 060 | `02_chaines/060_titre_de_chaque_mot.ks` | Mettre chaque mot en majuscule initiale |
| 061 | `02_chaines/061_occurrences.ks` | Compter les occurrences d'un caractère |
| 062 | `02_chaines/062_frequences.ks` | Fréquence de chaque lettre |
| 063 | `02_chaines/063_anagrammes.ks` | Tester deux anagrammes |
| 064 | `02_chaines/064_cesar.ks` | Chiffre de César |
| 065 | `02_chaines/065_comparaison_de_textes.ks` | Comparer des textes |
| 066 | `02_chaines/066_construire_en_boucle.ks` | Construire un texte en boucle |
| 067 | `02_chaines/067_tableau_multiplication.ks` | Table de multiplication formatée |
| 068 | `02_chaines/068_ligne_de_titre.ks` | Encadrer un titre |
| 069 | `02_chaines/069_binaire_en_texte.ks` | Écrire un entier en binaire |
| 070 | `02_chaines/070_nettoyer_saisie.ks` | Nettoyer une saisie |
| 071 | `02_chaines/071_verifier_email_simple.ks` | Vérification très simple d'un email |
| 072 | `02_chaines/072_unicode.ks` | Texte accentué et Unicode |

### 03_controle — Structures de contrôle (29)

| N° | Fichier | Sujet |
|---|---|---|
| 073 | `03_controle/073_if_simple.ks` | if / else |
| 074 | `03_controle/074_if_else_if.ks` | Chaîne else if |
| 075 | `03_controle/075_conditions_combinees.ks` | Combiner des conditions |
| 076 | `03_controle/076_while_compteur.ks` | Boucle while |
| 077 | `03_controle/077_for_range.ks` | for avec range(n) |
| 078 | `03_controle/078_for_range_bornes.ks` | range(debut, fin) |
| 079 | `03_controle/079_for_range_pas.ks` | range avec un pas |
| 080 | `03_controle/080_for_liste.ks` | Parcourir une liste |
| 081 | `03_controle/081_break.ks` | Sortir d'une boucle avec break |
| 082 | `03_controle/082_continue.ks` | Passer à l'itération suivante |
| 083 | `03_controle/083_boucles_imbriquees.ks` | Boucles imbriquées |
| 084 | `03_controle/084_somme_1_a_n.ks` | Somme de 1 à n |
| 085 | `03_controle/085_factorielle_boucle.ks` | Factorielle avec une boucle |
| 086 | `03_controle/086_fizzbuzz.ks` | FizzBuzz |
| 087 | `03_controle/087_nombres_pairs.ks` | Lister les nombres pairs |
| 088 | `03_controle/088_collatz.ks` | Suite de Collatz |
| 089 | `03_controle/089_table_multiplication.ks` | Table de multiplication |
| 090 | `03_controle/090_devinette.ks` | Jeu de devinette (simulé) |
| 091 | `03_controle/091_premiers_avec_break.ks` | Nombre premier avec break |
| 092 | `03_controle/092_parcours_avec_index.ks` | Parcourir avec un index |
| 093 | `03_controle/093_boucle_infinie_controlee.ks` | while true avec sortie |
| 094 | `03_controle/094_pyramide.ks` | Pyramide centrée |
| 095 | `03_controle/095_damier.ks` | Damier de 4 x 4 |
| 096 | `03_controle/096_minimum_dans_liste.ks` | Minimum d'une liste (à la main) |
| 097 | `03_controle/097_continue_et_break.ks` | continue et break ensemble |
| 098 | `03_controle/098_while_avec_liste.ks` | Vider une liste avec while |
| 099 | `03_controle/099_boucle_double_sortie.ks` | Sortir de deux boucles |
| 100 | `03_controle/100_pgcd_boucle.ks` | PGCD (algorithme d'Euclide) |
| 101 | `03_controle/101_somme_chiffres.ks` | Somme des chiffres d'un nombre |

### 04_fonctions — Fonctions, fermetures, récursion (34)

| N° | Fichier | Sujet |
|---|---|---|
| 102 | `04_fonctions/102_fonction_simple.ks` | Définir et appeler une fonction |
| 103 | `04_fonctions/103_retour_de_valeur.ks` | Retourner une valeur |
| 104 | `04_fonctions/104_plusieurs_parametres.ks` | Plusieurs paramètres |
| 105 | `04_fonctions/105_sans_type.ks` | Paramètres sans annotation |
| 106 | `04_fonctions/106_retour_anticipe.ks` | Retour anticipé |
| 107 | `04_fonctions/107_recursion_factorielle.ks` | Récursion : factorielle |
| 108 | `04_fonctions/108_recursion_fibonacci.ks` | Récursion : Fibonacci |
| 109 | `04_fonctions/109_recursion_puissance.ks` | Récursion : puissance rapide |
| 110 | `04_fonctions/110_recursion_somme_liste.ks` | Récursion sur une liste |
| 111 | `04_fonctions/111_tours_de_hanoi.ks` | Tours de Hanoï |
| 112 | `04_fonctions/112_fonction_valeur.ks` | Une fonction est une valeur |
| 113 | `04_fonctions/113_fonction_en_parametre.ks` | Passer une fonction en paramètre |
| 114 | `04_fonctions/114_lambda_fleche.ks` | Fonctions fléchées |
| 115 | `04_fonctions/115_lambda_avec_bloc.ks` | Fonction fléchée avec un bloc |
| 116 | `04_fonctions/116_fonction_anonyme.ks` | Fonction anonyme avec func |
| 117 | `04_fonctions/117_fermeture_compteur.ks` | Fermeture : un compteur |
| 118 | `04_fonctions/118_fermetures_independantes.ks` | Fermetures indépendantes |
| 119 | `04_fonctions/119_fabrique_additionneur.ks` | Fabrique de fonctions |
| 120 | `04_fonctions/120_composition.ks` | Composer deux fonctions |
| 121 | `04_fonctions/121_fonction_locale.ks` | Fonction locale |
| 122 | `04_fonctions/122_surcharge.ks` | Surcharge de fonctions |
| 123 | `04_fonctions/123_surcharge_arite.ks` | Surcharge selon le nombre d'arguments |
| 124 | `04_fonctions/124_retour_tuple.ks` | Retourner plusieurs valeurs |
| 125 | `04_fonctions/125_fonction_generique.ks` | Fonction générique |
| 126 | `04_fonctions/126_premier_element_generique.ks` | Générique sur une liste |
| 127 | `04_fonctions/127_map_maison.ks` | Écrire soi-même map |
| 128 | `04_fonctions/128_predicat.ks` | Fonction prédicat |
| 129 | `04_fonctions/129_pgcd_recursif.ks` | PGCD récursif |
| 130 | `04_fonctions/130_memoisation.ks` | Mémoïsation avec un dict |
| 131 | `04_fonctions/131_recursion_mutuelle.ks` | Récursion mutuelle |
| 132 | `04_fonctions/132_profondeur_recursion.ks` | Récursion trop profonde |
| 133 | `04_fonctions/133_arite_incorrecte.ks` | Mauvais nombre d'arguments |
| 134 | `04_fonctions/134_curryfication.ks` | Curryfication |
| 135 | `04_fonctions/135_pipeline_de_fonctions.ks` | Pipeline de transformations |

### 05_listes — Listes (44)

| N° | Fichier | Sujet |
|---|---|---|
| 136 | `05_listes/136_creer_liste.ks` | Créer une liste |
| 137 | `05_listes/137_acces_par_index.ks` | Lire et modifier par index |
| 138 | `05_listes/138_index_hors_limites.ks` | Index hors limites |
| 139 | `05_listes/139_ajouter.ks` | Ajouter avec add() |
| 140 | `05_listes/140_inserer.ks` | Insérer à une position |
| 141 | `05_listes/141_retirer_valeur.ks` | Retirer une valeur avec remove() |
| 142 | `05_listes/142_retirer_position.ks` | Retirer par position |
| 143 | `05_listes/143_pop.ks` | Dépiler avec pop() |
| 144 | `05_listes/144_premier_dernier.ks` | first() et last() |
| 145 | `05_listes/145_contient_et_position.ks` | contains et index_of |
| 146 | `05_listes/146_tranche.ks` | Extraire une sous-liste avec slice |
| 147 | `05_listes/147_inverser.ks` | Inverser une liste |
| 148 | `05_listes/148_trier_nombres.ks` | Trier une liste de nombres |
| 149 | `05_listes/149_trier_textes.ks` | Trier une liste de textes |
| 150 | `05_listes/150_trier_decroissant.ks` | Trier par ordre décroissant |
| 151 | `05_listes/151_joindre.ks` | Transformer une liste en texte |
| 152 | `05_listes/152_vider.ks` | Vider et tester si vide |
| 153 | `05_listes/153_get_set.ks` | get() et set() |
| 154 | `05_listes/154_reference_partagee.ks` | Les listes sont partagées |
| 155 | `05_listes/155_copier.ks` | Copier une liste avec copy() |
| 156 | `05_listes/156_map.ks` | map : transformer chaque élément |
| 157 | `05_listes/157_filter.ks` | filter : garder certains éléments |
| 158 | `05_listes/158_reduce.ks` | reduce : réduire à une valeur |
| 159 | `05_listes/159_any_all.ks` | any et all |
| 160 | `05_listes/160_chainage.ks` | Chaîner map, filter, reduce |
| 161 | `05_listes/161_liste_de_listes.ks` | Listes imbriquées |
| 162 | `05_listes/162_parcourir_matrice.ks` | Parcourir une matrice |
| 163 | `05_listes/163_transposee.ks` | Transposée d'une matrice |
| 164 | `05_listes/164_liste_de_range.ks` | Construire une liste depuis range |
| 165 | `05_listes/165_somme_moyenne.ks` | Somme et moyenne |
| 166 | `05_listes/166_maximum.ks` | Trouver le maximum |
| 167 | `05_listes/167_doublons.ks` | Supprimer les doublons |
| 168 | `05_listes/168_aplatir.ks` | Aplatir une liste de listes |
| 169 | `05_listes/169_pile.ks` | Une pile (LIFO) |
| 170 | `05_listes/170_file.ks` | Une file (FIFO) |
| 171 | `05_listes/171_zip_maison.ks` | Assembler deux listes |
| 172 | `05_listes/172_rotation.ks` | Rotation d'une liste |
| 173 | `05_listes/173_trier_par_cle.ks` | Trier des enregistrements |
| 174 | `05_listes/174_chunks.ks` | Découper en paquets |
| 175 | `05_listes/175_compter_element.ks` | Compter les occurrences |
| 176 | `05_listes/176_recherche_binaire.ks` | Recherche dichotomique |
| 177 | `05_listes/177_tuple_dans_liste.ks` | Liste de tuples |
| 178 | `05_listes/178_iterateur_liste.ks` | Un itérateur explicite |
| 179 | `05_listes/179_liste_alias_fonction.ks` | Modifier une liste dans une fonction |

### 06_dict_set_tuple_record — Dict, Set, Tuple, Record (44)

| N° | Fichier | Sujet |
|---|---|---|
| 180 | `06_dict_set_tuple_record/180_dict_creer.ks` | Créer un dictionnaire |
| 181 | `06_dict_set_tuple_record/181_dict_lire.ks` | Lire une valeur |
| 182 | `06_dict_set_tuple_record/182_dict_ecrire.ks` | Ajouter et modifier |
| 183 | `06_dict_set_tuple_record/183_dict_set.ks` | set() et get() |
| 184 | `06_dict_set_tuple_record/184_dict_contient.ks` | Tester l'existence d'une clé |
| 185 | `06_dict_set_tuple_record/185_dict_get_or.ks` | Valeur par défaut avec get_or |
| 186 | `06_dict_set_tuple_record/186_dict_retirer.ks` | Supprimer une entrée |
| 187 | `06_dict_set_tuple_record/187_dict_cles_valeurs.ks` | keys() et values() |
| 188 | `06_dict_set_tuple_record/188_dict_parcourir.ks` | Parcourir un dict |
| 189 | `06_dict_set_tuple_record/189_dict_vide_et_taille.ks` | Dict vide, is_empty, clear |
| 190 | `06_dict_set_tuple_record/190_dict_update.ks` | Fusionner deux dicts |
| 191 | `06_dict_set_tuple_record/191_dict_copie.ks` | Copier un dict |
| 192 | `06_dict_set_tuple_record/192_dict_compteur_mots.ks` | Compter les mots |
| 193 | `06_dict_set_tuple_record/193_dict_imbrique.ks` | Dict imbriqué |
| 194 | `06_dict_set_tuple_record/194_dict_inverse.ks` | Inverser un dict |
| 195 | `06_dict_set_tuple_record/195_dict_type_annote.ks` | Annoter un dict |
| 196 | `06_dict_set_tuple_record/196_dict_cle_absente.ks` | Clé absente : erreur |
| 197 | `06_dict_set_tuple_record/197_set_creer.ks` | Créer un ensemble |
| 198 | `06_dict_set_tuple_record/198_set_ajouter_retirer.ks` | add, remove, contains |
| 199 | `06_dict_set_tuple_record/199_set_union.ks` | Union |
| 200 | `06_dict_set_tuple_record/200_set_intersection.ks` | Intersection |
| 201 | `06_dict_set_tuple_record/201_set_difference.ks` | Différence |
| 202 | `06_dict_set_tuple_record/202_set_symetrique.ks` | Différence symétrique |
| 203 | `06_dict_set_tuple_record/203_set_sous_ensemble.ks` | Sous-ensemble et sur-ensemble |
| 204 | `06_dict_set_tuple_record/204_set_parcourir.ks` | Parcourir un ensemble |
| 205 | `06_dict_set_tuple_record/205_set_vers_liste.ks` | Ensemble vers liste |
| 206 | `06_dict_set_tuple_record/206_set_reference_copie.ks` | Ensemble : référence et copie |
| 207 | `06_dict_set_tuple_record/207_set_vide.ks` | Ensemble vide |
| 208 | `06_dict_set_tuple_record/208_set_egalite.ks` | Comparer deux ensembles |
| 209 | `06_dict_set_tuple_record/209_tuple_creer.ks` | Créer un tuple |
| 210 | `06_dict_set_tuple_record/210_tuple_un_element.ks` | Tuple à un élément |
| 211 | `06_dict_set_tuple_record/211_tuple_taille_contient.ks` | size, contains, index_of |
| 212 | `06_dict_set_tuple_record/212_tuple_destructuration_match.ks` | Décomposer un tuple avec match |
| 213 | `06_dict_set_tuple_record/213_tuple_comme_cle.ks` | Comparer des tuples |
| 214 | `06_dict_set_tuple_record/214_tuple_vers_liste.ks` | Tuple vers liste |
| 215 | `06_dict_set_tuple_record/215_record_creer.ks` | Créer un enregistrement (record) |
| 216 | `06_dict_set_tuple_record/216_record_modifier.ks` | Modifier un champ |
| 217 | `06_dict_set_tuple_record/217_record_type_alias.ks` | Nommer un type de record |
| 218 | `06_dict_set_tuple_record/218_record_en_parametre.ks` | Record en paramètre |
| 219 | `06_dict_set_tuple_record/219_record_cles_valeurs.ks` | keys() et values() d'un record |
| 220 | `06_dict_set_tuple_record/220_record_copie.ks` | Copier un record |
| 221 | `06_dict_set_tuple_record/221_record_imbrique.ks` | Records imbriqués |
| 222 | `06_dict_set_tuple_record/222_record_contre_dict.ks` | Record ou dict ? |
| 223 | `06_dict_set_tuple_record/223_liste_de_records.ks` | Liste de records |

### 07_classes_enums — Classes, interfaces, enums, alias (32)

| N° | Fichier | Sujet |
|---|---|---|
| 224 | `07_classes_enums/224_classe_simple.ks` | Une première classe |
| 225 | `07_classes_enums/225_constructeur.ks` | Le constructeur initialize |
| 226 | `07_classes_enums/226_constructeur_surcharge.ks` | Constructeur surchargé |
| 227 | `07_classes_enums/227_champs_modifiables.ks` | Modifier les champs d'un objet |
| 228 | `07_classes_enums/228_methodes_qui_modifient.ks` | Méthodes qui modifient l'objet |
| 229 | `07_classes_enums/229_champ_prive.ks` | Champs privés et encapsulation |
| 230 | `07_classes_enums/230_methode_privee.ks` | Méthode privée |
| 231 | `07_classes_enums/231_membre_protege.ks` | Membre protected |
| 232 | `07_classes_enums/232_heritage.ks` | Héritage |
| 233 | `07_classes_enums/233_redefinition.ks` | Redéfinir une méthode |
| 234 | `07_classes_enums/234_interface_simple.ks` | Une interface |
| 235 | `07_classes_enums/235_interface_en_parametre.ks` | Interface comme type de paramètre |
| 236 | `07_classes_enums/236_static_champ_methode.ks` | Membres static |
| 237 | `07_classes_enums/237_static_compteur.ks` | Compter les instances |
| 238 | `07_classes_enums/238_objets_en_liste.ks` | Liste d'objets |
| 239 | `07_classes_enums/239_reference_objet.ks` | Les objets sont partagés |
| 240 | `07_classes_enums/240_methode_chainee.ks` | Chaîner les appels |
| 241 | `07_classes_enums/241_classe_pile.ks` | Une pile (classe) |
| 242 | `07_classes_enums/242_classe_vecteur.ks` | Vecteur 2D |
| 243 | `07_classes_enums/243_compte_bancaire.ks` | Compte bancaire avec erreurs |
| 244 | `07_classes_enums/244_erreur_personnalisee.ks` | Classe d'erreur personnalisée |
| 245 | `07_classes_enums/245_singleton.ks` | Un singleton par static |
| 246 | `07_classes_enums/246_polymorphisme.ks` | Polymorphisme par interface |
| 247 | `07_classes_enums/247_enum_simple.ks` | Une énumération |
| 248 | `07_classes_enums/248_enum_match.ks` | Enum et match |
| 249 | `07_classes_enums/249_enum_methode.ks` | Enum avec méthode |
| 250 | `07_classes_enums/250_enum_jours.ks` | Jours de la semaine |
| 251 | `07_classes_enums/251_enum_dans_liste.ks` | Enum dans une liste |
| 252 | `07_classes_enums/252_alias_de_type.ks` | Alias de type |
| 253 | `07_classes_enums/253_type_union.ks` | Type union |
| 254 | `07_classes_enums/254_union_texte_nombre.ks` | Union de types différents |
| 255 | `07_classes_enums/255_alias_liste.ks` | Alias pour une liste |

### 08_match_option_result — match, Option, Result (32)

| N° | Fichier | Sujet |
|---|---|---|
| 256 | `08_match_option_result/256_match_litteraux.ks` | match sur des valeurs |
| 257 | `08_match_option_result/257_match_ou.ks` | Motifs alternatifs avec | |
| 258 | `08_match_option_result/258_match_intervalles.ks` | Intervalles dans un motif |
| 259 | `08_match_option_result/259_match_garde.ks` | Garde avec if |
| 260 | `08_match_option_result/260_match_texte.ks` | match sur des textes |
| 261 | `08_match_option_result/261_match_booleen.ks` | match sur un booléen |
| 262 | `08_match_option_result/262_match_liste.ks` | Décomposer une liste |
| 263 | `08_match_option_result/263_match_liste_exacte.ks` | Liste de taille exacte |
| 264 | `08_match_option_result/264_match_tuple.ks` | Décomposer un tuple |
| 265 | `08_match_option_result/265_match_effet.ks` | match comme instruction |
| 266 | `08_match_option_result/266_option_bases.ks` | Option : une valeur peut manquer |
| 267 | `08_match_option_result/267_option_unwrap.ks` | unwrap et unwrap_or |
| 268 | `08_match_option_result/268_option_unwrap_erreur.ks` | unwrap sur None |
| 269 | `08_match_option_result/269_option_map.ks` | map sur un Option |
| 270 | `08_match_option_result/270_option_and_then.ks` | and_then : enchaîner des Option |
| 271 | `08_match_option_result/271_option_ok_or.ks` | ok_or : Option vers Result |
| 272 | `08_match_option_result/272_option_chercher.ks` | Fonction qui peut échouer : Option |
| 273 | `08_match_option_result/273_option_match.ks` | Option avec match |
| 274 | `08_match_option_result/274_option_propagation.ks` | L'opérateur ? sur Option |
| 275 | `08_match_option_result/275_option_dict.ks` | Chercher dans un dict |
| 276 | `08_match_option_result/276_result_bases.ks` | Result : réussite ou erreur |
| 277 | `08_match_option_result/277_result_diviser.ks` | Diviser sans planter |
| 278 | `08_match_option_result/278_result_unwrap_or.ks` | unwrap, unwrap_err, unwrap_or |
| 279 | `08_match_option_result/279_result_map.ks` | map et map_err |
| 280 | `08_match_option_result/280_result_and_then.ks` | and_then sur Result |
| 281 | `08_match_option_result/281_result_match.ks` | Result avec match |
| 282 | `08_match_option_result/282_result_propagation.ks` | L'opérateur ? sur Result |
| 283 | `08_match_option_result/283_result_ok_err.ks` | ok() et err() |
| 284 | `08_match_option_result/284_result_expect.ks` | expect : extraire avec un message |
| 285 | `08_match_option_result/285_result_liste.ks` | Traiter une liste de Result |
| 286 | `08_match_option_result/286_result_avec_finally.ks` | ? et finally ensemble |
| 287 | `08_match_option_result/287_option_imbriquee.ks` | Option imbriqué avec match |

### 09_exceptions — Exceptions try/catch/finally (28)

| N° | Fichier | Sujet |
|---|---|---|
| 288 | `09_exceptions/288_try_catch_base.ks` | try / catch |
| 289 | `09_exceptions/289_erreur_kind_message.ks` | kind, message et to_string() |
| 290 | `09_exceptions/290_throw_texte.ks` | Lancer un texte avec throw |
| 291 | `09_exceptions/291_throw_nombre.ks` | Lancer un nombre |
| 292 | `09_exceptions/292_throw_record.ks` | Lancer un record |
| 293 | `09_exceptions/293_finally_toujours.ks` | finally s'exécute toujours |
| 294 | `09_exceptions/294_try_finally_seul.ks` | try / finally sans catch |
| 295 | `09_exceptions/295_propagation_appels.ks` | Propagation à travers les fonctions |
| 296 | `09_exceptions/296_catch_imbriques.ks` | try imbriqués |
| 297 | `09_exceptions/297_erreur_dans_catch_finally.ks` | Erreur dans catch : finally exécuté |
| 298 | `09_exceptions/298_handler_apres_catch.ks` | Un catch n'intercepte plus après avoir fini |
| 299 | `09_exceptions/299_return_dans_try.ks` | return dans try : finally exécuté |
| 300 | `09_exceptions/300_return_dans_finally.ks` | return dans finally |
| 301 | `09_exceptions/301_break_dans_try.ks` | break et continue dans try |
| 302 | `09_exceptions/302_erreur_type.ks` | TypeError |
| 303 | `09_exceptions/303_debordement_entier.ks` | IntegerOverflow |
| 304 | `09_exceptions/304_erreur_appel.ks` | NotCallable |
| 305 | `09_exceptions/305_validation_entree.ks` | Valider avec throw |
| 306 | `09_exceptions/306_reessayer.ks` | Réessayer après une erreur |
| 307 | `09_exceptions/307_ressource_liberee.ks` | Libérer une ressource dans finally |
| 308 | `09_exceptions/308_erreur_dans_boucle_hot.ks` | Erreur dans une boucle rapide |
| 309 | `09_exceptions/309_fermeture_apres_erreur.ks` | Une fermeture survit à une exception |
| 310 | `09_exceptions/310_catch_sans_type.ks` | catch (e) attrape aussi les erreurs internes |
| 311 | `09_exceptions/311_erreur_kind_liste.ks` | Distinguer plusieurs erreurs |
| 312 | `09_exceptions/312_propager_avec_contexte.ks` | Ajouter du contexte puis relancer |
| 313 | `09_exceptions/313_erreurs_et_taches.ks` | Erreur dans une tâche, récupérée par join |
| 314 | `09_exceptions/314_finally_dans_tache_annulee.ks` | finally et annulation de tâche |
| 315 | `09_exceptions/315_finally_libere_mutex.ks` | finally libère un verrou |

### 10_generiques_operateurs — Génériques et surcharge d'opérateurs (20)

| N° | Fichier | Sujet |
|---|---|---|
| 316 | `10_generiques_operateurs/316_generique_identite.ks` | Fonction générique |
| 317 | `10_generiques_operateurs/317_generique_paire.ks` | Paire générique |
| 318 | `10_generiques_operateurs/318_generique_liste.ks` | Générique sur List<T> |
| 319 | `10_generiques_operateurs/319_generique_explicite.ks` | Préciser le type à l'appel |
| 320 | `10_generiques_operateurs/320_generique_borne_add.ks` | Borne Add : accepter l'addition |
| 321 | `10_generiques_operateurs/321_generique_borne_mul.ks` | Borne Mul |
| 322 | `10_generiques_operateurs/322_generique_borne_ord.ks` | Borne Ord : comparer |
| 323 | `10_generiques_operateurs/323_generique_multi_bornes.ks` | Plusieurs bornes avec + |
| 324 | `10_generiques_operateurs/324_generique_borne_interface.ks` | Borne par interface |
| 325 | `10_generiques_operateurs/325_classe_generique.ks` | Classe générique |
| 326 | `10_generiques_operateurs/326_methode_generique.ks` | Méthode générique dans une classe |
| 327 | `10_generiques_operateurs/327_alias_generique.ks` | Alias de type générique |
| 328 | `10_generiques_operateurs/328_operateur_add.ks` | Surcharger + avec Add |
| 329 | `10_generiques_operateurs/329_operateur_sub.ks` | Surcharger - avec Sub |
| 330 | `10_generiques_operateurs/330_operateur_mul.ks` | Surcharger * avec Mul |
| 331 | `10_generiques_operateurs/331_operateur_eq.ks` | Surcharger == avec Eq |
| 332 | `10_generiques_operateurs/332_operateur_ord.ks` | Surcharger < avec Ord |
| 333 | `10_generiques_operateurs/333_operateur_generique_classe.ks` | Fonction générique avec une classe Add |
| 334 | `10_generiques_operateurs/334_operateur_div_heterogene.ks` | Opérateur hétérogène |
| 335 | `10_generiques_operateurs/335_interface_generique.ks` | Interface générique |

### 11_iterateurs_range — Itérateurs et range (13)

| N° | Fichier | Sujet |
|---|---|---|
| 336 | `11_iterateurs_range/336_range_taille.ks` | Un range est une valeur |
| 337 | `11_iterateurs_range/337_range_en_liste.ks` | range vers liste |
| 338 | `11_iterateurs_range/338_range_somme.ks` | Somme d'un range |
| 339 | `11_iterateurs_range/339_iter_manuel.ks` | Parcours manuel avec iter() |
| 340 | `11_iterateurs_range/340_iter_peek.ks` | peek : regarder sans avancer |
| 341 | `11_iterateurs_range/341_iter_map_collect.ks` | Itérateur : map puis collect |
| 342 | `11_iterateurs_range/342_iter_filter.ks` | Itérateur : filter |
| 343 | `11_iterateurs_range/343_iter_take_skip.ks` | take et skip |
| 344 | `11_iterateurs_range/344_iter_count_any_all.ks` | count, any, all |
| 345 | `11_iterateurs_range/345_iter_chaine.ks` | Pipeline complet |
| 346 | `11_iterateurs_range/346_iter_dict.ks` | Itérer un dict |
| 347 | `11_iterateurs_range/347_classe_iterable.ks` | Une classe itérable |
| 348 | `11_iterateurs_range/348_iterateur_chaine.ks` | Itérer une chaîne |

### 12_async_await — async / await (16)

| N° | Fichier | Sujet |
|---|---|---|
| 349 | `12_async_await/349_async_base.ks` | async func et await |
| 350 | `12_async_await/350_async_inline.ks` | await directement sur l'appel |
| 351 | `12_async_await/351_async_parallele.ks` | Lancer plusieurs tâches puis attendre |
| 352 | `12_async_await/352_async_chaine.ks` | Chaîne d'await |
| 353 | `12_async_await/353_async_erreur.ks` | Une erreur remonte par await |
| 354 | `12_async_await/354_async_erreur_runtime.ks` | Erreur runtime dans une tâche |
| 355 | `12_async_await/355_async_plusieurs_attentes.ks` | Plusieurs tâches attendent la même |
| 356 | `12_async_await/356_async_type_deduit.ks` | Type de retour déduit |
| 357 | `12_async_await/357_async_tableau_de_taches.ks` | Une liste de tâches |
| 358 | `12_async_await/358_async_pas_dattente.ks` | Une tâche non attendue s'exécute quand même |
| 359 | `12_async_await/359_async_annulation.ks` | Annuler une tâche attendue |
| 360 | `12_async_await/360_async_mauvais_await.ks` | await sur autre chose qu'une tâche |
| 361 | `12_async_await/361_async_pipeline.ks` | Pipeline de tâches |
| 362 | `12_async_await/362_async_avec_canal.ks` | async et canal ensemble |
| 363 | `12_async_await/363_async_recursion_fib.ks` | Fibonacci asynchrone |
| 364 | `12_async_await/364_async_finally.ks` | finally dans une fonction async |

### 13_concurrence — Concurrence (tâches, canaux, verrous) (35)

| N° | Fichier | Sujet |
|---|---|---|
| 365 | `13_concurrence/365_spawn_join.ks` | spawn et join |
| 366 | `13_concurrence/366_spawn_plusieurs.ks` | Plusieurs tâches |
| 367 | `13_concurrence/367_task_status.ks` | Suivre l'état d'une tâche |
| 368 | `13_concurrence/368_yield_cooperatif.ks` | yield : céder la main |
| 369 | `13_concurrence/369_sleep_tache.ks` | sleep dans une tâche |
| 370 | `13_concurrence/370_canal_base.ks` | Canal : send et recv |
| 371 | `13_concurrence/371_canal_ordre.ks` | Ordre garanti |
| 372 | `13_concurrence/372_canal_borne.ks` | Canal borné |
| 373 | `13_concurrence/373_canal_try_recv.ks` | try_recv : sans attendre |
| 374 | `13_concurrence/374_canal_try_send.ks` | try_send sur canal plein |
| 375 | `13_concurrence/375_canal_fermeture.ks` | Fermer un canal |
| 376 | `13_concurrence/376_canal_ferme_recv.ks` | Recevoir sur un canal fermé |
| 377 | `13_concurrence/377_producteur_consommateur.ks` | Producteur / consommateur |
| 378 | `13_concurrence/378_pipeline_canaux.ks` | Pipeline à deux étages |
| 379 | `13_concurrence/379_travailleurs_partages.ks` | Plusieurs travailleurs, un canal |
| 380 | `13_concurrence/380_select_base.ks` | select : attendre plusieurs canaux |
| 381 | `13_concurrence/381_select_timeout.ks` | select avec délai |
| 382 | `13_concurrence/382_select_ferme.ks` | select et canal fermé |
| 383 | `13_concurrence/383_select_envoi.ks` | select en émission |
| 384 | `13_concurrence/384_mutex_base.ks` | Mutex : exclusion mutuelle |
| 385 | `13_concurrence/385_mutex_try_lock.ks` | try_lock |
| 386 | `13_concurrence/386_mutex_finally.ks` | Toujours déverrouiller avec finally |
| 387 | `13_concurrence/387_mutex_deadlock.ks` | Détection de blocage sur soi-même |
| 388 | `13_concurrence/388_semaphore_base.ks` | Sémaphore |
| 389 | `13_concurrence/389_semaphore_limite.ks` | Limiter la concurrence |
| 390 | `13_concurrence/390_wait_group_base.ks` | WaitGroup : attendre un groupe |
| 391 | `13_concurrence/391_wait_group_underflow.ks` | done() en trop |
| 392 | `13_concurrence/392_barrier_base.ks` | Barrier : rendez-vous |
| 393 | `13_concurrence/393_barrier_invalide.ks` | Barrier de taille invalide |
| 394 | `13_concurrence/394_rwlock_base.ks` | RwLock : lecteurs multiples, un écrivain |
| 395 | `13_concurrence/395_event_base.ks` | Event : un signal |
| 396 | `13_concurrence/396_event_reset.ks` | reset d'un event |
| 397 | `13_concurrence/397_condvar_base.ks` | Variable de condition |
| 398 | `13_concurrence/398_annuler_tache.ks` | Annuler une tâche |
| 399 | `13_concurrence/399_capture_interdite.ks` | Pas de capture de variable locale |

### 14_maths_algorithmes — Maths et algorithmes (32)

| N° | Fichier | Sujet |
|---|---|---|
| 400 | `14_maths_algorithmes/400_racine_carree.ks` | Racine carrée et puissance |
| 401 | `14_maths_algorithmes/401_trigonometrie.ks` | Trigonométrie |
| 402 | `14_maths_algorithmes/402_exp_log.ks` | Exponentielle et logarithmes |
| 403 | `14_maths_algorithmes/403_arrondis_floats.ks` | Arrondir un flottant |
| 404 | `14_maths_algorithmes/404_aire_cercle.ks` | Aire d'un cercle |
| 405 | `14_maths_algorithmes/405_distance.ks` | Distance entre deux points |
| 406 | `14_maths_algorithmes/406_equation_second_degre.ks` | Équation du second degré |
| 407 | `14_maths_algorithmes/407_conversion_temperature.ks` | Celsius vers Fahrenheit |
| 408 | `14_maths_algorithmes/408_nombre_premier.ks` | Test de primalité |
| 409 | `14_maths_algorithmes/409_crible_eratosthene.ks` | Crible d'Ératosthène |
| 410 | `14_maths_algorithmes/410_decomposition_facteurs.ks` | Décomposition en facteurs premiers |
| 411 | `14_maths_algorithmes/411_ppcm.ks` | PPCM |
| 412 | `14_maths_algorithmes/412_nombres_parfaits.ks` | Nombres parfaits |
| 413 | `14_maths_algorithmes/413_fibonacci_iteratif.ks` | Fibonacci itératif |
| 414 | `14_maths_algorithmes/414_binaire_decimal.ks` | Décimal vers binaire (à la main) |
| 415 | `14_maths_algorithmes/415_binaire_vers_decimal.ks` | Binaire vers décimal |
| 416 | `14_maths_algorithmes/416_moyenne_ecart_type.ks` | Moyenne et écart-type |
| 417 | `14_maths_algorithmes/417_mediane.ks` | Médiane |
| 418 | `14_maths_algorithmes/418_tri_a_bulles.ks` | Tri à bulles |
| 419 | `14_maths_algorithmes/419_tri_selection.ks` | Tri par sélection |
| 420 | `14_maths_algorithmes/420_tri_insertion.ks` | Tri par insertion |
| 421 | `14_maths_algorithmes/421_tri_fusion.ks` | Tri fusion |
| 422 | `14_maths_algorithmes/422_tri_rapide.ks` | Tri rapide |
| 423 | `14_maths_algorithmes/423_recherche_lineaire.ks` | Recherche linéaire avec Option |
| 424 | `14_maths_algorithmes/424_pascal.ks` | Triangle de Pascal |
| 425 | `14_maths_algorithmes/425_matrice_produit.ks` | Produit de matrices |
| 426 | `14_maths_algorithmes/426_chiffres_romains.ks` | Nombre vers chiffres romains |
| 427 | `14_maths_algorithmes/427_parentheses_equilibrees.ks` | Parenthèses équilibrées |
| 428 | `14_maths_algorithmes/428_conversion_base.ks` | Changer de base |
| 429 | `14_maths_algorithmes/429_jeu_de_la_vie_etape.ks` | Jeu de la vie : une étape |
| 430 | `14_maths_algorithmes/430_nombres_amicaux.ks` | Somme des diviseurs |
| 431 | `14_maths_algorithmes/431_bits_compter.ks` | Compter les bits à 1 |

### 15_fichiers_json_systeme — Fichiers, JSON, système (18)

| N° | Fichier | Sujet |
|---|---|---|
| 432 | `15_fichiers_json_systeme/432_ecrire_lire_fichier.ks` | Écrire puis lire un fichier |
| 433 | `15_fichiers_json_systeme/433_ajouter_fichier.ks` | Ajouter à la fin d'un fichier |
| 434 | `15_fichiers_json_systeme/434_lire_lignes.ks` | Lire ligne par ligne |
| 435 | `15_fichiers_json_systeme/435_fichier_existe.ks` | Tester l'existence |
| 436 | `15_fichiers_json_systeme/436_taille_fichier.ks` | Taille d'un fichier |
| 437 | `15_fichiers_json_systeme/437_fichier_absent.ks` | Lire un fichier absent |
| 438 | `15_fichiers_json_systeme/438_lecture_sure.ks` | Lecture sûre avec Result |
| 439 | `15_fichiers_json_systeme/439_compter_lignes_mots.ks` | Statistiques d'un fichier |
| 440 | `15_fichiers_json_systeme/440_chemins.ks` | Manipuler des chemins |
| 441 | `15_fichiers_json_systeme/441_joindre_chemins.ks` | Assembler un chemin |
| 442 | `15_fichiers_json_systeme/442_json_decoder.ks` | Lire du JSON |
| 443 | `15_fichiers_json_systeme/443_json_encoder.ks` | Écrire du JSON |
| 444 | `15_fichiers_json_systeme/444_json_fichier.ks` | Sauvegarder des données en JSON |
| 445 | `15_fichiers_json_systeme/445_json_invalide.ks` | JSON invalide |
| 446 | `15_fichiers_json_systeme/446_systeme_os.ks` | Informations système |
| 447 | `15_fichiers_json_systeme/447_arguments_programme.ks` | Arguments de la ligne de commande |
| 448 | `15_fichiers_json_systeme/448_code_de_sortie.ks` | Terminer avec un code |
| 449 | `15_fichiers_json_systeme/449_saisie_utilisateur.ks` | Lire au clavier |

### 16_mini_projets — Mini-projets (14)

| N° | Fichier | Sujet |
|---|---|---|
| 450 | `16_mini_projets/450_todo_liste.ks` | Liste de tâches |
| 451 | `16_mini_projets/451_compte_bancaire_result.ks` | Banque avec Result |
| 452 | `16_mini_projets/452_inventaire.ks` | Inventaire de magasin |
| 453 | `16_mini_projets/453_calculatrice_npi.ks` | Calculatrice en notation polonaise inversée |
| 454 | `16_mini_projets/454_morpion_victoire.ks` | Morpion : détecter une victoire |
| 455 | `16_mini_projets/455_notes_etudiants.ks` | Bulletin de notes |
| 456 | `16_mini_projets/456_machine_a_etats.ks` | Machine à états avec enum |
| 457 | `16_mini_projets/457_frequence_mots.ks` | Mots les plus fréquents |
| 458 | `16_mini_projets/458_evenements.ks` | Système d'événements |
| 459 | `16_mini_projets/459_cache_lru_simple.ks` | Petit cache |
| 460 | `16_mini_projets/460_file_priorite.ks` | File d'attente à priorités |
| 461 | `16_mini_projets/461_parcours_graphe.ks` | Parcours en largeur d'un graphe |
| 462 | `16_mini_projets/462_labyrinthe_plus_court.ks` | Plus court chemin dans une grille |
| 463 | `16_mini_projets/463_serveur_simule.ks` | Petit serveur simulé avec tâches |

### 17_projets_jeux_simulations — Mini-projets : jeux et simulations (50)

| N° | Fichier | Sujet |
|---|---|---|
| 464 | `17_projets_jeux_simulations/464_p_morpion_rendu.ks` | Morpion : afficher la grille |
| 465 | `17_projets_jeux_simulations/465_p_morpion_ia.ks` | Morpion : coup de l'ordinateur |
| 466 | `17_projets_jeux_simulations/466_p_puissance4.ks` | Puissance 4 : détecter quatre alignés |
| 467 | `17_projets_jeux_simulations/467_p_pendu.ks` | Le pendu |
| 468 | `17_projets_jeux_simulations/468_p_mastermind.ks` | Mastermind : évaluer une proposition |
| 469 | `17_projets_jeux_simulations/469_p_taureaux_vaches.ks` | Taureaux et vaches |
| 470 | `17_projets_jeux_simulations/470_p_nim.ks` | Jeu de Nim : stratégie gagnante |
| 471 | `17_projets_jeux_simulations/471_p_batons_21.ks` | Jeu des 21 bâtons |
| 472 | `17_projets_jeux_simulations/472_p_pierre_feuille_ciseaux.ks` | Pierre, feuille, ciseaux |
| 473 | `17_projets_jeux_simulations/473_p_blackjack.ks` | Blackjack : valeur d'une main |
| 474 | `17_projets_jeux_simulations/474_p_poker_main.ks` | Poker : reconnaître une main |
| 475 | `17_projets_jeux_simulations/475_p_yams.ks` | Yams : calculer un score |
| 476 | `17_projets_jeux_simulations/476_p_loterie.ks` | Loterie : nombre de combinaisons |
| 477 | `17_projets_jeux_simulations/477_p_de_alea.ks` | Dé pseudo-aléatoire |
| 478 | `17_projets_jeux_simulations/478_p_melange_cartes.ks` | Mélange de cartes (Fisher-Yates) |
| 479 | `17_projets_jeux_simulations/479_p_jeu_de_la_vie_planeur.ks` | Jeu de la vie : un planeur |
| 480 | `17_projets_jeux_simulations/480_p_fourmi_langton.ks` | Fourmi de Langton |
| 481 | `17_projets_jeux_simulations/481_p_snake.ks` | Snake : déplacer le serpent |
| 482 | `17_projets_jeux_simulations/482_p_robot_grille.ks` | Robot sur une grille |
| 483 | `17_projets_jeux_simulations/483_p_n_reines.ks` | Problème des N reines |
| 484 | `17_projets_jeux_simulations/484_p_sudoku_4x4.ks` | Sudoku 4 x 4 : valider une grille |
| 485 | `17_projets_jeux_simulations/485_p_taquin_solvable.ks` | Taquin : la grille est-elle solvable ? |
| 486 | `17_projets_jeux_simulations/486_p_cavalier.ks` | Cavalier d'échecs : coups possibles |
| 487 | `17_projets_jeux_simulations/487_p_hanoi_coups.ks` | Tours de Hanoï : lister les déplacements |
| 488 | `17_projets_jeux_simulations/488_p_serpents_echelles.ks` | Serpents et échelles |
| 489 | `17_projets_jeux_simulations/489_p_2048_ligne.ks` | 2048 : fusionner une ligne |
| 490 | `17_projets_jeux_simulations/490_p_tetris_lignes.ks` | Tétris : effacer les lignes complètes |
| 491 | `17_projets_jeux_simulations/491_p_pong.ks` | Pong : rebonds d'une balle |
| 492 | `17_projets_jeux_simulations/492_p_demineur.ks` | Démineur : compter les mines voisines |
| 493 | `17_projets_jeux_simulations/493_p_labyrinthe_existe.ks` | Labyrinthe : existe-t-il un chemin ? |
| 494 | `17_projets_jeux_simulations/494_p_bataille_navale.ks` | Bataille navale : tirs |
| 495 | `17_projets_jeux_simulations/495_p_course_tortues.ks` | Course de tortues |
| 496 | `17_projets_jeux_simulations/496_p_roulette.ks` | Roulette : couleur d'un numéro |
| 497 | `17_projets_jeux_simulations/497_p_craps.ks` | Craps : règles du jeu |
| 498 | `17_projets_jeux_simulations/498_p_bowling.ks` | Bowling : calcul du score |
| 499 | `17_projets_jeux_simulations/499_p_echiquier.ks` | Dessiner un échiquier |
| 500 | `17_projets_jeux_simulations/500_p_memory.ks` | Jeu de mémoire (paires) |
| 501 | `17_projets_jeux_simulations/501_p_simon.ks` | Simon : mémoriser une séquence |
| 502 | `17_projets_jeux_simulations/502_p_chifoumi_tournoi.ks` | Tournoi à élimination |
| 503 | `17_projets_jeux_simulations/503_p_deux_sommes_cible.ks` | Compte est bon simplifié |
| 504 | `17_projets_jeux_simulations/504_p_ascenseur.ks` | Simulation d'ascenseur |
| 505 | `17_projets_jeux_simulations/505_p_feu_tricolore.ks` | Feu tricolore avec durées |
| 506 | `17_projets_jeux_simulations/506_p_distributeur.ks` | Distributeur de boissons |
| 507 | `17_projets_jeux_simulations/507_p_course_voitures.ks` | Course de voitures |
| 508 | `17_projets_jeux_simulations/508_p_regle_90.ks` | Automate cellulaire (règle 90) |
| 509 | `17_projets_jeux_simulations/509_p_jeu_pendule.ks` | Chasse au trésor sur grille |
| 510 | `17_projets_jeux_simulations/510_p_loto_grille.ks` | Grille de loto : contrôle des gains |
| 511 | `17_projets_jeux_simulations/511_p_carre_magique.ks` | Carré magique |
| 512 | `17_projets_jeux_simulations/512_p_labyrinthe_dessin.ks` | Dessiner une carte au trésor |
| 513 | `17_projets_jeux_simulations/513_p_solitaire_score.ks` | Solitaire : score d'une partie |

### 18_projets_texte_donnees — Mini-projets : texte, données, calculs (50)

| N° | Fichier | Sujet |
|---|---|---|
| 514 | `18_projets_texte_donnees/514_p_compteur_mots.ks` | Compteur de mots |
| 515 | `18_projets_texte_donnees/515_p_rot13.ks` | Chiffrement ROT13 |
| 516 | `18_projets_texte_donnees/516_p_vigenere.ks` | Chiffre de Vigenère |
| 517 | `18_projets_texte_donnees/517_p_morse_encoder.ks` | Code Morse : encoder |
| 518 | `18_projets_texte_donnees/518_p_morse_decoder.ks` | Code Morse : décoder |
| 519 | `18_projets_texte_donnees/519_p_rle_encoder.ks` | Compression RLE : encoder |
| 520 | `18_projets_texte_donnees/520_p_rle_decoder.ks` | Compression RLE : décoder |
| 521 | `18_projets_texte_donnees/521_p_romain_vers_entier.ks` | Chiffres romains vers entier |
| 522 | `18_projets_texte_donnees/522_p_nombre_en_lettres.ks` | Nombre en lettres (français) |
| 523 | `18_projets_texte_donnees/523_p_slugify.ks` | Slugify : texte vers URL |
| 524 | `18_projets_texte_donnees/524_p_titre_casse.ks` | Mettre un titre en majuscules initiales |
| 525 | `18_projets_texte_donnees/525_p_csv.ks` | Lecteur CSV |
| 526 | `18_projets_texte_donnees/526_p_ini.ks` | Lecteur de fichier INI |
| 527 | `18_projets_texte_donnees/527_p_template.ks` | Moteur de modèles |
| 528 | `18_projets_texte_donnees/528_p_markdown_titres.ks` | Table des matières Markdown |
| 529 | `18_projets_texte_donnees/529_p_url.ks` | Analyseur d'URL |
| 530 | `18_projets_texte_donnees/530_p_emails.ks` | Extraire les adresses e-mail d'un texte |
| 531 | `18_projets_texte_donnees/531_p_tokenizer.ks` | Découper une expression en jetons |
| 532 | `18_projets_texte_donnees/532_p_expr_eval.ks` | Évaluateur d'expressions arithmétiques |
| 533 | `18_projets_texte_donnees/533_p_brainfuck.ks` | Interpréteur Brainfuck |
| 534 | `18_projets_texte_donnees/534_p_pile_vm.ks` | Machine à pile |
| 535 | `18_projets_texte_donnees/535_p_base64.ks` | Encodage Base64 |
| 536 | `18_projets_texte_donnees/536_p_hexdump.ks` | Affichage hexadécimal |
| 537 | `18_projets_texte_donnees/537_p_checksum.ks` | Somme de contrôle |
| 538 | `18_projets_texte_donnees/538_p_luhn.ks` | Algorithme de Luhn |
| 539 | `18_projets_texte_donnees/539_p_isbn10.ks` | Validation d'un ISBN-10 |
| 540 | `18_projets_texte_donnees/540_p_iban.ks` | Validation d'un IBAN |
| 541 | `18_projets_texte_donnees/541_p_bissextile.ks` | Années bissextiles |
| 542 | `18_projets_texte_donnees/542_p_jour_semaine.ks` | Jour de la semaine (Zeller) |
| 543 | `18_projets_texte_donnees/543_p_age.ks` | Calcul d'âge |
| 544 | `18_projets_texte_donnees/544_p_duree_hms.ks` | Secondes vers heures:minutes:secondes |
| 545 | `18_projets_texte_donnees/545_p_calendrier.ks` | Afficher le calendrier d'un mois |
| 546 | `18_projets_texte_donnees/546_p_unites.ks` | Convertisseur d'unités |
| 547 | `18_projets_texte_donnees/547_p_devises.ks` | Convertisseur de devises |
| 548 | `18_projets_texte_donnees/548_p_temperatures.ks` | Table de conversion des températures |
| 549 | `18_projets_texte_donnees/549_p_imc.ks` | Indice de masse corporelle |
| 550 | `18_projets_texte_donnees/550_p_pourboire.ks` | Calculateur de pourboire |
| 551 | `18_projets_texte_donnees/551_p_pret.ks` | Mensualité d'un prêt |
| 552 | `18_projets_texte_donnees/552_p_interets_composes.ks` | Intérêts composés |
| 553 | `18_projets_texte_donnees/553_p_facture.ks` | Facture avec TVA |
| 554 | `18_projets_texte_donnees/554_p_histogramme.ks` | Histogramme en texte |
| 555 | `18_projets_texte_donnees/555_p_barres_echelle.ks` | Barres mises à l'échelle |
| 556 | `18_projets_texte_donnees/556_p_retour_ligne.ks` | Retour à la ligne automatique |
| 557 | `18_projets_texte_donnees/557_p_centrer.ks` | Centrer du texte |
| 558 | `18_projets_texte_donnees/558_p_levenshtein.ks` | Distance de Levenshtein |
| 559 | `18_projets_texte_donnees/559_p_lcs.ks` | Plus longue sous-séquence commune |
| 560 | `18_projets_texte_donnees/560_p_plus_long_palindrome.ks` | Plus long palindrome d'un texte |
| 561 | `18_projets_texte_donnees/561_p_top_n_mots.ks` | Les N mots les plus fréquents |
| 562 | `18_projets_texte_donnees/562_p_diff.ks` | Comparer deux versions d'un texte |
| 563 | `18_projets_texte_donnees/563_p_acronyme.ks` | Générer un acronyme |

### 19_projets_poo_systemes — Mini-projets : objets, structures, systèmes (50)

| N° | Fichier | Sujet |
|---|---|---|
| 564 | `19_projets_poo_systemes/564_p_pile_generique.ks` | Pile générique |
| 565 | `19_projets_poo_systemes/565_p_file_fifo.ks` | File d'attente (FIFO) |
| 566 | `19_projets_poo_systemes/566_p_deque.ks` | File à double entrée |
| 567 | `19_projets_poo_systemes/567_p_liste_chainee.ks` | Liste chaînée |
| 568 | `19_projets_poo_systemes/568_p_arbre_bst.ks` | Arbre binaire de recherche |
| 569 | `19_projets_poo_systemes/569_p_tas_min.ks` | Tas binaire (file de priorité) |
| 570 | `19_projets_poo_systemes/570_p_graphe_classe.ks` | Graphe non orienté |
| 571 | `19_projets_poo_systemes/571_p_file_priorite_classe.ks` | File de priorité |
| 572 | `19_projets_poo_systemes/572_p_statistiques_flux.ks` | Statistiques au fil de l'eau |
| 573 | `19_projets_poo_systemes/573_p_bus_evenements.ks` | Bus d'événements |
| 574 | `19_projets_poo_systemes/574_p_observateur_thermostat.ks` | Observateur : alerte de température |
| 575 | `19_projets_poo_systemes/575_p_porte_etats.ks` | Machine à états d'une porte |
| 576 | `19_projets_poo_systemes/576_p_formes_aires.ks` | Formes géométriques |
| 577 | `19_projets_poo_systemes/577_p_paie_employes.ks` | Paie des employés |
| 578 | `19_projets_poo_systemes/578_p_parking.ks` | Parking |
| 579 | `19_projets_poo_systemes/579_p_distributeur_billets.ks` | Distributeur de billets |
| 580 | `19_projets_poo_systemes/580_p_annuaire.ks` | Annuaire téléphonique |
| 581 | `19_projets_poo_systemes/581_p_playlist.ks` | Lecteur de playlist |
| 582 | `19_projets_poo_systemes/582_p_panier_reductions.ks` | Panier avec codes promo |
| 583 | `19_projets_poo_systemes/583_p_fraction.ks` | Fractions exactes |
| 584 | `19_projets_poo_systemes/584_p_complexes.ks` | Nombres complexes |
| 585 | `19_projets_poo_systemes/585_p_vecteur_2d.ks` | Vecteurs 2D |
| 586 | `19_projets_poo_systemes/586_p_monnaie.ks` | Type Monnaie |
| 587 | `19_projets_poo_systemes/587_p_polynome.ks` | Polynômes |
| 588 | `19_projets_poo_systemes/588_p_temperature_statique.ks` | Convertisseur avec méthodes static |
| 589 | `19_projets_poo_systemes/589_p_bitset.ks` | Ensemble de bits |
| 590 | `19_projets_poo_systemes/590_p_tampon_circulaire.ks` | Tampon circulaire |
| 591 | `19_projets_poo_systemes/591_p_editeur_annuler.ks` | Éditeur de texte avec annuler / rétablir |
| 592 | `19_projets_poo_systemes/592_p_commandes_annulables.ks` | Patron Commande |
| 593 | `19_projets_poo_systemes/593_p_fabrique_formes.ks` | Patron Fabrique |
| 594 | `19_projets_poo_systemes/594_p_constructeur_requete.ks` | Patron Builder : requête SQL |
| 595 | `19_projets_poo_systemes/595_p_strategie_tri.ks` | Patron Stratégie |
| 596 | `19_projets_poo_systemes/596_p_decorateur_compteur.ks` | Décorateur : compter les appels |
| 597 | `19_projets_poo_systemes/597_p_tri_topologique.ks` | Tri topologique (algorithme de Kahn) |
| 598 | `19_projets_poo_systemes/598_p_tourniquet.ks` | Ordonnanceur à tourniquet (round robin) |
| 599 | `19_projets_poo_systemes/599_p_allocateur_memoire.ks` | Allocateur mémoire (first fit) |
| 600 | `19_projets_poo_systemes/600_p_taille_dossiers.ks` | Taille des dossiers |
| 601 | `19_projets_poo_systemes/601_p_mini_base_de_donnees.ks` | Mini base de données |
| 602 | `19_projets_poo_systemes/602_p_tableur.ks` | Mini tableur |
| 603 | `19_projets_poo_systemes/603_p_arbre_indente.ks` | Afficher une arborescence |
| 604 | `19_projets_poo_systemes/604_p_validateur_formulaire.ks` | Validation de formulaire |
| 605 | `19_projets_poo_systemes/605_p_inventaire_seuils.ks` | Inventaire avec seuils |
| 606 | `19_projets_poo_systemes/606_p_bibliotheque.ks` | Bibliothèque : emprunts |
| 607 | `19_projets_poo_systemes/607_p_reservation_salle.ks` | Réservation de salle |
| 608 | `19_projets_poo_systemes/608_p_classement_etudiants.ks` | Classement d'étudiants |
| 609 | `19_projets_poo_systemes/609_p_pomodoro.ks` | Planning Pomodoro |
| 610 | `19_projets_poo_systemes/610_p_matrice_classe.ks` | Classe Matrice |
| 611 | `19_projets_poo_systemes/611_p_fusion_intervalles.ks` | Fusionner des intervalles |
| 612 | `19_projets_poo_systemes/612_p_plugins.ks` | Registre de plugins |
| 613 | `19_projets_poo_systemes/613_p_compte_historique.ks` | Compte avec historique d'opérations |

### 20_projets_algo_concurrence — Mini-projets : algorithmes, concurrence, async (50)

| N° | Fichier | Sujet |
|---|---|---|
| 614 | `20_projets_algo_concurrence/614_p_dijkstra.ks` | Plus courts chemins de Dijkstra |
| 615 | `20_projets_algo_concurrence/615_p_union_find.ks` | Union-Find (ensembles disjoints) |
| 616 | `20_projets_algo_concurrence/616_p_kruskal.ks` | Arbre couvrant minimal (Kruskal) |
| 617 | `20_projets_algo_concurrence/617_p_sac_a_dos.ks` | Problème du sac à dos 0/1 |
| 618 | `20_projets_algo_concurrence/618_p_rendu_monnaie.ks` | Rendu de monnaie minimal |
| 619 | `20_projets_algo_concurrence/619_p_plus_longue_croissante.ks` | Plus longue sous-suite croissante |
| 620 | `20_projets_algo_concurrence/620_p_kadane.ks` | Sous-tableau de somme maximale |
| 621 | `20_projets_algo_concurrence/621_p_sous_ensembles.ks` | Tous les sous-ensembles |
| 622 | `20_projets_algo_concurrence/622_p_permutations.ks` | Toutes les permutations |
| 623 | `20_projets_algo_concurrence/623_p_combinaisons.ks` | Toutes les combinaisons de k éléments |
| 624 | `20_projets_algo_concurrence/624_p_sudoku_solveur.ks` | Solveur de Sudoku 4 x 4 |
| 625 | `20_projets_algo_concurrence/625_p_bornes_dichotomie.ks` | Bornes inférieure et supérieure |
| 626 | `20_projets_algo_concurrence/626_p_quickselect.ks` | k-ième plus petit élément |
| 627 | `20_projets_algo_concurrence/627_p_tri_comptage.ks` | Tri par comptage |
| 628 | `20_projets_algo_concurrence/628_p_tri_radix.ks` | Tri radix (par chiffres) |
| 629 | `20_projets_algo_concurrence/629_p_fenetre_glissante.ks` | Maximum sur fenêtre glissante |
| 630 | `20_projets_algo_concurrence/630_p_sommes_prefixes.ks` | Sommes préfixes |
| 631 | `20_projets_algo_concurrence/631_p_deux_pointeurs.ks` | Deux pointeurs sur liste triée |
| 632 | `20_projets_algo_concurrence/632_p_puissance_modulaire.ks` | Exponentiation modulaire |
| 633 | `20_projets_algo_concurrence/633_p_euclide_etendu.ks` | Algorithme d'Euclide étendu |
| 634 | `20_projets_algo_concurrence/634_p_fibonacci_matrice.ks` | Fibonacci par puissance de matrice |
| 635 | `20_projets_algo_concurrence/635_p_catalan.ks` | Nombres de Catalan |
| 636 | `20_projets_algo_concurrence/636_p_collatz_record.ks` | Plus long vol de Collatz |
| 637 | `20_projets_algo_concurrence/637_p_armstrong.ks` | Nombres d'Armstrong |
| 638 | `20_projets_algo_concurrence/638_p_nombres_heureux.ks` | Nombres heureux |
| 639 | `20_projets_algo_concurrence/639_p_triplets_pythagoriciens.ks` | Triplets pythagoriciens |
| 640 | `20_projets_algo_concurrence/640_p_ppcm_liste.ks` | PPCM d'une liste |
| 641 | `20_projets_algo_concurrence/641_p_factorielle_limite.ks` | Factorielle et limite des entiers |
| 642 | `20_projets_algo_concurrence/642_p_code_gray.ks` | Code de Gray |
| 643 | `20_projets_algo_concurrence/643_p_somme_parallele.ks` | Somme parallèle |
| 644 | `20_projets_algo_concurrence/644_p_map_reduce_mots.ks` | MapReduce : compter des mots |
| 645 | `20_projets_algo_concurrence/645_p_pool_travailleurs.ks` | Pool de travailleurs |
| 646 | `20_projets_algo_concurrence/646_p_pipeline_trois_etages.ks` | Pipeline à trois étages |
| 647 | `20_projets_algo_concurrence/647_p_tampon_borne.ks` | Producteur / consommateur à tampon borné |
| 648 | `20_projets_algo_concurrence/648_p_limiteur_concurrence.ks` | Limiter la concurrence avec un sémaphore |
| 649 | `20_projets_algo_concurrence/649_p_philosophes.ks` | Dîner des philosophes |
| 650 | `20_projets_algo_concurrence/650_p_phases_barriere.ks` | Phases synchronisées par une barrière |
| 651 | `20_projets_algo_concurrence/651_p_tri_fusion_parallele.ks` | Tri fusion parallèle |
| 652 | `20_projets_algo_concurrence/652_p_matrice_parallele.ks` | Produit de matrices : une tâche par ligne |
| 653 | `20_projets_algo_concurrence/653_p_telechargements_async.ks` | Téléchargements asynchrones |
| 654 | `20_projets_algo_concurrence/654_p_async_reessai.ks` | Réessayer une opération asynchrone |
| 655 | `20_projets_algo_concurrence/655_p_delai_depasse.ks` | Délai dépassé |
| 656 | `20_projets_algo_concurrence/656_p_publication_abonnement.ks` | Publication / abonnement |
| 657 | `20_projets_algo_concurrence/657_p_fan_in.ks` | Fan-in : plusieurs producteurs, un consommateur |
| 658 | `20_projets_algo_concurrence/658_p_depart_course.ks` | Départ de course avec un event |
| 659 | `20_projets_algo_concurrence/659_p_virements_mutex.ks` | Virements protégés par un mutex |
| 660 | `20_projets_algo_concurrence/660_p_cache_rwlock.ks` | Cache protégé par RwLock |
| 661 | `20_projets_algo_concurrence/661_p_annulation_propre.ks` | Annulation propre de tâches |
| 662 | `20_projets_algo_concurrence/662_p_wait_group_resultats.ks` | WaitGroup : collecter des résultats |
| 663 | `20_projets_algo_concurrence/663_p_premiers_paralleles.ks` | Compter les nombres premiers en parallèle |

