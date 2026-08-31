use std::rc::Rc;

use crate::error::compile_error::CompileError;
use crate::runtime::upvalue::Upvalue;

use super::compiler::Compiler;
use super::context::CompilerContextRef;

impl Compiler {
    // ============================================================
    // UPVALUES
    // ============================================================

    pub(crate) fn add_upvalue(&mut self, index: usize, is_local: bool) -> Result<usize, CompileError> {
        let mut context = self.context.borrow_mut();

        for (i, upvalue) in context.upvalues.iter().enumerate() {
            if upvalue.index as usize == index && upvalue.is_local == is_local {
                return Ok(i);
            }
        }

        if context.upvalues.len() >= u8::MAX as usize {
            return Err(CompileError::TooManyUpvalues);
        }

        let index_result = context.upvalues.len();

        context.upvalues.push(Upvalue {
            index: index as u8,
            is_local,
        });

        Ok(index_result)
    }

    /*
     * Résolution récursive correcte :
     *
     * enfant
     *   ↓
     * parent local       => is_local = true
     *
     * ou
     *
     * parent upvalue     => is_local = false
     *
     * Exemple :
     *
     * make()
     *   x
     *
     *   get()
     *     get2()
     *       x
     *
     * get capture x
     * get2 capture l'upvalue de get
     */
    pub(crate) fn resolve_upvalue(&mut self, name: &str) -> Result<Option<usize>, CompileError> {
        let enclosing = {
            let context = self.context.borrow();

            match &context.enclosing {
                Some(parent) => Rc::clone(parent),
                None => return Ok(None),
            }
        };

        let result = Self::resolve_upvalue_recursive(&enclosing, name)?;

        match result {
            Some((index, is_local)) => {
                let upvalue = self.add_upvalue(index, is_local)?;
                Ok(Some(upvalue))
            }

            None => Ok(None),
        }
    }

    fn resolve_upvalue_recursive(
        context: &CompilerContextRef,
        name: &str,
    ) -> Result<Option<(usize, bool)>, CompileError> {
        // --------------------------------------------------------
        // 1. Variable locale du parent immédiat
        // --------------------------------------------------------

        {
            let context_ref = context.borrow();

            if let Some(slot) = context_ref.locals.resolve_local(name)? {
                return Ok(Some((slot as usize, true)));
            }
        }

        // --------------------------------------------------------
        // 2. Chercher plus loin
        // --------------------------------------------------------

        let enclosing = {
            let context_ref = context.borrow();

            match &context_ref.enclosing {
                Some(parent) => Rc::clone(parent),
                None => return Ok(None),
            }
        };

        let result = Self::resolve_upvalue_recursive(&enclosing, name)?;

        let Some((index, is_local)) = result else {
            return Ok(None);
        };

        // --------------------------------------------------------
        // 3. Le parent doit lui-même capturer la variable
        // --------------------------------------------------------

        let parent_upvalue = {
            let mut context_ref = context.borrow_mut();

            for (i, upvalue) in context_ref.upvalues.iter().enumerate() {
                if upvalue.index as usize == index && upvalue.is_local == is_local {
                    return Ok(Some((i, false)));
                }
            }

            if context_ref.upvalues.len() >= u8::MAX as usize {
                return Err(CompileError::TooManyUpvalues);
            }

            let new_index = context_ref.upvalues.len();

            context_ref.upvalues.push(Upvalue {
                index: index as u8,
                is_local,
            });

            new_index
        };

        Ok(Some((parent_upvalue, false)))
    }
}