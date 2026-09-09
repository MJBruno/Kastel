use super::VirtualMachine;

use crate::{
    error::runtime_error::RuntimeError,
    runtime::{iterator::IteratorKind, object::Object, value::Value},
};

impl VirtualMachine {
    // ============================================================
    //                         GET ITERATOR
    // ============================================================

    pub(crate) fn op_get_iterator(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop()?;

        self.push(value.to_iterator()?);

        Ok(())
    }

    // ============================================================
    //                     ITERATOR HAS NEXT
    // ============================================================

    pub(crate) fn op_iterator_has_next(&mut self) -> Result<(), RuntimeError> {
        let iterator = self.pop()?;

        let result = self.iterator_has_next_value(&iterator)?;

        self.push(Value::Boolean(result));

        Ok(())
    }

    // ============================================================
    //                        ITERATOR NEXT
    // ============================================================

    pub(crate) fn op_iterator_next(&mut self) -> Result<(), RuntimeError> {
        let iterator = self.pop()?;

        let value = self.iterator_next_value(&iterator)?;

        self.push(value);

        Ok(())
    }

    // ============================================================
    //                    ITERATOR HAS NEXT
    // ============================================================

    fn iterator_has_next_value(&mut self, iterator: &Value) -> Result<bool, RuntimeError> {
        match self.iterator_peek_value(iterator) {
            Ok(_) => Ok(true),

            Err(RuntimeError::IteratorExhausted) => Ok(false),

            Err(error) => Err(error),
        }
    }

    // ============================================================
    //                       ITERATOR NEXT
    // ============================================================

    pub fn iterator_next_value(&mut self, iterator: &Value) -> Result<Value, RuntimeError> {
        // --------------------------------------------------------
        // Un cached value.
        // --------------------------------------------------------
        if let Some(value) = self.take_iterator_cache(iterator)? {
            return Ok(value);
        }

        self.iterator_next_uncached(iterator)
    }

    // ============================================================
    //                       ITERATOR PEEK
    // ============================================================

    fn iterator_peek_value(&mut self, iterator: &Value) -> Result<Value, RuntimeError> {
        if let Some(value) = self.get_iterator_cache(iterator)? {
            return Ok(value);
        }

        let value = self.iterator_next_uncached(iterator)?;

        self.set_iterator_cache(iterator, value.clone())?;

        Ok(value)
    }

    // ============================================================
    //                   NEXT WITHOUT CACHE
    // ============================================================

    fn iterator_next_uncached(&mut self, iterator: &Value) -> Result<Value, RuntimeError> {
        let Value::Object(handle) = iterator else {
            return Err(RuntimeError::TypeError);
        };

        let kind = {
            let object = handle.borrow();

            let Object::Iterator(state) = &*object else {
                return Err(RuntimeError::TypeError);
            };

            state.kind.clone()
        };

        match kind {
            // ====================================================
            // RANGE
            // ====================================================
            IteratorKind::Range {
                current,
                stop,
                step,
            } => {
                let has_next = if step >= 0.0 {
                    current < stop
                } else {
                    current > stop
                };

                if !has_next {
                    return Err(RuntimeError::IteratorExhausted);
                }

                self.set_range_current(iterator, current + step)?;

                Ok(Value::Integer(current as i64))
            }
            // ====================================================
            // DICT
            // ====================================================
            IteratorKind::Dict { dict, index } => {
                let value = {
                    let object = dict.borrow();

                    let Object::Dict(entries) = &*object else {
                        return Err(RuntimeError::TypeError);
                    };

                    entries
                        .get(index)
                        .map(|(key, _)| key.clone())
                        .ok_or(RuntimeError::IteratorExhausted)?
                };

                self.set_dict_index(iterator, index + 1)?;

                Ok(value)
            }
            // ====================================================
            // ARRAY
            // ====================================================
            IteratorKind::Array { array, index } => {
                let value = {
                    let object = array.borrow();

                    let Object::Array(elements) = &*object else {
                        return Err(RuntimeError::TypeError);
                    };

                    elements
                        .get(index)
                        .cloned()
                        .ok_or(RuntimeError::IteratorExhausted)?
                };

                self.set_array_index(iterator, index + 1)?;

                Ok(value)
            }

            // ====================================================
            // MAP
            // ====================================================
            IteratorKind::Map { source, callback } => {
                let value = self.iterator_next_value(&source)?;

                self.invoke_sync(callback, &[value])
            }

            // ====================================================
            // FILTER
            // ====================================================
            IteratorKind::Filter { source, callback } => loop {
                let value = match self.iterator_next_value(&source) {
                    Ok(value) => value,

                    Err(RuntimeError::IteratorExhausted) => {
                        return Err(RuntimeError::IteratorExhausted);
                    }

                    Err(error) => return Err(error),
                };

                let keep = self.invoke_sync(callback.clone(), std::slice::from_ref(&value))?;

                if keep.is_truthy() {
                    return Ok(value);
                }
            },

            // ====================================================
            // TAKE
            // ====================================================
            IteratorKind::Take { source, remaining } => {
                if remaining == 0 {
                    return Err(RuntimeError::IteratorExhausted);
                }

                let value = self.iterator_next_value(&source)?;

                self.set_take_remaining(iterator, remaining - 1)?;

                Ok(value)
            }

            // ====================================================
            // SKIP
            // ====================================================
            IteratorKind::Skip { source, remaining } => {
                let mut remaining = remaining;

                while remaining > 0 {
                    self.iterator_next_value(&source)?;

                    remaining -= 1;
                }

                self.set_skip_remaining(iterator, remaining)?;

                self.iterator_next_value(&source)
            }
            // ====================================================
            // STRING
            // ====================================================
            IteratorKind::String { string, index } => {
                let value = {
                    let object = string.borrow();

                    let Object::String(value) = &*object else {
                        return Err(RuntimeError::TypeError);
                    };

                    value
                        .chars()
                        .nth(index)
                        .map(|character| Value::new_string(character.to_string()))
                        .ok_or(RuntimeError::IteratorExhausted)?
                };

                self.set_string_index(iterator, index + 1)?;

                Ok(value)
            }
        }
    }

    fn set_string_index(&mut self, iterator: &Value, index: usize) -> Result<(), RuntimeError> {
        let Value::Object(handle) = iterator else {
            return Err(RuntimeError::TypeError);
        };

        let mut object = handle.borrow_mut();

        let Object::Iterator(state) = &mut *object else {
            return Err(RuntimeError::TypeError);
        };

        if let IteratorKind::String {
            index: state_index, ..
        } = &mut state.kind
        {
            *state_index = index;

            Ok(())
        } else {
            Err(RuntimeError::TypeError)
        }
    }
    fn set_dict_index(&mut self, iterator: &Value, index: usize) -> Result<(), RuntimeError> {
        let Value::Object(handle) = iterator else {
            return Err(RuntimeError::TypeError);
        };

        let mut object = handle.borrow_mut();

        let Object::Iterator(state) = &mut *object else {
            return Err(RuntimeError::TypeError);
        };

        if let IteratorKind::Dict {
            index: state_index, ..
        } = &mut state.kind
        {
            *state_index = index;

            Ok(())
        } else {
            Err(RuntimeError::TypeError)
        }
    }
    // ============================================================
    //                         CACHE
    // ============================================================

    fn get_iterator_cache(&self, iterator: &Value) -> Result<Option<Value>, RuntimeError> {
        let Value::Object(handle) = iterator else {
            return Err(RuntimeError::TypeError);
        };

        let object = handle.borrow();

        let Object::Iterator(state) = &*object else {
            return Err(RuntimeError::TypeError);
        };

        Ok(state.cached.clone())
    }

    fn take_iterator_cache(&mut self, iterator: &Value) -> Result<Option<Value>, RuntimeError> {
        let Value::Object(handle) = iterator else {
            return Err(RuntimeError::TypeError);
        };

        let mut object = handle.borrow_mut();

        let Object::Iterator(state) = &mut *object else {
            return Err(RuntimeError::TypeError);
        };

        Ok(state.cached.take())
    }

    fn set_iterator_cache(&mut self, iterator: &Value, value: Value) -> Result<(), RuntimeError> {
        let Value::Object(handle) = iterator else {
            return Err(RuntimeError::TypeError);
        };

        let mut object = handle.borrow_mut();

        let Object::Iterator(state) = &mut *object else {
            return Err(RuntimeError::TypeError);
        };

        state.cached = Some(value);

        Ok(())
    }

    // ============================================================
    //                     STATE MUTATION
    // ============================================================

    fn set_range_current(&mut self, iterator: &Value, current: f64) -> Result<(), RuntimeError> {
        let Value::Object(handle) = iterator else {
            return Err(RuntimeError::TypeError);
        };

        let mut object = handle.borrow_mut();

        let Object::Iterator(state) = &mut *object else {
            return Err(RuntimeError::TypeError);
        };

        if let IteratorKind::Range {
            current: state_current,
            ..
        } = &mut state.kind
        {
            *state_current = current;
            Ok(())
        } else {
            Err(RuntimeError::TypeError)
        }
    }

    fn set_array_index(&mut self, iterator: &Value, index: usize) -> Result<(), RuntimeError> {
        let Value::Object(handle) = iterator else {
            return Err(RuntimeError::TypeError);
        };

        let mut object = handle.borrow_mut();

        let Object::Iterator(state) = &mut *object else {
            return Err(RuntimeError::TypeError);
        };

        if let IteratorKind::Array {
            index: state_index, ..
        } = &mut state.kind
        {
            *state_index = index;
            Ok(())
        } else {
            Err(RuntimeError::TypeError)
        }
    }

    fn set_take_remaining(
        &mut self,
        iterator: &Value,
        remaining: usize,
    ) -> Result<(), RuntimeError> {
        let Value::Object(handle) = iterator else {
            return Err(RuntimeError::TypeError);
        };

        let mut object = handle.borrow_mut();

        let Object::Iterator(state) = &mut *object else {
            return Err(RuntimeError::TypeError);
        };

        if let IteratorKind::Take {
            remaining: state_remaining,
            ..
        } = &mut state.kind
        {
            *state_remaining = remaining;
            Ok(())
        } else {
            Err(RuntimeError::TypeError)
        }
    }

    fn set_skip_remaining(
        &mut self,
        iterator: &Value,
        remaining: usize,
    ) -> Result<(), RuntimeError> {
        let Value::Object(handle) = iterator else {
            return Err(RuntimeError::TypeError);
        };

        let mut object = handle.borrow_mut();

        let Object::Iterator(state) = &mut *object else {
            return Err(RuntimeError::TypeError);
        };

        if let IteratorKind::Skip {
            remaining: state_remaining,
            ..
        } = &mut state.kind
        {
            *state_remaining = remaining;
            Ok(())
        } else {
            Err(RuntimeError::TypeError)
        }
    }

    // ============================================================
    //                      METHOD DISPATCH
    // ============================================================

    pub(crate) fn invoke_iterator_method(
        &mut self,
        method: &str,
        args: &[Value],
    ) -> Result<Value, RuntimeError> {
        let receiver = args.first().cloned().ok_or(RuntimeError::TypeError)?;

        match method {
            // ====================================================
            // next()
            // ====================================================
            "next" => {
                Self::expect_method_args(args, 1)?;

                self.iterator_next_value(&receiver)
            }

            // ====================================================
            // has_next()
            // ====================================================
            "has_next" => {
                Self::expect_method_args(args, 1)?;

                Ok(Value::Boolean(self.iterator_has_next_value(&receiver)?))
            }

            // ====================================================
            // peek()
            // ====================================================
            "peek" => {
                Self::expect_method_args(args, 1)?;

                self.iterator_peek_value(&receiver)
            }

            // ====================================================
            // map(fn)
            // ====================================================
            "map" => {
                Self::expect_method_args(args, 2)?;
                Self::ensure_callable(&args[1])?;

                Ok(Value::new_map_iterator(receiver, args[1].clone()))
            }

            // ====================================================
            // filter(fn)
            // ====================================================
            "filter" => {
                Self::expect_method_args(args, 2)?;
                Self::ensure_callable(&args[1])?;

                Ok(Value::new_filter_iterator(receiver, args[1].clone()))
            }

            // ============================================================
            //                         take(n)
            // ============================================================
            "take" => {
                Self::expect_method_args(args, 2)?;

                let count = Self::expect_non_negative_integer(&args[1])?;

                Ok(Value::new_take_iterator(receiver, count))
            }

            // ============================================================
            //                         skip(n)
            // ============================================================
            "skip" => {
                Self::expect_method_args(args, 2)?;

                let count = Self::expect_non_negative_integer(&args[1])?;

                Ok(Value::new_skip_iterator(receiver, count))
            }

            // ====================================================
            // collect()
            // ====================================================
            "collect" => {
                Self::expect_method_args(args, 1)?;

                self.iterator_collect(&receiver)
            }

            // ====================================================
            // count()
            // ====================================================
            "count" => {
                Self::expect_method_args(args, 1)?;

                self.iterator_count(&receiver)
            }

            // ====================================================
            // any(fn)
            // ====================================================
            "any" => {
                Self::expect_method_args(args, 2)?;
                Self::ensure_callable(&args[1])?;

                self.iterator_any(&receiver, args[1].clone())
            }

            // ====================================================
            // all(fn)
            // ====================================================
            "all" => {
                Self::expect_method_args(args, 2)?;
                Self::ensure_callable(&args[1])?;

                self.iterator_all(&receiver, args[1].clone())
            }
            // ============================================================
            //                         to_iterator()
            // ============================================================
            "to_iterator" => {
                Self::expect_method_args(args, 1)?;

                Ok(receiver.to_iterator()?)
            }
            _ => Err(RuntimeError::ObjectFieldNotFound {
                name: method.to_string(),
                suggestion: None,
            }),
        }
    }
    fn expect_non_negative_integer(value: &Value) -> Result<usize, RuntimeError> {
        match value {
            Value::Integer(value) if *value >= 0 => {
                usize::try_from(*value).map_err(|_| RuntimeError::TypeError)
            }

            _ => Err(RuntimeError::TypeError),
        }
    }
    // ============================================================
    //                       COLLECT
    // ============================================================

    fn iterator_collect(&mut self, iterator: &Value) -> Result<Value, RuntimeError> {
        let mut values = Vec::new();

        loop {
            match self.iterator_next_value(iterator) {
                Ok(value) => values.push(value),

                Err(RuntimeError::IteratorExhausted) => {
                    break;
                }

                Err(error) => return Err(error),
            }
        }

        Ok(Value::new_array(values))
    }

    // ============================================================
    //                         COUNT
    // ============================================================

    fn iterator_count(&mut self, iterator: &Value) -> Result<Value, RuntimeError> {
        let mut count = 0usize;

        loop {
            match self.iterator_next_value(iterator) {
                Ok(_) => {
                    count = count.checked_add(1).ok_or(RuntimeError::InvalidFunction)?;
                }

                Err(RuntimeError::IteratorExhausted) => {
                    break;
                }

                Err(error) => return Err(error),
            }
        }

        Ok(Value::Integer(count as i64))
    }

    // ============================================================
    //                           ANY
    // ============================================================

    fn iterator_any(&mut self, iterator: &Value, callback: Value) -> Result<Value, RuntimeError> {
        loop {
            let value = match self.iterator_next_value(iterator) {
                Ok(value) => value,

                Err(RuntimeError::IteratorExhausted) => {
                    return Ok(Value::Boolean(false));
                }

                Err(error) => return Err(error),
            };

            let result = self.invoke_sync(callback.clone(), std::slice::from_ref(&value))?;

            if result.is_truthy() {
                return Ok(Value::Boolean(true));
            }
        }
    }

    // ============================================================
    //                           ALL
    // ============================================================

    fn iterator_all(&mut self, iterator: &Value, callback: Value) -> Result<Value, RuntimeError> {
        loop {
            let value = match self.iterator_next_value(iterator) {
                Ok(value) => value,

                Err(RuntimeError::IteratorExhausted) => {
                    return Ok(Value::Boolean(true));
                }

                Err(error) => return Err(error),
            };

            let result = self.invoke_sync(callback.clone(), std::slice::from_ref(&value))?;

            if !result.is_truthy() {
                return Ok(Value::Boolean(false));
            }
        }
    }

    // ============================================================
    //                          HELPERS
    // ============================================================

    fn expect_method_args(args: &[Value], expected: usize) -> Result<(), RuntimeError> {
        if args.len() != expected {
            return Err(RuntimeError::WrongArgumentCount {
                expected: expected - 1,
                found: args.len().saturating_sub(1),
            });
        }

        Ok(())
    }

    fn ensure_callable(value: &Value) -> Result<(), RuntimeError> {
        match value {
            Value::NativeFunction(_) => Ok(()),

            Value::Object(handle) => {
                let object = handle.borrow();

                match &*object {
                    Object::Function(_) | Object::Closure(_) => Ok(()),

                    _ => Err(RuntimeError::NotCallable),
                }
            }

            _ => Err(RuntimeError::NotCallable),
        }
    }
}
