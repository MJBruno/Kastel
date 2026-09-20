use std::collections::{HashMap, HashSet};

use super::VirtualMachine;

use crate::{
    error::runtime_error::RuntimeError,
    frontend::ast::{CONSTRUCTOR_NAME, FIELD_INITIALIZER_PREFIX},
    runtime::{gc_handle::Gc, object::Object, value::Value},
};

impl VirtualMachine {
    // ========================================================
    // CLASS
    // ========================================================

    pub(crate) fn op_class(
        &mut self,
        base_count: usize,
        method_count: usize,
        private_count: usize,
    ) -> Result<(), RuntimeError> {
        let method_values = method_count
            .checked_mul(2)
            .ok_or(RuntimeError::InvalidFunction)?;

        // Disposition sur la pile :
        //   [bases...] nom [nom_méthode closure]* [nom_membre_privé]*
        let total = base_count
            .checked_add(1)
            .and_then(|value| value.checked_add(method_values))
            .and_then(|value| value.checked_add(private_count))
            .ok_or(RuntimeError::InvalidFunction)?;

        if self.stack.len() < total {
            return Err(RuntimeError::StackUnderflow);
        }

        let start = self.stack.len() - total;

        let mut superclass = None;
        let mut interfaces = Vec::with_capacity(base_count);

        for index in 0..base_count {
            let value = self
                .stack
                .get(start + index)
                .cloned()
                .ok_or(RuntimeError::StackUnderflow)?;

            let handle = match value {
                Value::Object(handle) => handle,
                _ => return Err(RuntimeError::TypeError),
            };

            match &*handle.borrow() {
                Object::Class { .. } => {
                    if superclass.is_some() {
                        return Err(RuntimeError::TypeError);
                    }

                    superclass = Some(handle.clone());
                }

                Object::Interface { .. } => {
                    interfaces.push(handle.clone());
                }

                _ => return Err(RuntimeError::TypeError),
            }
        }

        let class_name = self
            .stack
            .get(start + base_count)
            .and_then(Value::as_string_value)
            .ok_or(RuntimeError::TypeError)?;

        let methods_start = start + base_count + 1;
        let mut methods = HashMap::<String, Vec<Value>>::with_capacity(method_count);

        for index in 0..method_count {
            let base = methods_start + index * 2;

            let method_name = self
                .stack
                .get(base)
                .and_then(Value::as_string_value)
                .ok_or(RuntimeError::TypeError)?;

            let method = self
                .stack
                .get(base + 1)
                .cloned()
                .ok_or(RuntimeError::StackUnderflow)?;

            let arity = match &method {
                Value::Object(handle) => {
                    let object = handle.borrow();
                    match &*object {
                        Object::Closure(closure) => closure
                            .function
                            .arity
                            .checked_sub(1)
                            .ok_or(RuntimeError::TypeError)?,
                        _ => return Err(RuntimeError::NotCallable),
                    }
                }
                _ => return Err(RuntimeError::NotCallable),
            };

            let overloads = methods.entry(method_name.clone()).or_default();

            if overloads.iter().any(|existing| match existing {
                Value::Object(handle) => {
                    let object = handle.borrow();
                    matches!(
                        &*object,
                        Object::Closure(closure)
                            if closure.function.arity.checked_sub(1) == Some(arity)
                    )
                }
                _ => false,
            }) {
                return Err(RuntimeError::DuplicateMethod {
                    name: method_name,
                    arity,
                });
            }

            overloads.push(method);
        }

        let private_start = methods_start + method_values;
        let mut private_members = HashSet::<String>::with_capacity(private_count);

        for index in 0..private_count {
            let member = self
                .stack
                .get(private_start + index)
                .and_then(Value::as_string_value)
                .ok_or(RuntimeError::TypeError)?;

            private_members.insert(member);
        }

        self.stack.truncate(start);

        let class_value = Value::new_class(
            class_name,
            superclass,
            interfaces,
            methods,
            private_members,
        );

        let class_handle = match &class_value {
            Value::Object(handle) => handle.clone(),
            _ => return Err(RuntimeError::InvalidFunction),
        };

        {
            let mut class_object = class_handle.borrow_mut();

            let Object::Class { methods, .. } = &mut *class_object else {
                return Err(RuntimeError::InvalidFunction);
            };

            for overloads in methods.values() {
                for method in overloads {
                    if let Value::Object(handle) = method {
                        let mut object = handle.borrow_mut();

                        if let Object::Closure(closure) = &mut *object {
                            closure.owner_class = Some(class_handle.clone());
                        }
                    }
                }
            }
        }

        Self::validate_interfaces(&class_handle)?;

        self.push(class_value);

        Ok(())
    }

    // ========================================================
    // INTERFACE VALIDATION
    // ========================================================

    fn collect_interface_methods(
        interface: Gc<Object>,
        methods: &mut HashSet<(String, usize)>,
    ) -> Result<(), RuntimeError> {
        let (bases, own_methods) = {
            let object = interface.borrow();

            match &*object {
                Object::Interface { bases, methods, .. } => (bases.clone(), methods.clone()),

                _ => return Err(RuntimeError::TypeError),
            }
        };

        for base in bases {
            Self::collect_interface_methods(base, methods)?;
        }

        // Les signatures sont identifiées par `(nom, arité)` : deux
        // interfaces (ou une interface et sa parente) peuvent exiger
        // `area()` et `area(unit)` ensemble sans conflit.
        methods.extend(own_methods);

        Ok(())
    }

    fn validate_interfaces(class: &Gc<Object>) -> Result<(), RuntimeError> {
        let interfaces = {
            let object = class.borrow();

            match &*object {
                Object::Class { interfaces, .. } => interfaces.clone(),
                _ => return Err(RuntimeError::TypeError),
            }
        };

        for interface in interfaces {
            let interface_name = {
                let object = interface.borrow();

                match &*object {
                    Object::Interface { name, .. } => name.clone(),
                    _ => return Err(RuntimeError::TypeError),
                }
            };

            let mut collected = HashSet::<(String, usize)>::new();

            Self::collect_interface_methods(interface.clone(), &mut collected)?;

            // Ordre déterministe : l'erreur rapportée ne dépend pas de
            // l'ordre d'itération du HashSet.
            let mut requirements: Vec<(String, usize)> = collected.into_iter().collect();
            requirements.sort();

            for (name, required_arity) in requirements {
                // La surcharge exacte (même nom, même arité) existe : OK.
                if Self::find_class_method_from(class.clone(), &name, required_arity).is_some() {
                    continue;
                }

                // Sinon : soit la méthode existe avec d'autres arités
                // (erreur d'arité), soit elle n'existe pas du tout.
                let declared = Self::class_method_arities(class.clone(), &name);

                return Err(match declared.first() {
                    Some(found) => RuntimeError::InterfaceMethodArityMismatch {
                        interface: interface_name.clone(),
                        method: name,
                        expected: required_arity,
                        found: *found,
                    },

                    None => RuntimeError::InterfaceMethodMissing {
                        interface: interface_name.clone(),
                        method: name,
                    },
                });
            }
        }

        Ok(())
    }

    // ========================================================
    // INTERFACE
    // ========================================================

    pub(crate) fn op_interface(
        &mut self,
        base_count: usize,
        method_count: usize,
    ) -> Result<(), RuntimeError> {
        let method_values = method_count
            .checked_mul(2)
            .ok_or(RuntimeError::InvalidFunction)?;

        let total = base_count
            .checked_add(1)
            .and_then(|value| value.checked_add(method_values))
            .ok_or(RuntimeError::InvalidFunction)?;

        if self.stack.len() < total {
            return Err(RuntimeError::StackUnderflow);
        }

        let start = self.stack.len() - total;

        let mut bases = Vec::with_capacity(base_count);

        for index in 0..base_count {
            let value = self
                .stack
                .get(start + index)
                .cloned()
                .ok_or(RuntimeError::StackUnderflow)?;

            let handle = match value {
                Value::Object(handle) => handle,
                _ => return Err(RuntimeError::TypeError),
            };

            if !matches!(&*handle.borrow(), Object::Interface { .. }) {
                return Err(RuntimeError::TypeError);
            }

            bases.push(handle);
        }

        let name_index = start + base_count;

        let name = self
            .stack
            .get(name_index)
            .and_then(Value::as_string_value)
            .ok_or(RuntimeError::TypeError)?;

        let methods_start = name_index + 1;
        let mut methods = HashSet::<(String, usize)>::with_capacity(method_count);

        for index in 0..method_count {
            let base = methods_start + index * 2;

            let method_name = self
                .stack
                .get(base)
                .and_then(Value::as_string_value)
                .ok_or(RuntimeError::TypeError)?;

            let arity_value = self
                .stack
                .get(base + 1)
                .ok_or(RuntimeError::StackUnderflow)?;

            let arity = match arity_value {
                Value::Integer(value) if *value >= 0 => {
                    usize::try_from(*value).map_err(|_| RuntimeError::TypeError)?
                }

                _ => return Err(RuntimeError::TypeError),
            };

            // Même nom avec des arités différentes = surcharge autorisée ;
            // même nom ET même arité = vraie redéclaration.
            if !methods.insert((method_name.clone(), arity)) {
                return Err(RuntimeError::DuplicateMethod {
                    name: method_name,
                    arity,
                });
            }
        }

        self.stack.truncate(start);
        self.push(Value::new_interface(name, bases, methods));

        Ok(())
    }

    // ========================================================
    // NEW INSTANCE
    // ========================================================

    pub(crate) fn op_new_instance(&mut self, arg_count: usize) -> Result<(), RuntimeError> {
        // Les racines ajoutées par `op_new_instance_inner` (classe et
        // arguments, retirés de la pile) sont libérées quoi qu'il arrive.
        let mark = self.temp_roots.len();
        let result = self.op_new_instance_inner(arg_count);

        self.temp_roots.truncate(mark);

        result
    }

    fn op_new_instance_inner(&mut self, arg_count: usize) -> Result<(), RuntimeError> {
        let required = arg_count
            .checked_add(1)
            .ok_or(RuntimeError::InvalidFunction)?;

        if self.stack.len() < required {
            return Err(RuntimeError::StackUnderflow);
        }

        let class_index = self.stack.len() - required;

        let class_value = self
            .stack
            .get(class_index)
            .cloned()
            .ok_or(RuntimeError::StackUnderflow)?;

        let class_handle = match &class_value {
            Value::Object(handle) => match &*handle.borrow() {
                Object::Class { .. } => handle.clone(),
                _ => return Err(RuntimeError::NotCallable),
            },

            _ => return Err(RuntimeError::NotCallable),
        };

        let instance = Value::new_instance(class_handle.clone());

        let args = self
            .stack
            .get(class_index + 1..)
            .ok_or(RuntimeError::StackUnderflow)?
            .to_vec();

        self.stack.truncate(class_index);

        self.push(instance.clone());

        // La classe et les arguments viennent de quitter la pile : on les
        // enracine pendant les initialiseurs de champs et le constructeur,
        // qui peuvent déclencher un GC.
        self.temp_roots.push(class_value.clone());
        self.temp_roots.extend(args.iter().cloned());

        // 1. Choix du constructeur, par arité (surcharge). Les constructeurs
        //    de la classe de base sont hérités.
        let constructor =
            Self::find_class_method_from(class_handle.clone(), CONSTRUCTOR_NAME, arg_count);

        if constructor.is_none() {
            // Aucun `initialize` ne prend `arg_count` arguments. S'il en
            // existe d'autres arités, c'est une erreur d'arité. Sinon la
            // classe n'a pas de constructeur : le constructeur PAR DÉFAUT
            // implicite (sans paramètre) s'applique et n'accepte aucun
            // argument.
            let declared = Self::class_method_arities(class_handle.clone(), CONSTRUCTOR_NAME);

            let expected = declared.first().copied().unwrap_or(0);

            if !declared.is_empty() || arg_count != 0 {
                return Err(RuntimeError::WrongArgumentCount {
                    expected,
                    found: arg_count,
                });
            }
        }

        // 2. Valeurs initiales des champs, de la classe de base vers la
        //    classe dérivée, AVANT le constructeur.
        for class in Self::class_chain(class_handle.clone()) {
            if let Some(initializer) = Self::find_field_initializer(&class) {
                self.invoke_sync(initializer, std::slice::from_ref(&instance))?;
            }
        }

        // 3. Constructeur explicite, s'il y en a un.
        if let Some(constructor) = constructor {
            let mut constructor_args = Vec::with_capacity(arg_count + 1);

            constructor_args.push(instance.clone());
            constructor_args.extend(args);

            self.invoke_sync(constructor, &constructor_args)?;
        }

        self.pop()?;
        self.push(instance);

        Ok(())
    }

    /// Classes de la hiérarchie de `class`, de la plus ancienne (racine) à
    /// la plus dérivée (`class` elle-même en dernier).
    fn class_chain(class: Gc<Object>) -> Vec<Gc<Object>> {
        let mut chain = Vec::new();
        let mut current = Some(class);

        while let Some(handle) = current {
            let superclass = match &*handle.borrow() {
                Object::Class { superclass, .. } => superclass.clone(),
                _ => None,
            };

            chain.push(handle);
            current = superclass;
        }

        chain.reverse();
        chain
    }

    /// Méthode cachée `__fields_<Classe>` qui porte les valeurs initiales
    /// des champs déclarés par CETTE classe (pas ceux de ses bases).
    fn find_field_initializer(class: &Gc<Object>) -> Option<Value> {
        let object = class.borrow();

        match &*object {
            Object::Class { name, methods, .. } => methods
                .get(&format!("{FIELD_INITIALIZER_PREFIX}{name}"))
                .and_then(|overloads| overloads.first())
                .cloned(),

            _ => None,
        }
    }

    // ========================================================
    // INSTANCE OF
    // ========================================================

    pub(crate) fn is_value_instance_of(
        value: &Value,
        target: &Value,
    ) -> Result<bool, RuntimeError> {
        let Value::Object(value_handle) = value else {
            return Ok(false);
        };

        let Value::Object(target_handle) = target else {
            return Err(RuntimeError::TypeError);
        };

        let value_class = {
            let object = value_handle.borrow();

            match &*object {
                Object::Instance { class, .. } => class.clone(),
                _ => return Ok(false),
            }
        };

        let Some(value_class) = value_class else {
            return Ok(false);
        };

        let target_kind = {
            let object = target_handle.borrow();

            match &*object {
                Object::Class { .. } => 0,
                Object::Interface { .. } => 1,
                _ => return Err(RuntimeError::TypeError),
            }
        };

        match target_kind {
            0 => {
                let mut current = Some(value_class);

                while let Some(class_handle) = current {
                    if Gc::ptr_eq(&class_handle, target_handle) {
                        return Ok(true);
                    }

                    current = {
                        let object = class_handle.borrow();

                        match &*object {
                            Object::Class { superclass, .. } => superclass.clone(),

                            _ => None,
                        }
                    };
                }

                Ok(false)
            }

            1 => Self::class_implements_interface(value_class, target_handle.clone()),

            _ => Err(RuntimeError::TypeError),
        }
    }

    fn class_implements_interface(
        class: Gc<Object>,
        target: Gc<Object>,
    ) -> Result<bool, RuntimeError> {
        let mut current_class = Some(class);

        while let Some(class_handle) = current_class {
            let (interfaces, superclass) = {
                let object = class_handle.borrow();

                match &*object {
                    Object::Class {
                        interfaces,
                        superclass,
                        ..
                    } => (interfaces.clone(), superclass.clone()),

                    _ => return Ok(false),
                }
            };

            for interface in interfaces {
                if Self::interface_extends_or_is(interface, &target)? {
                    return Ok(true);
                }
            }

            current_class = superclass;
        }

        Ok(false)
    }

    fn interface_extends_or_is(
        interface: Gc<Object>,
        target: &Gc<Object>,
    ) -> Result<bool, RuntimeError> {
        if Gc::ptr_eq(&interface, target) {
            return Ok(true);
        }

        let bases = {
            let object = interface.borrow();

            match &*object {
                Object::Interface { bases, .. } => bases.clone(),
                _ => return Ok(false),
            }
        };

        for base in bases {
            if Self::interface_extends_or_is(base, target)? {
                return Ok(true);
            }
        }

        Ok(false)
    }
}
