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
}

#[derive(Clone, Debug)]
/// Table des variables locales actuellement visibles par le compilateur.
pub struct LocalTable {
    locals: Vec<Local>,
}

impl LocalTable {
    /// Crée une table locale vide.
    pub fn new() -> Self {
        Self { locals: Vec::new() }
    }

    /// Retourne le nombre de variables locales actuellement enregistrées.
    pub fn len(&self) -> usize {
        self.locals.len()
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

        if self.locals.len() >= u8::MAX as usize {
            return Err(CompileError::TooManyLocals);
        }

        let slot = self.locals.len() as u8;

        self.locals.push(Local {
            name: name.to_string(),
            depth: None,
            slot,
            mutable,
        });

        Ok(slot)
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