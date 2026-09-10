use std::collections::HashMap;

use super::VirtualMachine;

use crate::{
    error::runtime_error::RuntimeError,
    runtime::{
        gc_handle::Gc,
        object::Object,
        value::Value,
    },
};

impl VirtualMachine {
    // ========================================================
    // CLASS
    // ========================================================

    pub(crate) fn op_class(
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
        let mut methods = HashMap::with_capacity(method_count);

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

            if !matches!(
                &method,
                Value::Object(handle)
                    if matches!(&*handle.borrow(), Object::Closure(_))
            ) {
                return Err(RuntimeError::NotCallable);
            }

            if methods.insert(method_name, method).is_some() {
                return Err(RuntimeError::TypeError);
            }
        }

        self.stack.truncate(start);

        let class_value =
            Value::new_class(class_name, superclass, interfaces, methods);

        let class_handle = match &class_value {
            Value::Object(handle) => handle.clone(),
            _ => return Err(RuntimeError::InvalidFunction),
        };

        {
            let mut class_object = class_handle.borrow_mut();

            let Object::Class { methods, .. } = &mut *class_object else {
                return Err(RuntimeError::InvalidFunction);
            };

            for method in methods.values() {
                if let Value::Object(handle) = method {
                    let mut object = handle.borrow_mut();

                    if let Object::Closure(closure) = &mut *object {
                        closure.owner_class = Some(class_handle.clone());
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
        methods: &mut HashMap<String, usize>,
    ) -> Result<(), RuntimeError> {
        let (bases, own_methods) = {
            let object = interface.borrow();

            match &*object {
                Object::Interface {
                    bases,
                    methods,
                    ..
                } => (bases.clone(), methods.clone()),

                _ => return Err(RuntimeError::TypeError),
            }
        };

        for base in bases {
            Self::collect_interface_methods(base, methods)?;
        }

        for (name, arity) in own_methods {
            if let Some(existing) = methods.get(&name)
                && *existing != arity
            {
                return Err(RuntimeError::TypeError);
            }

            methods.insert(name, arity);
        }

        Ok(())
    }

    fn validate_interfaces(
        class: &Gc<Object>,
    ) -> Result<(), RuntimeError> {
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

            let mut requirements = HashMap::<String, usize>::new();

            Self::collect_interface_methods(
                interface.clone(),
                &mut requirements,
            )?;

            for (name, required_arity) in requirements {
                let method =
                    Self::find_class_method_from(class.clone(), &name);

                let Some(method) = method else {
                    return Err(RuntimeError::InterfaceMethodMissing {
                        interface: interface_name.clone(),
                        method: name,
                    });
                };

                let actual_arity = match method {
                    Value::Object(handle) => {
                        let object = handle.borrow();

                        match &*object {
                            Object::Closure(closure) => closure
                                .function
                                .arity
                                .checked_sub(1)
                                .ok_or(RuntimeError::TypeError)?,

                            _ => return Err(RuntimeError::TypeError),
                        }
                    }

                    _ => return Err(RuntimeError::TypeError),
                };

                if actual_arity != required_arity {
                    return Err(RuntimeError::InterfaceMethodArityMismatch {
                        interface: interface_name.clone(),
                        method: name,
                        expected: required_arity,
                        found: actual_arity,
                    });
                }
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
        let mut methods = HashMap::with_capacity(method_count);

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
                    usize::try_from(*value)
                        .map_err(|_| RuntimeError::TypeError)?
                }

                _ => return Err(RuntimeError::TypeError),
            };

            if methods.insert(method_name, arity).is_some() {
                return Err(RuntimeError::TypeError);
            }
        }

        self.stack.truncate(start);
        self.push(Value::new_interface(name, bases, methods));

        Ok(())
    }

    // ========================================================
    // NEW INSTANCE
    // ========================================================

    pub(crate) fn op_new_instance(
        &mut self,
        arg_count: usize,
    ) -> Result<(), RuntimeError> {
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

        let init = Self::find_class_method_from(class_handle, "init");

        match init {
            Some(init) => {
                let mut init_args = Vec::with_capacity(arg_count + 1);

                init_args.push(instance.clone());
                init_args.extend(args);

                self.invoke_sync(init, &init_args)?;

                self.pop()?;
            }

            None => {
                if !args.is_empty() {
                    return Err(RuntimeError::WrongArgumentCount {
                        expected: 0,
                        found: args.len(),
                    });
                }

                self.pop()?;
            }
        }

        self.push(instance);

        Ok(())
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

                while let Some(class) = current {
                    if Gc::ptr_eq(&class, target_handle) {
                        return Ok(true);
                    }

                    current = {
                        let object = class.borrow();

                        match &*object {
                            Object::Class { superclass, .. } => {
                                superclass.clone()
                            }

                            _ => None,
                        }
                    };
                }

                Ok(false)
            }

            1 => {
                Self::class_implements_interface(
                    value_class,
                    target_handle.clone(),
                )
            }

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