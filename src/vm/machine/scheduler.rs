use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::{Rc, Weak};

use crate::error::runtime_error::RuntimeError;
use crate::module::module::ModuleLoader;
use crate::runtime::gc_handle::Gc;
use crate::runtime::object::Object;
use crate::runtime::value::Value;

use super::{RunStatus, VirtualMachine};

pub(crate) const DEFAULT_TASK_QUANTUM: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TaskStateStatus {
    Ready,
    Completed,
    Failed,
}

pub(crate) struct TaskState {
    pub(crate) vm: VirtualMachine,
    pub(crate) status: TaskStateStatus,
    pub(crate) result: Option<Value>,
    pub(crate) error: Option<RuntimeError>,
}

#[derive(Debug)]
pub(crate) struct TaskHandle {
    pub(crate) id: usize,
    pub(crate) scheduler: Weak<RefCell<Scheduler>>,
}

impl PartialEq for TaskHandle {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && Weak::ptr_eq(&self.scheduler, &other.scheduler)
    }
}

impl Eq for TaskHandle {}

pub(crate) struct Scheduler {
    pub(crate) tasks: Vec<Option<TaskState>>,
    pub(crate) ready: VecDeque<usize>,
    pub(crate) running: Vec<usize>,
    pub(crate) next_id: usize,
    pub(crate) quantum: usize,
}

impl Scheduler {
    pub(crate) fn new() -> Self {
        Self {
            tasks: Vec::new(),
            ready: VecDeque::new(),
            running: Vec::new(),
            next_id: 0,
            quantum: DEFAULT_TASK_QUANTUM,
        }
    }

    pub(crate) fn spawn(
        shared: &Rc<RefCell<Self>>,
        closure: Gc<Object>,
        arguments: Vec<Value>,
        globals: Rc<RefCell<std::collections::HashMap<String, Value>>>,
        module_loader: ModuleLoader,
        module_path: Option<std::path::PathBuf>,
    ) -> Result<Rc<TaskHandle>, RuntimeError> {
        let id = {
            let mut scheduler = shared.borrow_mut();
            let id = scheduler.next_id;
            scheduler.next_id = scheduler
                .next_id
                .checked_add(1)
                .ok_or(RuntimeError::InvalidFunction)?;
            id
        };

        let weak = Rc::downgrade(shared);
        let vm = VirtualMachine::new_task(
            closure,
            arguments,
            globals,
            module_loader,
            module_path,
            weak.clone(),
        )?;

        let state = TaskState {
            vm,
            status: TaskStateStatus::Ready,
            result: None,
            error: None,
        };

        let mut scheduler = shared.borrow_mut();
        if id != scheduler.tasks.len() {
            return Err(RuntimeError::InvalidFunction);
        }
        scheduler.tasks.push(Some(state));
        scheduler.ready.push_back(id);

        Ok(Rc::new(TaskHandle { id, scheduler: weak }))
    }

    pub(crate) fn poll(shared: &Rc<RefCell<Self>>) -> Result<bool, RuntimeError> {
        let (id, mut task, quantum) = {
            let mut scheduler = shared.borrow_mut();

            let id = loop {
                let Some(id) = scheduler.ready.pop_front() else {
                    return Ok(false);
                };

                if scheduler.tasks.get(id).and_then(Option::as_ref).is_some() {
                    break id;
                }
            };

            let task = scheduler
                .tasks
                .get_mut(id)
                .and_then(Option::take)
                .ok_or(RuntimeError::TaskNotFound)?;

            scheduler.running.push(id);
            let quantum = scheduler.quantum;
            (id, task, quantum)
        };

        let outcome = task.vm.run_quantum(quantum);

        let mut scheduler = shared.borrow_mut();
        scheduler.running.pop();

        match outcome {
            Ok(RunStatus::Yielded) => {
                task.status = TaskStateStatus::Ready;
                scheduler.ready.push_back(id);
            }
            Ok(RunStatus::Completed) => {
                task.status = TaskStateStatus::Completed;
                task.result = Some(task.vm.last_result_value());
            }
            Err(error) => {
                task.status = TaskStateStatus::Failed;
                task.error = Some(error);
            }
        }

        scheduler.tasks[id] = Some(task);
        Ok(true)
    }

    pub(crate) fn join(shared: &Rc<RefCell<Self>>, id: usize) -> Result<Value, RuntimeError> {
        loop {
            {
                let scheduler = shared.borrow();

                if scheduler.running.contains(&id) {
                    return Err(RuntimeError::TaskDeadlock);
                }

                let state = scheduler
                    .tasks
                    .get(id)
                    .and_then(Option::as_ref)
                    .ok_or(RuntimeError::TaskNotFound)?;

                match state.status {
                    TaskStateStatus::Completed => {
                        return Ok(state.result.clone().unwrap_or(Value::None));
                    }
                    TaskStateStatus::Failed => {
                        return Err(state.error.clone().unwrap_or(RuntimeError::NativeError));
                    }
                    TaskStateStatus::Ready => {}
                }
            }

            if !Self::poll(shared)? {
                return Err(RuntimeError::TaskDeadlock);
            }
        }
    }

    pub(crate) fn status(shared: &Rc<RefCell<Self>>, id: usize) -> Result<&'static str, RuntimeError> {
        let scheduler = shared.borrow();

        if scheduler.running.contains(&id) {
            return Ok("running");
        }

        match scheduler
            .tasks
            .get(id)
            .and_then(Option::as_ref)
            .map(|task| task.status)
        {
            Some(TaskStateStatus::Ready) => Ok("ready"),
            Some(TaskStateStatus::Completed) => Ok("done"),
            Some(TaskStateStatus::Failed) => Ok("failed"),
            None => Err(RuntimeError::TaskNotFound),
        }
    }

    pub(crate) fn is_done(shared: &Rc<RefCell<Self>>, id: usize) -> Result<bool, RuntimeError> {
        Ok(matches!(Self::status(shared, id)?, "done" | "failed"))
    }

    pub(crate) fn append_gc_roots(
        &self,
        values: &mut Vec<Value>,
        upvalues: &mut Vec<Rc<RefCell<crate::runtime::upvalue::ObjUpvalue>>>,
    ) {
        for task in self.tasks.iter().flatten() {
            values.extend(task.vm.stack.iter().cloned());
            values.extend(task.vm.temp_roots.iter().cloned());
            values.extend(task.vm.globals.borrow().values().cloned());
            values.extend(
                task.vm
                    .frames
                    .iter()
                    .map(|frame| Value::Object(frame.closure.clone())),
            );

            if let Some(exception) = &task.vm.pending_exception {
                values.push(exception.value.clone());
            }

            if let Some(result) = &task.result {
                values.push(result.clone());
            }

            if let Some(error) = &task.error {
                Self::root_runtime_error(error, values);
            }

            upvalues.extend(task.vm.open_upvalues.iter().cloned());
        }
    }

    fn root_runtime_error(error: &RuntimeError, values: &mut Vec<Value>) {
        match error {
            RuntimeError::Thrown(value) => values.push(value.clone()),
            RuntimeError::WithLocation { source, .. } => Self::root_runtime_error(source, values),
            _ => {}
        }
    }
}
