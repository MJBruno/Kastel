use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::{Rc, Weak};

use crate::error::runtime_error::RuntimeError;
use crate::module::module::ModuleLoader;
use crate::runtime::channel::ChannelState;
use crate::runtime::gc_handle::Gc;
use crate::runtime::object::Object;
use crate::runtime::value::Value;

use super::{RunStatus, VirtualMachine};

pub(crate) const DEFAULT_TASK_QUANTUM: usize = 1024;

/// Nombre maximal de quanta exécutés par `drain` en fin de programme. Au-delà,
/// les tâches encore actives (boucle infinie avec `yield`, par exemple) sont
/// abandonnées avec un avertissement au lieu de bloquer la sortie.
pub(crate) const DRAIN_MAX_POLLS: usize = 200_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TaskStateStatus {
    Ready,
    Waiting,
    Completed,
    Failed,
    Cancelled,
}

pub(crate) struct TaskState {
    pub(crate) vm: VirtualMachine,
    pub(crate) status: TaskStateStatus,
    pub(crate) result: Option<Value>,
    pub(crate) error: Option<RuntimeError>,
    /// L'échec (`error`) a été remonté à quelqu'un (`join`) ou signalé.
    pub(crate) observed: bool,
    /// Plus aucun handle ne référence la tâche : elle tourne jusqu'au bout,
    /// puis son slot est libéré (et son échec éventuel signalé).
    pub(crate) detached: bool,
}

#[derive(Debug)]
pub struct TaskHandle {
    pub(crate) id: usize,
    pub(crate) scheduler: Weak<RefCell<Scheduler>>,
}

impl PartialEq for TaskHandle {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && Weak::ptr_eq(&self.scheduler, &other.scheduler)
    }
}

impl Eq for TaskHandle {}

impl Drop for TaskHandle {
    fn drop(&mut self) {
        let Some(shared) = self.scheduler.upgrade() else {
            return;
        };

        // `try_borrow_mut` : le handle peut être détruit pendant qu'un emprunt
        // du scheduler est actif (ex. pendant `poll`). Dans ce cas rare, le slot
        // n'est pas libéré tout de suite, mais on ne panique jamais.
        if let Ok(mut scheduler) = shared.try_borrow_mut() {
            scheduler.detach(self.id);
        }
    }
}

pub(crate) struct Scheduler {
    pub(crate) tasks: Vec<Option<TaskState>>,
    pub(crate) ready: VecDeque<usize>,
    pub(crate) running: Vec<usize>,
    pub(crate) next_id: usize,
    pub(crate) quantum: usize,
    pub(crate) waiting_channels: HashMap<usize, VecDeque<usize>>,
    /// Annulations demandées pendant qu'une tâche est en cours d'exécution.
    /// La tâche courante n'est pas présente dans `tasks` pendant son quantum,
    /// donc la demande doit vivre à côté du tableau principal.
    pub(crate) cancel_requested: std::collections::HashSet<usize>,
}

impl Scheduler {
    pub(crate) fn new() -> Self {
        Self {
            tasks: Vec::new(),
            ready: VecDeque::new(),
            running: Vec::new(),
            next_id: 0,
            quantum: DEFAULT_TASK_QUANTUM,
            waiting_channels: HashMap::new(),
            cancel_requested: std::collections::HashSet::new(),
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
            id,
        )?;

        let state = TaskState {
            vm,
            status: TaskStateStatus::Ready,
            result: None,
            error: None,
            observed: false,
            detached: false,
        };

        let mut scheduler = shared.borrow_mut();
        if id != scheduler.tasks.len() {
            return Err(RuntimeError::InvalidFunction);
        }
        scheduler.tasks.push(Some(state));
        scheduler.ready.push_back(id);

        Ok(Rc::new(TaskHandle {
            id,
            scheduler: weak,
        }))
    }

    pub(crate) fn poll(shared: &Rc<RefCell<Self>>) -> Result<bool, RuntimeError> {
        let (id, mut task, quantum) = {
            let mut scheduler = shared.borrow_mut();

            let id = loop {
                let Some(id) = scheduler.ready.pop_front() else {
                    if scheduler.has_live_tasks() {
                        return Err(RuntimeError::TaskDeadlock);
                    }
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

        let cancellation_requested = scheduler.cancel_requested.remove(&id);

        if cancellation_requested {
            task.status = TaskStateStatus::Cancelled;
            task.result = None;
            task.error = None;
            task.vm.release_task_resources();
        } else {
            match outcome {
                Ok(RunStatus::Yielded) => {
                    task.status = TaskStateStatus::Ready;
                    scheduler.ready.push_back(id);
                }
                Ok(RunStatus::Waiting) => {
                    task.status = TaskStateStatus::Waiting;
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
        }

        let finished = matches!(
            task.status,
            TaskStateStatus::Completed | TaskStateStatus::Failed | TaskStateStatus::Cancelled
        );

        // Une tâche terminée n'a plus besoin de sa pile ni de ses frames :
        // seuls `result` / `error` restent utiles (pour `join`).
        if finished && !cancellation_requested {
            task.vm.release_task_resources();
        }

        if finished && task.detached {
            if task.status == TaskStateStatus::Failed && !task.observed {
                Self::report_unobserved(id, task.error.as_ref());
            }
            // Le slot reste à `None` : la tâche est libérée.
        } else {
            scheduler.tasks[id] = Some(task);
        }

        Ok(true)
    }

    /// Appelé quand le dernier handle d'une tâche est détruit.
    fn detach(&mut self, id: usize) {
        let Some(slot) = self.tasks.get_mut(id) else {
            return;
        };

        // `None` : tâche en cours d'exécution (sortie temporairement de
        // `tasks`) ou déjà libérée. Cas rare, sans conséquence fonctionnelle.
        let Some(task) = slot.as_mut() else {
            return;
        };

        match task.status {
            TaskStateStatus::Ready | TaskStateStatus::Waiting => task.detached = true,
            TaskStateStatus::Completed | TaskStateStatus::Cancelled => *slot = None,
            TaskStateStatus::Failed => {
                if !task.observed {
                    Self::report_unobserved(id, task.error.as_ref());
                }
                *slot = None;
            }
        }
    }

    fn report_unobserved(id: usize, error: Option<&RuntimeError>) {
        match error {
            Some(error) => {
                eprintln!("Erreur : la tâche {id} a échoué sans que personne ne l'attende : {error}")
            }
            None => eprintln!("Erreur : la tâche {id} a échoué sans que personne ne l'attende"),
        }
    }

    /// Fin de programme : exécute les tâches qui n'ont jamais été jointes
    /// (jusqu'à ce qu'elles se terminent ou se bloquent), puis signale les
    /// échecs que personne n'a observés.
    pub(crate) fn drain(shared: &Rc<RefCell<Self>>) {
        let mut polls = 0usize;

        loop {
            match Self::poll(shared) {
                Ok(true) => {
                    polls += 1;

                    if polls >= DRAIN_MAX_POLLS {
                        eprintln!(
                            "Avertissement : des tâches encore actives ont été abandonnées à la fin du programme"
                        );
                        break;
                    }
                }
                Ok(false) => break,
                Err(RuntimeError::TaskDeadlock) => {
                    let scheduler = shared.borrow();
                    let waiting: Vec<usize> = scheduler
                        .tasks
                        .iter()
                        .enumerate()
                        .filter_map(|(id, slot)| {
                            slot.as_ref()
                                .filter(|task| task.status == TaskStateStatus::Waiting)
                                .map(|_| id)
                        })
                        .collect();

                    if waiting.is_empty() {
                        eprintln!("Erreur du scheduler : Task scheduler deadlock detected.");
                    } else {
                        eprintln!(
                            "Erreur du scheduler : deadlock détecté, tâches en attente : {:?}",
                            waiting
                        );
                    }
                    break;
                }
                Err(error) => {
                    eprintln!("Erreur du scheduler : {error}");
                    break;
                }
            }
        }

        let mut scheduler = shared.borrow_mut();

        for (id, slot) in scheduler.tasks.iter_mut().enumerate() {
            if let Some(task) = slot
                && task.status == TaskStateStatus::Failed && !task.observed {
                    task.observed = true;
                    Self::report_unobserved(id, task.error.as_ref());
                }
        }
    }

    fn has_live_tasks(&self) -> bool {
        !self.running.is_empty()
            || self.tasks.iter().flatten().any(|task| {
                matches!(task.status, TaskStateStatus::Ready | TaskStateStatus::Waiting)
            })
    }

    fn channel_key(channel: &Rc<RefCell<ChannelState>>) -> usize {
        Rc::as_ptr(channel) as usize
    }

    pub(crate) fn wait_on_channel(
        shared: &Rc<RefCell<Self>>,
        task_id: usize,
        channel: Rc<RefCell<ChannelState>>,
    ) -> Result<(), RuntimeError> {
        let key = Self::channel_key(&channel);
        let mut scheduler = shared.borrow_mut();

        let state = scheduler.tasks.get(task_id).and_then(Option::as_ref);

        // The running task is temporarily removed from `tasks` while its quantum
        // executes, so registration is also valid for a currently running task.
        if state.is_none() && !scheduler.running.contains(&task_id) {
            return Err(RuntimeError::TaskNotFound);
        }

        let waiters = scheduler.waiting_channels.entry(key).or_default();
        if !waiters.contains(&task_id) {
            waiters.push_back(task_id);
        }

        Ok(())
    }

    pub(crate) fn close_channel(
        shared: &Rc<RefCell<Self>>,
        channel: &Rc<RefCell<ChannelState>>,
    ) -> Result<bool, RuntimeError> {
        let closed_now = channel.borrow_mut().close();
        if !closed_now {
            return Ok(false);
        }

        let key = Self::channel_key(channel);
        let mut scheduler = shared.borrow_mut();
        let Some(waiters) = scheduler.waiting_channels.remove(&key) else {
            return Ok(true);
        };

        for task_id in waiters {
            let should_wake = {
                let Some(task) = scheduler.tasks.get_mut(task_id).and_then(Option::as_mut) else {
                    continue;
                };

                if task.status != TaskStateStatus::Waiting {
                    false
                } else {
                    task.vm.resume_from_channel_error(RuntimeError::ChannelClosed)?;
                    task.status = TaskStateStatus::Ready;
                    true
                }
            };

            if should_wake {
                scheduler.ready.push_back(task_id);
            }
        }

        Ok(true)
    }

    pub(crate) fn wake_one_channel(
        shared: &Rc<RefCell<Self>>,
        channel: &Rc<RefCell<ChannelState>>,
        value: Value,
    ) -> Result<bool, RuntimeError> {
        let key = Self::channel_key(channel);
        let mut scheduler = shared.borrow_mut();
        let mut pending_value = Some(value);

        loop {
            let task_id = match scheduler.waiting_channels.get_mut(&key) {
                Some(waiters) => waiters.pop_front(),
                None => None,
            };

            if scheduler
                .waiting_channels
                .get(&key)
                .is_some_and(|waiters| waiters.is_empty())
            {
                scheduler.waiting_channels.remove(&key);
            }

            let Some(task_id) = task_id else {
                return Ok(false);
            };

            let should_wake = {
                let Some(task) = scheduler.tasks.get_mut(task_id).and_then(Option::as_mut) else {
                    continue;
                };

                if task.status != TaskStateStatus::Waiting {
                    false
                } else {
                    let value = pending_value.take().ok_or(RuntimeError::NativeError)?;

                    task.vm.resume_from_channel(value)?;
                    task.status = TaskStateStatus::Ready;
                    true
                }
            };

            if should_wake {
                scheduler.ready.push_back(task_id);
                return Ok(true);
            }
        }
    }

    pub(crate) fn cancel(shared: &Rc<RefCell<Self>>, id: usize) -> Result<(), RuntimeError> {
        let mut scheduler = shared.borrow_mut();

        if scheduler.running.contains(&id) {
            scheduler.cancel_requested.insert(id);
            return Ok(());
        }

        let status = scheduler
            .tasks
            .get(id)
            .and_then(Option::as_ref)
            .map(|task| task.status)
            .ok_or(RuntimeError::TaskNotFound)?;

        match status {
            TaskStateStatus::Ready | TaskStateStatus::Waiting => {
                let task = scheduler
                    .tasks
                    .get_mut(id)
                    .and_then(Option::as_mut)
                    .ok_or(RuntimeError::TaskNotFound)?;

                task.status = TaskStateStatus::Cancelled;
                task.result = None;
                task.error = None;
                task.vm.release_task_resources();

                scheduler.ready.retain(|queued_id| *queued_id != id);

                for waiters in scheduler.waiting_channels.values_mut() {
                    waiters.retain(|waiting_id| *waiting_id != id);
                }
                scheduler
                    .waiting_channels
                    .retain(|_, waiters| !waiters.is_empty());
            }
            TaskStateStatus::Completed
            | TaskStateStatus::Failed
            | TaskStateStatus::Cancelled => {}
        }

        Ok(())
    }

    pub(crate) fn join(shared: &Rc<RefCell<Self>>, id: usize) -> Result<Value, RuntimeError> {
        loop {
            {
                let mut scheduler = shared.borrow_mut();

                if scheduler.running.contains(&id) {
                    return Err(RuntimeError::TaskDeadlock);
                }

                let state = scheduler
                    .tasks
                    .get_mut(id)
                    .and_then(Option::as_mut)
                    .ok_or(RuntimeError::TaskNotFound)?;

                match state.status {
                    TaskStateStatus::Completed => {
                        return Ok(state.result.clone().unwrap_or(Value::None));
                    }
                    TaskStateStatus::Failed => {
                        // L'échec est remonté à l'appelant : il est observé.
                        state.observed = true;
                        return Err(state.error.clone().unwrap_or(RuntimeError::NativeError));
                    }
                    TaskStateStatus::Cancelled => {
                        return Err(RuntimeError::TaskCancelled);
                    }
                    TaskStateStatus::Ready | TaskStateStatus::Waiting => {}
                }
            }

            if !Self::poll(shared)? {
                return Err(RuntimeError::TaskDeadlock);
            }
        }
    }

    pub(crate) fn status(
        shared: &Rc<RefCell<Self>>,
        id: usize,
    ) -> Result<&'static str, RuntimeError> {
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
            Some(TaskStateStatus::Waiting) => Ok("waiting"),
            Some(TaskStateStatus::Completed) => Ok("done"),
            Some(TaskStateStatus::Failed) => Ok("failed"),
            Some(TaskStateStatus::Cancelled) => Ok("cancelled"),
            None => Err(RuntimeError::TaskNotFound),
        }
    }

    pub(crate) fn is_done(shared: &Rc<RefCell<Self>>, id: usize) -> Result<bool, RuntimeError> {
        Ok(matches!(Self::status(shared, id)?, "done" | "failed" | "cancelled"))
    }

    pub(crate) fn append_gc_roots(
        &self,
        values: &mut Vec<Value>,
        upvalues: &mut Vec<Rc<RefCell<crate::runtime::upvalue::ObjUpvalue>>>,
    ) {
        for task in self.tasks.iter().flatten() {
            values.extend(task.vm.stack.iter().cloned());
            values.extend(task.vm.temp_roots.iter().cloned());
            if let Some(channel) = &task.vm.waiting_channel {
                values.push(channel.clone());
            }
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
