use super::VirtualMachine;
use super::bytecode::frame_closure;

use crate::{
    error::runtime_error::RuntimeError,
    runtime::{gc_handle::Gc, object::Object, value::Value},
    stdlib::{array, dict},
};

impl VirtualMachine {
    // ============================================================
    //                     METHOD RESOLUTION
    // ============================================================

    pub(crate) fn find_class_method_from(
        class: Gc<Object>,
        name: &str,
        arg_count: usize,
    ) -> Option<Value> {
        Self::find_method_in_hierarchy(Some(class), name, arg_count)
    }

    pub(crate) fn find_base_method(
        class: Gc<Object>,
        name: &str,
        arg_count: usize,
    ) -> Option<Value> {
        let parent = {
            let object = class.borrow();

            match &*object {
                Object::Class { superclass, .. } => superclass.clone(),
                _ => None,
            }
        };

        Self::find_method_in_hierarchy(parent, name, arg_count)
    }

    fn find_method_in_hierarchy(
        mut current: Option<Gc<Object>>,
        name: &str,
        arg_count: usize,
    ) -> Option<Value> {
        while let Some(handle) = current {
            let object = handle.borrow();

            match &*object {
                Object::Class {
                    methods,
                    superclass,
                    ..
                } => {
                    if let Some(overloads) = methods.get(name) {
                        for method in overloads {
                            if let Value::Object(handle) = method {
                                let object = handle.borrow();
                                if let Object::Closure(closure) = &*object
                                    && closure.function.arity.checked_sub(1) == Some(arg_count)
                                {
                                    return Some(method.clone());
                                }
                            }
                        }
                    }

                    current = superclass.clone();
                }

                _ => return None,
            }
        }

        None
    }

    /// Arités (hors `this`) sous lesquelles `name` est déclarée dans la
    /// hiérarchie de `class` : triées, sans doublon. Sert uniquement à
    /// produire une erreur d'arité précise quand aucune surcharge ne
    /// correspond au nombre d'arguments passés.
    pub(crate) fn class_method_arities(class: Gc<Object>, name: &str) -> Vec<usize> {
        Self::method_arities_in_hierarchy(Some(class), name)
    }

    /// Comme `class_method_arities`, mais à partir de la classe parente
    /// (appel `base.methode(...)`).
    pub(crate) fn base_method_arities(class: Gc<Object>, name: &str) -> Vec<usize> {
        let parent = {
            let object = class.borrow();

            match &*object {
                Object::Class { superclass, .. } => superclass.clone(),
                _ => None,
            }
        };

        Self::method_arities_in_hierarchy(parent, name)
    }

    fn method_arities_in_hierarchy(mut current: Option<Gc<Object>>, name: &str) -> Vec<usize> {
        let mut arities = Vec::new();

        while let Some(handle) = current {
            let object = handle.borrow();

            match &*object {
                Object::Class {
                    methods,
                    superclass,
                    ..
                } => {
                    if let Some(overloads) = methods.get(name) {
                        for method in overloads {
                            if let Value::Object(method_handle) = method {
                                let method_object = method_handle.borrow();

                                if let Object::Closure(closure) = &*method_object
                                    && let Some(arity) = closure.function.arity.checked_sub(1)
                                {
                                    arities.push(arity);
                                }
                            }
                        }
                    }

                    current = superclass.clone();
                }

                _ => break,
            }
        }

        arities.sort_unstable();
        arities.dedup();
        arities
    }

    // ============================================================
    //                     INVOKE METHOD
    // ============================================================

    pub(crate) fn op_invoke_method(
        &mut self,
        method_constant: usize,
        arg_count: usize,
    ) -> Result<(), RuntimeError> {
        let method_constant = method_constant
            .try_into()
            .map_err(|_| RuntimeError::InvalidFunction)?;

        let method_value = self.read_constant(method_constant)?;

        let method_name = method_value
            .as_string_value()
            .ok_or(RuntimeError::TypeError)?;

        let required = arg_count
            .checked_add(1)
            .ok_or(RuntimeError::InvalidFunction)?;

        if self.stack.len() < required {
            return Err(RuntimeError::StackUnderflow);
        }

        let receiver_index = self.stack.len() - required;
        let receiver = self.stack[receiver_index].clone();
        let args = self.stack[receiver_index..].to_vec();

        self.stack.truncate(receiver_index);

        if method_name == "to_iterator" {
            if arg_count != 0 {
                return Err(RuntimeError::WrongArgumentCount {
                    expected: 0,
                    found: arg_count,
                });
            }

            self.push(receiver.to_iterator()?);
            return Ok(());
        }

        let result = match &receiver {
            Value::Range { .. } => {
                let iterator = receiver.to_iterator()?;

                let mut iterator_args = args.clone();
                iterator_args[0] = iterator;

                self.invoke_iterator_method(&method_name, &iterator_args)?
            }

            Value::Object(handle) => {
                let object_kind = {
                    let object = handle.borrow();

                    match &*object {
                        Object::Instance { .. } => 0,
                        Object::Iterator(_) => 1,
                        Object::String(_) => 2,
                        Object::Array(_) => 3,
                        Object::Dict(_) => 4,
                        Object::Tuple(_) => 6,
                        Object::Module(_) => 7,
                        _ => 5,
                    }
                };

                match object_kind {
                    0 => {
                        let class_handle = {
                            let object = handle.borrow();

                            match &*object {
                                Object::Instance { class, .. } => {
                                    class.clone().ok_or(RuntimeError::TypeError)?
                                }

                                _ => return Err(RuntimeError::TypeError),
                            }
                        };

                        let method = match Self::find_class_method_from(
                            class_handle.clone(),
                            &method_name,
                            arg_count,
                        ) {
                            Some(method) => method,

                            None => {
                                // La méthode existe-t-elle sous une autre
                                // arité ? Alors c'est une erreur d'arité,
                                // pas un champ introuvable.
                                let declared =
                                    Self::class_method_arities(class_handle, &method_name);

                                if let Some(expected) = declared.first() {
                                    return Err(RuntimeError::WrongArgumentCount {
                                        expected: *expected,
                                        found: arg_count,
                                    });
                                }

                                return Err(RuntimeError::ObjectFieldNotFound {
                                    name: method_name,
                                    suggestion: None,
                                });
                            }
                        };

                        let method_handle = match method {
                            Value::Object(method_handle)
                                if matches!(&*method_handle.borrow(), Object::Closure(_)) =>
                            {
                                method_handle
                            }

                            _ => return Err(RuntimeError::NotCallable),
                        };

                        /*
                         * obj.method(a, b)
                         *
                         * devient :
                         *
                         * method(obj, a, b)
                         *
                         * La closure de méthode attend `this` comme premier paramètre.
                         */

                        self.push(Value::Object(method_handle));
                        self.push(receiver);

                        for argument in args.iter().skip(1) {
                            self.push(argument.clone());
                        }

                        self.execute_call(arg_count + 1)?;

                        return Ok(());
                    }
                    1 => self.invoke_iterator_method(&method_name, &args)?,

                    2 => match crate::stdlib::string::dispatch_method(&method_name, &args)? {
                        Some(result) => result,

                        None => {
                            return Err(RuntimeError::ObjectFieldNotFound {
                                name: method_name,
                                suggestion: None,
                            });
                        }
                    },

                    3 => {
                        if let Some(result) = array::dispatch_method(&method_name, &args)? {
                            result
                        } else {
                            self.invoke_array_functional(&method_name, &args)?
                        }
                    }

                    4 => match dict::dispatch_method(&method_name, &args)? {
                        Some(result) => result,

                        None => {
                            return Err(RuntimeError::ObjectFieldNotFound {
                                name: method_name,
                                suggestion: None,
                            });
                        }
                    },

                    6 => match crate::stdlib::tuple::dispatch_method(&method_name, &args)? {
                        Some(result) => result,

                        None => {
                            return Err(RuntimeError::ObjectFieldNotFound {
                                name: method_name,
                                suggestion: None,
                            });
                        }
                    },

                    7 => {
                        /*
                         * module.function(a, b)
                         *
                         * Un export de module n'a pas de `this` : contrairement à
                         * une méthode d'instance, on pousse directement la
                         * fonction exportée puis les arguments (sans le
                         * receveur, qui n'est que le module lui-même).
                         */
                        let exported = {
                            let object = handle.borrow();

                            match &*object {
                                Object::Module(module) => module.get_export(&method_name).cloned(),
                                _ => None,
                            }
                        };

                        let Some(exported) = exported else {
                            return Err(RuntimeError::ObjectFieldNotFound {
                                name: method_name,
                                suggestion: None,
                            });
                        };

                        self.push(exported);

                        for argument in args.iter().skip(1) {
                            self.push(argument.clone());
                        }

                        self.execute_call(arg_count)?;

                        return Ok(());
                    }

                    _ => {
                        return Err(RuntimeError::ObjectFieldNotFound {
                            name: method_name,
                            suggestion: None,
                        });
                    }
                }
            }

            _ => return Err(RuntimeError::NotObject),
        };

        self.push(result);
        Ok(())
    }

    // ============================================================
    //                     BASE METHOD
    // ============================================================

    pub(crate) fn op_invoke_base_method(
        &mut self,
        method_constant: usize,
        arg_count: usize,
    ) -> Result<(), RuntimeError> {
        let method_constant =
            u8::try_from(method_constant).map_err(|_| RuntimeError::InvalidFunction)?;

        let method_value = self.read_constant(method_constant)?;

        let method_name = method_value
            .as_string_value()
            .ok_or(RuntimeError::TypeError)?;

        /*
         * compile_base_method_call pousse `this` (via Expression::This)
         * puis les `arg_count` arguments réels avant d'émettre
         * InvokeBaseMethod : la pile contient donc, du bas vers le
         * haut, [this, arg0, ..., arg(n-1)] — exactement comme pour un
         * appel de méthode normal (op_invoke_method) où le receveur est
         * suivi de ses arguments.
         *
         * L'ancienne implémentation relisait `this` via
         * frame.slot_start (en ignorant la valeur poussée par le
         * compilateur) et ne retirait donc jamais ce `this` de la
         * pile : chaque appel `base.methode(...)` laissait une valeur
         * fantôme, décalant de un tous les emplacements de variables
         * locales compilés après cet appel dans la même fonction.
         */
        let required = arg_count
            .checked_add(1)
            .ok_or(RuntimeError::InvalidFunction)?;

        if self.stack.len() < required {
            return Err(RuntimeError::StackUnderflow);
        }

        let this_index = self.stack.len() - required;
        let this_value = self.stack[this_index].clone();
        let args = self.stack[this_index..].to_vec();

        self.stack.truncate(this_index);

        let frame = self
            .frames
            .last()
            .cloned()
            .ok_or(RuntimeError::InvalidFunction)?;

        let owner_class = {
            let closure = frame_closure(&frame.closure);

            closure.owner_class.clone().ok_or(RuntimeError::TypeError)?
        };

        let method = match Self::find_base_method(owner_class.clone(), &method_name, arg_count) {
            Some(method) => method,

            None => {
                let declared = Self::base_method_arities(owner_class, &method_name);

                if let Some(expected) = declared.first() {
                    return Err(RuntimeError::WrongArgumentCount {
                        expected: *expected,
                        found: arg_count,
                    });
                }

                return Err(RuntimeError::ObjectFieldNotFound {
                    name: method_name,
                    suggestion: None,
                });
            }
        };

        self.push(method);
        self.push(this_value);

        for argument in args.into_iter().skip(1) {
            self.push(argument);
        }

        self.execute_call(arg_count + 1)?;

        Ok(())
    }

    // ============================================================
    //                  ARRAY FUNCTIONAL METHODS
    // ============================================================

    fn invoke_array_functional(
        &mut self,
        method: &str,
        args: &[Value],
    ) -> Result<Value, RuntimeError> {
        match method {
            "map" => {
                if args.len() != 2 {
                    return Err(RuntimeError::WrongArgumentCount {
                        expected: 2,
                        found: args.len(),
                    });
                }

                let elements = Self::array_snapshot(&args[0])?;
                let callback = args[1].clone();

                let mut result = Vec::with_capacity(elements.len());

                for element in elements {
                    result.push(self.invoke_sync(callback.clone(), &[element])?);
                }

                Ok(Value::new_array(result))
            }

            "filter" => {
                if args.len() != 2 {
                    return Err(RuntimeError::WrongArgumentCount {
                        expected: 2,
                        found: args.len(),
                    });
                }

                let elements = Self::array_snapshot(&args[0])?;
                let callback = args[1].clone();

                let mut result = Vec::new();

                for element in elements {
                    let keep =
                        self.invoke_sync(callback.clone(), std::slice::from_ref(&element))?;

                    if keep.is_truthy() {
                        result.push(element);
                    }
                }

                Ok(Value::new_array(result))
            }

            "reduce" => {
                if args.len() != 3 {
                    return Err(RuntimeError::WrongArgumentCount {
                        expected: 3,
                        found: args.len(),
                    });
                }

                let elements = Self::array_snapshot(&args[0])?;
                let callback = args[1].clone();
                let mut accumulator = args[2].clone();

                for element in elements {
                    accumulator = self.invoke_sync(callback.clone(), &[accumulator, element])?;
                }

                Ok(accumulator)
            }

            "any" => {
                if args.len() != 2 {
                    return Err(RuntimeError::WrongArgumentCount {
                        expected: 2,
                        found: args.len(),
                    });
                }

                let elements = Self::array_snapshot(&args[0])?;
                let callback = args[1].clone();

                for element in elements {
                    let value = self.invoke_sync(callback.clone(), &[element])?;

                    if value.is_truthy() {
                        return Ok(Value::Boolean(true));
                    }
                }

                Ok(Value::Boolean(false))
            }

            "all" => {
                if args.len() != 2 {
                    return Err(RuntimeError::WrongArgumentCount {
                        expected: 2,
                        found: args.len(),
                    });
                }

                let elements = Self::array_snapshot(&args[0])?;
                let callback = args[1].clone();

                for element in elements {
                    let value = self.invoke_sync(callback.clone(), &[element])?;

                    if !value.is_truthy() {
                        return Ok(Value::Boolean(false));
                    }
                }

                Ok(Value::Boolean(true))
            }

            _ => Err(RuntimeError::ObjectFieldNotFound {
                name: method.to_string(),
                suggestion: None,
            }),
        }
    }

    fn array_snapshot(value: &Value) -> Result<Vec<Value>, RuntimeError> {
        match value {
            Value::Object(handle) => {
                let object = handle.borrow();

                match &*object {
                    Object::Array(array) => Ok(array.clone()),
                    _ => Err(RuntimeError::TypeError),
                }
            }

            _ => Err(RuntimeError::TypeError),
        }
    }
}
