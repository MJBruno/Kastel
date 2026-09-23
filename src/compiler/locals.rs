use crate::error::compile_error::CompileError;

#[derive(Clone, Debug)]
/// # Local
/// Représente une variable locale connue du compilateur.
/// `name` contient le nom source de la variable, `depth` indique la profondeur
/// de portée à laquelle elle est initialisée et `slot` correspond à sa position
/// dans la pile de la VM.
pub struct Local {
    /// Nom de la variable tel qu'il apparaît dans le programme source.
    pub name: String,

    /// Profondeur de portée où la variable a été initialisée.
    /// `None` signifie que la variable est encore en cours d'initialisation.
    pub depth: Option<usize>,

    /// Emplacement de la variable dans la pile des variables locales.
    pub slot: u8,

    /// Pour distinguer une déclaration `const` ou `let`.
    pub mutable: bool,

    /// Non vide si cette locale contient une FONCTION déclarée par `func`
    /// (et non une variable) : arités déjà déclarées sous ce nom, pour la
    /// surcharge de fonctions locales.
    pub function_arities: Vec<usize>,
}

#[derive(Clone, Debug)]
/// Table des variables locales actuellement visibles par le compilateur.
pub struct LocalTable {
    locals: Vec<Local>,

    /// Nombre maximal de slots locaux simultanément utilisés depuis
    /// la création de cette table.
    ///
    /// Contrairement à `locals.len()`, cette valeur ne diminue jamais
    /// lorsque `pop_scope()` retire des variables. Elle représente donc
    /// correctement la taille maximale nécessaire au frame runtime.
    max_slots: usize,
}

impl LocalTable {
    /// Crée une table locale vide.
    pub fn new() -> Self {
        Self {
            locals: Vec::new(),
            max_slots: 0,
        }
    }

    /// Retourne le nombre de variables locales actuellement enregistrées.
    pub fn len(&self) -> usize {
        self.locals.len()
    }

    /// Retourne le nombre maximal de slots locaux utilisés pendant
    /// toute la compilation de cette fonction.
    pub fn max_slots(&self) -> usize {
        self.max_slots
    }

    /// Noms de toutes les variables locales actuellement visibles — utilisé
    /// uniquement pour construire des suggestions "vouliez-vous dire ?" en
    /// cas de variable non définie, jamais pour de la résolution réelle.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.locals.iter().map(|local| local.name.as_str())
    }

    /// Déclare une nouvelle variable locale et lui attribue un slot.
    /// La fonction vérifie également les redéclarations dans la portée courante
    /// et refuse de dépasser la capacité représentable par un `u8`.
    pub fn declare_local(
        &mut self,
        name: &str,
        depth: usize,
        mutable: bool,
    ) -> Result<u8, CompileError> {
        for local in self.locals.iter().rev() {
            if let Some(local_depth) = local.depth {
                if local_depth < depth {
                    break;
                }

                if local.name == name {
                    return Err(CompileError::VariableAlreadyDeclared(name.to_string()));
                }
            }
        }

        if self.locals.len() > u8::MAX as usize {
            return Err(CompileError::TooManyLocals);
        }

        let slot = self.locals.len() as u8;

        self.locals.push(Local {
            name: name.to_string(),
            depth: None,
            slot,
            mutable,
            function_arities: Vec::new(),
        });

        self.max_slots = self.max_slots.max(self.locals.len());

        Ok(slot)
    }

    /// Fonction locale `name` déjà déclarée DANS la portée `depth` : son slot
    /// et les arités déjà prises (base de la surcharge de fonctions locales).
    pub fn local_function_in_scope(&self, name: &str, depth: usize) -> Option<(u8, Vec<usize>)> {
        for local in self.locals.iter().rev() {
            let Some(local_depth) = local.depth else {
                continue;
            };

            if local_depth < depth {
                break;
            }

            if local.name == name {
                return if local.function_arities.is_empty() {
                    None
                } else {
                    Some((local.slot, local.function_arities.clone()))
                };
            }
        }

        None
    }

    /// Note que la locale `slot` contient une fonction de `arity` paramètres.
    pub fn add_function_arity(&mut self, slot: u8, arity: usize) {
        if let Some(local) = self.locals.get_mut(slot as usize) {
            local.function_arities.push(arity);
        }
    }

    pub fn is_mutable(&self, name: &str) -> Result<Option<bool>, CompileError> {
        for local in self.locals.iter().rev() {
            if local.name != name {
                continue;
            }

            if local.depth.is_none() {
                return Err(CompileError::VariableUseInInitializer(name.to_string()));
            }

            return Ok(Some(local.mutable));
        }

        Ok(None)
    }

    /// Marque la dernière variable déclarée comme complètement initialisée.
    pub fn mark_initialized(&mut self, depth: usize) {
        if let Some(local) = self.locals.last_mut() {
            local.depth = Some(depth);
        }
    }

    /// Recherche une variable locale depuis la portée la plus proche.
    /// Retourne son slot si elle existe, ou `None` lorsqu'elle n'est pas locale.
    pub fn resolve_local(&self, name: &str) -> Result<Option<u8>, CompileError> {
        for local in self.locals.iter().rev() {
            if local.name != name {
                continue;
            }

            if local.depth.is_none() {
                return Err(CompileError::VariableUseInInitializer(name.to_string()));
            }

            return Ok(Some(local.slot));
        }

        Ok(None)
    }

    /// Supprime les variables appartenant aux portées qui viennent de se terminer.
    /// Retourne le nombre de variables retirées afin que le compilateur puisse
    /// générer autant d'instructions `Pop` dans le bytecode.
    pub fn pop_scope(&mut self, depth: usize) -> usize {
        let mut count = 0;

        while let Some(local) = self.locals.last() {
            let local_depth = match local.depth {
                Some(depth) => depth,
                None => break,
            };

            if local_depth <= depth {
                break;
            }

            self.locals.pop();
            count += 1;
        }

        count
    }

    /// Compte les variables qui doivent être retirées avant un saut hors de portée.
    pub fn cleanup_count(&self, depth: usize) -> usize {
        self.locals
            .iter()
            .filter(|local| matches!(local.depth, Some(d) if d > depth))
            .count()
    }
}
