use super::VirtualMachine;
use super::bytecode::frame_closure;

use crate::{
    error::runtime_error::RuntimeError,
    frontend::ast::CONSTRUCTOR_NAME,
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
        let object = class.borrow();
        let Object::Class { methods, .. } = &*object else {
            return None;
        };

        let overloads = methods.get(name)?;

        overloads.iter().find_map(|method| {
            let Value::Object(handle) = method else {
                return None;
            };

            let object = handle.borrow();
            let Object::Closure(closure) = &*object else {
                return None;
            };

            (closure.function.arity.checked_sub(1) == Some(arg_count))
                .then(|| method.clone())
        })
    }

    pub(crate) fn class_method_arities(class: Gc<Object>, name: &str) -> Vec<usize> {
        let object = class.borrow();
        let Object::Class { methods, .. } = &*object else {
            return Vec::new();
        };

        let mut arities = methods
            .get(name)
            .into_iter()
            .flat_map(|overloads| overloads.iter())
            .filter_map(|method| match method {
                Value::Object(handle) => {
                    let object = handle.borrow();
                    match &*object {
                        Object::Closure(closure) => closure.function.arity.checked_sub(1),
                        _ => None,
                    }
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        arities.sort_unstable();
        arities.dedup();
        arities
    }

    // ============================================================
    //                            RANGE
    // ============================================================

    /// Méthodes d'un `range(...)` : `size()`, `is_empty()`, `start()`,
    /// `stop()`, `step()`, `to_string()`. `None` si `name` n'en fait pas
    /// partie (la VM bascule alors sur les méthodes d'itérateur).
    fn range_method(
        name: &str,
        start: f64,
        stop: f64,
        step: f64,
        receiver: &Value,
        arg_count: usize,
    ) -> Result<Option<Value>, RuntimeError> {
        if !matches!(
            name,
            "size" | "is_empty" | "start" | "stop" | "step" | "to_string"
        ) {
            return Ok(None);
        }

        if arg_count != 0 {
            return Err(RuntimeError::WrongArgumentCount {
                expected: 0,
                found: arg_count,
            });
        }

        // `range()` n'accepte que des entiers et un pas non nul.
        let (start, stop, step) = (start as i128, stop as i128, step as i128);

        let size = if step > 0 && start < stop {
            (stop - start + step - 1) / step
        } else if step < 0 && start > stop {
            (start - stop + (-step) - 1) / (-step)
        } else {
            0
        };

        let value = match name {
            "size" => Value::Integer(size as i64),
            "is_empty" => Value::Boolean(size == 0),
            "start" => Value::Integer(start as i64),
            "stop" => Value::Integer(stop as i64),
            "step" => Value::Integer(step as i64),
            _ => Value::new_string(receiver.to_string()),
        };

        Ok(Some(value))
    }

    // ============================================================
    //                  VISIBILITÉ DES MEMBRES
    // ============================================================

    /// Classe du code en cours d'exécution (`None` hors d'une méthode).
    /// Les closures créées dans une méthode héritent de son propriétaire
    /// de classe afin de conserver les règles de visibilité.
    pub(crate) fn caller_owner_class(&self) -> Option<Gc<Object>> {
        let frame = self.frames.last()?;
        let owner = frame_closure(&frame.closure).owner_class.clone();

        owner
    }

    /// Il n'existe plus de sous-classes en Kastel. Le mot-clé `protected`,
    /// conservé pour compatibilité, est donc limité à la classe déclarante.
    fn is_same_class(
        caller: Option<Gc<Object>>,
        owner: &Gc<Object>,
    ) -> bool {
        caller.as_ref().is_some_and(|current| Gc::ptr_eq(current, owner))
    }

    /// `new C(...)` : le constructeur est toujours celui de `C`. Il n'existe
    /// aucun constructeur hérité.
    pub(crate) fn ensure_constructor_access(&self, class: &Gc<Object>) -> Result<(), RuntimeError> {
        let (declares, is_private, is_protected, class_name) = {
            let object = class.borrow();

            match &*object {
                Object::Class {
                    name,
                    methods,
                    private_members,
                    protected_members,
                    ..
                } => (
                    methods.contains_key(CONSTRUCTOR_NAME),
                    private_members.contains(CONSTRUCTOR_NAME),
                    protected_members.contains(CONSTRUCTOR_NAME),
                    name.clone(),
                ),

                _ => return Ok(()),
            }
        };

        if !declares {
            return Ok(());
        }

        let caller = self.caller_owner_class();
        let allowed = if is_private || is_protected {
            Self::is_same_class(caller, class)
        } else {
            true
        };

        if allowed {
            return Ok(());
        }

        if is_protected {
            Err(RuntimeError::ProtectedMemberAccess {
                class_name,
                member: CONSTRUCTOR_NAME.to_string(),
            })
        } else {
            Err(RuntimeError::PrivateMemberAccess {
                class_name,
                member: CONSTRUCTOR_NAME.to_string(),
            })
        }
    }

    /// Refuse l'accès à `receiver.name` si `name` est un membre `private`
    /// déclaré par une classe de `receiver` et que le code appelant n'est
    /// PAS une méthode de cette classe.
    ///
    /// Sans effet sur les valeurs qui ne sont pas des instances : le typage
    /// reste dynamique, seule la visibilité déclarée est contrôlée.
    pub(crate) fn ensure_member_access(
        &self,
        receiver: &Value,
        name: &str,
    ) -> Result<(), RuntimeError> {
        let Value::Object(handle) = receiver else {
            return Ok(());
        };

        // `NomClasse.membre` : accès STATIQUE, directement sur la classe
        // (jamais hérité, donc pas de remontée de hiérarchie ici — voir
        // `ensure_static_member_access`).
        if matches!(&*handle.borrow(), Object::Class { .. }) {
            return self.ensure_static_member_access(handle, name);
        }

        let class = {
            let object = handle.borrow();

            match &*object {
                Object::Instance {
                    class: Some(class), ..
                } => class.clone(),

                _ => return Ok(()),
            }
        };

        let (is_private, is_protected, class_name) = {
            let object = class.borrow();

            match &*object {
                Object::Class {
                    name: class_name,
                    private_members,
                    protected_members,
                    ..
                } => (
                    private_members.contains(name),
                    protected_members.contains(name),
                    class_name.clone(),
                ),

                _ => return Ok(()),
            }
        };

        if !is_private && !is_protected {
            return Ok(());
        }

        let caller = self.caller_owner_class();
        let allowed = Self::is_same_class(caller, &class);

        if allowed {
            return Ok(());
        }

        if is_protected {
            return Err(RuntimeError::ProtectedMemberAccess {
                class_name,
                member: name.to_string(),
            });
        }

        Err(RuntimeError::PrivateMemberAccess {
            class_name,
            member: name.to_string(),
        })
    }

    /// Comme `ensure_member_access`, pour un accès STATIQUE
    /// (`NomClasse.membre`) : les membres statiques ne sont pas hérités, donc
    /// seule la classe désignée elle-même est consultée (pas sa hiérarchie).
    fn ensure_static_member_access(
        &self,
        class: &Gc<Object>,
        name: &str,
    ) -> Result<(), RuntimeError> {
        let (is_private, is_protected, class_name) = {
            let object = class.borrow();

            match &*object {
                Object::Class {
                    name: class_name,
                    private_members,
                    protected_members,
                    ..
                } => (
                    private_members.contains(name),
                    protected_members.contains(name),
                    class_name.clone(),
                ),

                _ => return Ok(()),
            }
        };

        if !is_private && !is_protected {
            return Ok(());
        }

        let caller = self.caller_owner_class();
        let allowed = Self::is_same_class(caller, class);

        if allowed {
            return Ok(());
        }

        if is_protected {
            Err(RuntimeError::ProtectedMemberAccess {
                class_name,
                member: name.to_string(),
            })
        } else {
            Err(RuntimeError::PrivateMemberAccess {
                class_name,
                member: name.to_string(),
            })
        }
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

        // `iter()` : convention unique pour obtenir un itérateur (la syntaxe
        // principale reste `for x in collection`).
        if method_name == "iter"
            && !matches!(&receiver, Value::Object(handle) if matches!(&*handle.borrow(), Object::Instance { .. }))
        {
            if arg_count != 0 {
                return Err(RuntimeError::WrongArgumentCount {
                    expected: 0,
                    found: arg_count,
                });
            }

            self.push(receiver.to_iterator()?);
            return Ok(());
        }

        // Ancien nom, supprimé au profit de `iter()`.
        if method_name == "to_iterator" {
            return Err(crate::stdlib::renamed_method_error("to_iterator", "iter()"));
        }

        let result = match &receiver {
            Value::Range { start, stop, step } => {
                // API standard : size(), is_empty(), start(), stop(), step(),
                // to_string(). Le reste (map, filter, take...) passe par
                // l'itérateur.
                match Self::range_method(&method_name, *start, *stop, *step, &receiver, arg_count)?
                {
                    Some(result) => result,

                    None => {
                        let iterator = receiver.to_iterator()?;

                        let mut iterator_args = args.clone();
                        iterator_args[0] = iterator;

                        self.invoke_iterator_method(&method_name, &iterator_args)?
                    }
                }
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
                        Object::Set(_) => 8,
                        Object::Record(_) => 9,
                        Object::EnumVariant { .. } => 10,
                        Object::Class { .. } => 11,
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

                        // Méthode privée : appelable uniquement depuis la
                        // classe qui la déclare.
                        self.ensure_member_access(&receiver, &method_name)?;

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

                    8 => match crate::stdlib::set::dispatch_method(&method_name, &args)? {
                        Some(result) => result,

                        None => {
                            return Err(RuntimeError::ObjectFieldNotFound {
                                name: method_name,
                                suggestion: None,
                            });
                        }
                    },

                    9 => {
                        // Champ contenant une fonction : `p.greet(...)` l'appelle
                        // (sans `this`, comme un export de module). Le CHAMP
                        // l'emporte sur les méthodes d'introspection.
                        let field = {
                            let object = handle.borrow();

                            match &*object {
                                Object::Record(fields) => fields
                                    .iter()
                                    .find(|(name, _)| name.as_str() == method_name.as_str())
                                    .map(|(_, value)| value.clone()),

                                _ => None,
                            }
                        };

                        if let Some(callable) = field {
                            self.push(callable);

                            for argument in args.iter().skip(1) {
                                self.push(argument.clone());
                            }

                            self.execute_call(arg_count)?;

                            return Ok(());
                        }

                        match crate::stdlib::record::dispatch_method(&method_name, &args)? {
                            Some(result) => result,

                            None => {
                                return Err(RuntimeError::ObjectFieldNotFound {
                                    name: method_name,
                                    suggestion: None,
                                });
                            }
                        }
                    }

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

                    10 => {
                        let (method, expected) = {
                            let object = handle.borrow();

                            match &*object {
                                Object::EnumVariant { methods, .. } => {
                                    let method = methods.get(&method_name).and_then(|overloads| {
                                        overloads.iter().find(|value| {
                                            matches!(
                                                value,
                                                Value::Object(method_handle)
                                                    if matches!(
                                                        &*method_handle.borrow(),
                                                        Object::Closure(closure)
                                                            if closure.function.arity == arg_count + 1
                                                    )
                                            )
                                        })
                                    }).cloned();

                                    let expected = methods.get(&method_name).and_then(|overloads| {
                                        overloads.first().and_then(|value| match value {
                                            Value::Object(method_handle) => {
                                                let method_object = method_handle.borrow();
                                                match &*method_object {
                                                    Object::Closure(closure) => {
                                                        closure.function.arity.checked_sub(1)
                                                    }
                                                    _ => None,
                                                }
                                            }
                                            _ => None,
                                        })
                                    });

                                    (method, expected)
                                }

                                _ => return Err(RuntimeError::TypeError),
                            }
                        };

                        let Some(Value::Object(method_handle)) = method else {
                            if let Some(expected) = expected {
                                return Err(RuntimeError::WrongArgumentCount {
                                    expected,
                                    found: arg_count,
                                });
                            }

                            return Err(RuntimeError::ObjectFieldNotFound {
                                name: method_name,
                                suggestion: None,
                            });
                        };

                        if !matches!(&*method_handle.borrow(), Object::Closure(_)) {
                            return Err(RuntimeError::NotCallable);
                        }

                        self.push(Value::Object(method_handle));
                        self.push(receiver);

                        for argument in args.iter().skip(1) {
                            self.push(argument.clone());
                        }

                        self.execute_call(arg_count + 1)?;

                        return Ok(());
                    }

                    11 => {
                        // `NomClasse.methode(a, b)` : méthode STATIQUE,
                        // jamais héritée. Contrairement à une méthode
                        // d'instance, il n'y a pas de `this` à pousser avant
                        // les arguments.
                        self.ensure_member_access(&receiver, &method_name)?;

                        let (method, declared_arity) = {
                            let object = handle.borrow();

                            match &*object {
                                Object::Class { static_methods, .. } => {
                                    let overloads = static_methods.get(&method_name);

                                    let method = overloads.and_then(|overloads| {
                                        overloads.iter().find(|value| {
                                            matches!(
                                                value,
                                                Value::Object(method_handle)
                                                    if matches!(
                                                        &*method_handle.borrow(),
                                                        Object::Closure(closure)
                                                            if closure.function.arity == arg_count
                                                    )
                                            )
                                        })
                                    }).cloned();

                                    let declared_arity = overloads.and_then(|overloads| {
                                        overloads.first().and_then(|value| match value {
                                            Value::Object(method_handle) => {
                                                match &*method_handle.borrow() {
                                                    Object::Closure(closure) => {
                                                        Some(closure.function.arity)
                                                    }
                                                    _ => None,
                                                }
                                            }
                                            _ => None,
                                        })
                                    });

                                    (method, declared_arity)
                                }

                                _ => return Err(RuntimeError::TypeError),
                            }
                        };

                        let method_handle = match method {
                            Some(Value::Object(method_handle))
                                if matches!(&*method_handle.borrow(), Object::Closure(_)) =>
                            {
                                method_handle
                            }

                            _ => {
                                // Pas de méthode statique de cette arité : soit
                                // elle existe sous une autre arité (erreur
                                // d'arité), soit un CHAMP statique porte ce
                                // nom et se trouve être appelable (comme un
                                // champ de Record), soit le nom est inconnu.
                                if let Some(expected) = declared_arity {
                                    return Err(RuntimeError::WrongArgumentCount {
                                        expected,
                                        found: arg_count,
                                    });
                                }

                                let field = {
                                    let object = handle.borrow();

                                    match &*object {
                                        Object::Class { statics, .. } => {
                                            statics.get(&method_name).cloned()
                                        }
                                        _ => None,
                                    }
                                };

                                let Some(callable) = field else {
                                    return Err(RuntimeError::ObjectFieldNotFound {
                                        name: method_name,
                                        suggestion: None,
                                    });
                                };

                                self.push(callable);

                                for argument in args.iter().skip(1) {
                                    self.push(argument.clone());
                                }

                                self.execute_call(arg_count)?;

                                return Ok(());
                            }
                        };

                        self.push(Value::Object(method_handle));

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
    //                  ARRAY FUNCTIONAL METHODS
    // ============================================================

    /// `map`, `filter`, `reduce`, `any`, `all` : rappellent du code Kastel.
    ///
    /// Le receveur et les arguments (`args`) ont été retirés de la pile : ils
    /// ne sont plus tenus que par des variables Rust. On les enracine donc
    /// pendant tout l'appel, sinon un GC déclenché dans le rappel les
    /// viderait (`break_cycle`).
    fn invoke_array_functional(
        &mut self,
        method: &str,
        args: &[Value],
    ) -> Result<Value, RuntimeError> {
        self.with_temp_roots(args, |vm| vm.invoke_array_functional_inner(method, args))
    }

    fn invoke_array_functional_inner(
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

                // Le rappel peut modifier le tableau d'origine : on garde
                // les éléments du cliché vivants.
                self.temp_roots.extend(elements.iter().cloned());
                let callback = args[1].clone();

                let mut result = Vec::with_capacity(elements.len());

                for element in elements {
                    let value = self.invoke_sync(callback.clone(), &[element])?;

                    // Résultat intermédiaire : seulement tenu par `result`
                    // jusqu'à la construction du tableau final.
                    self.protect(&value);
                    result.push(value);
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

                // Le rappel peut modifier le tableau d'origine : on garde
                // les éléments du cliché vivants.
                self.temp_roots.extend(elements.iter().cloned());
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

                // Le rappel peut modifier le tableau d'origine : on garde
                // les éléments du cliché vivants.
                self.temp_roots.extend(elements.iter().cloned());
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

                // Le rappel peut modifier le tableau d'origine : on garde
                // les éléments du cliché vivants.
                self.temp_roots.extend(elements.iter().cloned());
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

                // Le rappel peut modifier le tableau d'origine : on garde
                // les éléments du cliché vivants.
                self.temp_roots.extend(elements.iter().cloned());
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
