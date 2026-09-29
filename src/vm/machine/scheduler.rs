use std::cell::RefCell;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::rc::{Rc, Weak};
use std::thread;
use std::time::{Duration, Instant};

use crate::error::runtime_error::RuntimeError;
use crate::module::module::ModuleLoader;
use crate::runtime::channel::ChannelState;
use crate::runtime::gc_handle::Gc;
use crate::runtime::mutex::MutexState;
use crate::runtime::object::Object;
use crate::runtime::semaphore::SemaphoreState;
use crate::runtime::value::Value;
use crate::runtime::wait_group::WaitGroupState;

use super::{RunStatus, VirtualMachine};

pub(crate) const DEFAULT_TASK_QUANTUM: usize = 1024;

/// Nombre maximal de quanta exécutés par `drain` en fin de programme. Au-delà,
/// les tâches encore actives (boucle infinie avec `yield`, par exemple) sont
/// abandonnées avec un avertissement au lieu de bloquer la sortie.
pub(crate) const DRAIN_MAX_POLLS: usize = 200_000;

/// Profondeur maximale de tâches imbriquées (une tâche qui `join` une autre
/// exécute la cible sur la pile native de la VM appelante). Au-delà, `join`
/// / `recv` / `select` lèvent `TaskNestingTooDeep` (erreur rattrapable) au lieu
/// de faire déborder la pile du processus.
pub(crate) const MAX_NESTED_TASK_DEPTH: usize = 64;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PollOutcome {
    Progressed,
    Idle,
    DeadlineReached,
}

pub(crate) struct Scheduler {
    pub(crate) tasks: Vec<Option<TaskState>>,
    pub(crate) ready: VecDeque<usize>,
    pub(crate) running: Vec<usize>,
    pub(crate) next_id: usize,
    pub(crate) quantum: usize,
    pub(crate) waiting_channels: HashMap<usize, VecDeque<usize>>,
    /// Pour les tâches en `select`, associe chaque tâche aux channels sur
    /// lesquels elle est enregistrée. Une même tâche peut donc apparaître
    /// dans plusieurs files d'attente, mais ne sera réveillée qu'une seule fois.
    pub(crate) waiting_selects: HashMap<usize, Vec<usize>>,
    /// Annulations demandées pendant qu'une tâche est en cours d'exécution.
    /// La tâche courante n'est pas présente dans `tasks` pendant son quantum,
    /// donc la demande doit vivre à côté du tableau principal.
    pub(crate) cancel_requested: std::collections::HashSet<usize>,
    /// Tâches endormies : `task_id -> échéance`. Le tas contient les mêmes
    /// entrées pour obtenir rapidement la prochaine échéance ; la HashMap
    /// permet d'ignorer proprement les anciennes entrées après annulation.
    pub(crate) sleeping_tasks: HashMap<usize, Instant>,
    pub(crate) timers: BinaryHeap<Reverse<(Instant, usize)>>,
    /// Tâche actuellement en attente sur un mutex. Une tâche ne peut bloquer
    /// que sur un seul mutex à la fois.
    pub(crate) waiting_mutexes: HashMap<usize, Rc<RefCell<MutexState>>>,
    /// Mutex actuellement détenus par chaque tâche. Le scheduler les conserve
    /// pour pouvoir libérer automatiquement les verrous si une tâche termine,
    /// échoue ou est annulée.
    pub(crate) owned_mutexes: HashMap<usize, Vec<Rc<RefCell<MutexState>>>>,
    pub(crate) waiting_wait_groups: HashMap<usize, Rc<RefCell<WaitGroupState>>>,
    /// Sémaphore attendu par chaque tâche.
    pub(crate) waiting_semaphores: HashMap<usize, Rc<RefCell<SemaphoreState>>>,
    /// Permis de sémaphore détenus par chaque tâche.
    pub(crate) owned_semaphores: HashMap<usize, Vec<Rc<RefCell<SemaphoreState>>>>,
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
            waiting_selects: HashMap::new(),
            cancel_requested: std::collections::HashSet::new(),
            sleeping_tasks: HashMap::new(),
            timers: BinaryHeap::new(),
            waiting_mutexes: HashMap::new(),
            owned_mutexes: HashMap::new(),
            waiting_wait_groups: HashMap::new(),
            waiting_semaphores: HashMap::new(),
            owned_semaphores: HashMap::new(),
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
        match Self::poll_until(shared, None)? {
            PollOutcome::Progressed => Ok(true),
            PollOutcome::Idle | PollOutcome::DeadlineReached => Ok(false),
        }
    }

    pub(crate) fn poll_until(
        shared: &Rc<RefCell<Self>>,
        deadline: Option<Instant>,
    ) -> Result<PollOutcome, RuntimeError> {
        let (id, mut task, quantum) = {
            let mut scheduler = shared.borrow_mut();

            if scheduler.running.len() >= MAX_NESTED_TASK_DEPTH {
                return Err(RuntimeError::TaskNestingTooDeep);
            }

            let id = loop {
                scheduler.wake_expired_timers();

                if let Some(stop_at) = deadline
                    && Instant::now() >= stop_at
                {
                    return Ok(PollOutcome::DeadlineReached);
                }

                if let Some(id) = scheduler.ready.pop_front() {
                    if scheduler.tasks.get(id).and_then(Option::as_ref).is_some() {
                        break id;
                    }
                    continue;
                }

                if !scheduler.has_live_tasks() {
                    if let Some(stop_at) = deadline {
                        let remaining = stop_at.saturating_duration_since(Instant::now());
                        if !remaining.is_zero() {
                            drop(scheduler);
                            thread::sleep(remaining);
                            return Ok(PollOutcome::DeadlineReached);
                        }
                        return Ok(PollOutcome::DeadlineReached);
                    }
                    return Ok(PollOutcome::Idle);
                }

                let next_timer = scheduler.next_timer_deadline();
                let next_wakeup = match (next_timer, deadline) {
                    (Some(timer), Some(stop_at)) => timer.min(stop_at),
                    (Some(timer), None) => timer,
                    (None, Some(stop_at)) => stop_at,
                    (None, None) => return Err(RuntimeError::TaskDeadlock),
                };

                let now = Instant::now();
                if next_wakeup <= now {
                    continue;
                }

                let remaining = next_wakeup.saturating_duration_since(now);
                drop(scheduler);
                thread::sleep(remaining);
                scheduler = shared.borrow_mut();

                if let Some(stop_at) = deadline
                    && Instant::now() >= stop_at
                    && scheduler.ready.is_empty()
                {
                    return Ok(PollOutcome::DeadlineReached);
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

        let cancel_now = cancellation_requested
            && matches!(outcome, Ok(RunStatus::Yielded) | Ok(RunStatus::Waiting));

        if cancel_now {
            task.status = TaskStateStatus::Cancelled;
            task.result = None;
            task.error = None;
            Self::unregister_channel_waits(&mut scheduler, id);
            scheduler.unregister_mutex_wait(id);
            scheduler.unregister_wait_group_wait(id);
            scheduler.unregister_semaphore_wait(id);
            scheduler.unregister_timer(id);
            scheduler.release_owned_mutexes(id);
            scheduler.release_owned_semaphores(id);
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

        if finished {
            scheduler.release_owned_mutexes(id);
            scheduler.release_owned_semaphores(id);
        }

        if finished && !cancel_now {
            scheduler.unregister_timer(id);
            scheduler.unregister_wait_group_wait(id);
            scheduler.unregister_semaphore_wait(id);
            task.vm.release_task_resources();
        }

        if finished && task.detached {
            if task.status == TaskStateStatus::Failed && !task.observed {
                Self::report_unobserved(id, task.error.as_ref());
            }
        } else {
            scheduler.tasks[id] = Some(task);
        }

        Ok(PollOutcome::Progressed)
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
                eprintln!(
                    "Erreur : la tâche {id} a échoué sans que personne ne l'attende : {error}"
                )
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
                && task.status == TaskStateStatus::Failed
                && !task.observed
            {
                task.observed = true;
                Self::report_unobserved(id, task.error.as_ref());
            }
        }
    }

    fn has_live_tasks(&self) -> bool {
        !self.running.is_empty()
            || self.tasks.iter().flatten().any(|task| {
                matches!(
                    task.status,
                    TaskStateStatus::Ready | TaskStateStatus::Waiting
                )
            })
    }

    fn wake_expired_timers(&mut self) {
        let now = Instant::now();

        while let Some(Reverse((deadline, task_id))) = self.timers.peek().cloned() {
            if deadline > now {
                break;
            }

            self.timers.pop();

            let Some(current_deadline) = self.sleeping_tasks.get(&task_id).copied() else {
                continue;
            };

            if current_deadline != deadline {
                continue;
            }

            self.sleeping_tasks.remove(&task_id);

            let Some(task) = self.tasks.get_mut(task_id).and_then(Option::as_mut) else {
                continue;
            };

            if task.status != TaskStateStatus::Waiting {
                continue;
            }

            let select_timeout = task.vm.waiting_select_channels.is_some();
            if task.vm.resume_from_timer().is_err() {
                continue;
            }

            task.status = TaskStateStatus::Ready;
            if select_timeout {
                Self::unregister_channel_waits(self, task_id);
            }
            self.ready.push_back(task_id);
        }
    }

    fn next_timer_deadline(&self) -> Option<Instant> {
        self.timers
            .iter()
            .filter_map(|Reverse((deadline, task_id))| {
                (self.sleeping_tasks.get(task_id) == Some(deadline)).then_some(*deadline)
            })
            .min()
    }

    pub(crate) fn sleep_task(
        shared: &Rc<RefCell<Self>>,
        task_id: usize,
        duration: Duration,
    ) -> Result<Instant, RuntimeError> {
        let deadline = Instant::now()
            .checked_add(duration)
            .ok_or(RuntimeError::InvalidFunction)?;

        let mut scheduler = shared.borrow_mut();
        if scheduler
            .tasks
            .get(task_id)
            .and_then(Option::as_ref)
            .is_none()
            && !scheduler.running.contains(&task_id)
        {
            return Err(RuntimeError::TaskNotFound);
        }

        scheduler.sleeping_tasks.insert(task_id, deadline);
        scheduler.timers.push(Reverse((deadline, task_id)));
        Ok(deadline)
    }

    fn unregister_timer(&mut self, task_id: usize) {
        self.sleeping_tasks.remove(&task_id);
    }

    fn mutex_key(mutex: &Rc<RefCell<MutexState>>) -> usize {
        Rc::as_ptr(mutex) as usize
    }

    fn add_owned_mutex(&mut self, task_id: usize, mutex: Rc<RefCell<MutexState>>) {
        let key = Self::mutex_key(&mutex);
        let owned = self.owned_mutexes.entry(task_id).or_default();
        if !owned.iter().any(|value| Self::mutex_key(value) == key) {
            owned.push(mutex);
        }
    }

    fn remove_owned_mutex(&mut self, task_id: usize, mutex: &Rc<RefCell<MutexState>>) {
        let key = Self::mutex_key(mutex);
        if let Some(owned) = self.owned_mutexes.get_mut(&task_id) {
            owned.retain(|value| Self::mutex_key(value) != key);
            if owned.is_empty() {
                self.owned_mutexes.remove(&task_id);
            }
        }
    }

    /// Essaie d'acquérir le mutex. `true` signifie que la tâche possède
    /// immédiatement le verrou ; `false` signifie qu'elle a été placée en FIFO
    /// et devra être réveillée par `unlock()`.
    pub(crate) fn lock_mutex(
        shared: &Rc<RefCell<Self>>,
        task_id: usize,
        mutex: &Rc<RefCell<MutexState>>,
    ) -> Result<bool, RuntimeError> {
        let mut scheduler = shared.borrow_mut();
        let state = scheduler.tasks.get(task_id).and_then(Option::as_ref);
        if state.is_none() && !scheduler.running.contains(&task_id) {
            return Err(RuntimeError::TaskNotFound);
        }

        let owner = mutex.borrow().owner;
        match owner {
            None => {
                mutex.borrow_mut().owner = Some(task_id);
                scheduler.add_owned_mutex(task_id, mutex.clone());
                Ok(true)
            }
            Some(owner_id) if owner_id == task_id => Err(RuntimeError::MutexDeadlock),
            Some(_) => {
                let mut state = mutex.borrow_mut();
                if !state.waiters.contains(&task_id) {
                    state.waiters.push_back(task_id);
                }
                drop(state);
                scheduler.waiting_mutexes.insert(task_id, mutex.clone());
                Ok(false)
            }
        }
    }

    /// Tente d'acquérir le mutex sans mettre la tâche en attente.
    pub(crate) fn try_lock_mutex(
        shared: &Rc<RefCell<Self>>,
        task_id: usize,
        mutex: &Rc<RefCell<MutexState>>,
    ) -> Result<bool, RuntimeError> {
        let mut scheduler = shared.borrow_mut();
        let state = scheduler.tasks.get(task_id).and_then(Option::as_ref);
        if state.is_none() && !scheduler.running.contains(&task_id) {
            return Err(RuntimeError::TaskNotFound);
        }

        let owner = mutex.borrow().owner;

        match owner {
            None => {
                mutex.borrow_mut().owner = Some(task_id);
                scheduler.add_owned_mutex(task_id, mutex.clone());
                Ok(true)
            }
            Some(_) => Ok(false),
        }
    }

    fn wake_next_mutex_waiter(&mut self, mutex: &Rc<RefCell<MutexState>>) {
        loop {
            let next_id = {
                let mut state = mutex.borrow_mut();
                state.waiters.pop_front()
            };

            let Some(next_id) = next_id else {
                mutex.borrow_mut().owner = None;
                return;
            };

            let should_wake = self
                .tasks
                .get(next_id)
                .and_then(Option::as_ref)
                .is_some_and(|task| {
                    task.status == TaskStateStatus::Waiting
                        && self.waiting_mutexes.get(&next_id).is_some_and(|waiting| {
                            Self::mutex_key(waiting) == Self::mutex_key(mutex)
                        })
                });

            self.waiting_mutexes.remove(&next_id);

            if !should_wake {
                continue;
            }

            mutex.borrow_mut().owner = Some(next_id);
            self.add_owned_mutex(next_id, mutex.clone());

            let Some(task) = self.tasks.get_mut(next_id).and_then(Option::as_mut) else {
                self.remove_owned_mutex(next_id, mutex);
                continue;
            };

            if task.vm.resume_from_mutex().is_err() {
                self.remove_owned_mutex(next_id, mutex);
                continue;
            }

            task.status = TaskStateStatus::Ready;
            self.ready.push_back(next_id);
            return;
        }
    }

    pub(crate) fn unlock_mutex(
        shared: &Rc<RefCell<Self>>,
        task_id: usize,
        mutex: &Rc<RefCell<MutexState>>,
    ) -> Result<(), RuntimeError> {
        let mut scheduler = shared.borrow_mut();
        if mutex.borrow().owner != Some(task_id) {
            return Err(RuntimeError::MutexNotOwner);
        }

        scheduler.remove_owned_mutex(task_id, mutex);
        scheduler.wake_next_mutex_waiter(mutex);
        Ok(())
    }

    fn unregister_mutex_wait(&mut self, task_id: usize) {
        let Some(mutex) = self.waiting_mutexes.remove(&task_id) else {
            return;
        };

        mutex.borrow_mut().waiters.retain(|id| *id != task_id);
    }

    fn release_owned_mutexes(&mut self, task_id: usize) {
        let mutexes = self.owned_mutexes.remove(&task_id).unwrap_or_default();

        for mutex in mutexes {
            if mutex.borrow().owner == Some(task_id) {
                self.wake_next_mutex_waiter(&mutex);
            }
        }
    }

    fn semaphore_key(semaphore: &Rc<RefCell<SemaphoreState>>) -> usize {
        Rc::as_ptr(semaphore) as usize
    }

    fn add_owned_semaphore(&mut self, task_id: usize, semaphore: Rc<RefCell<SemaphoreState>>) {
        self.owned_semaphores
            .entry(task_id)
            .or_default()
            .push(semaphore);
    }

    fn remove_owned_semaphore(
        &mut self,
        task_id: usize,
        semaphore: &Rc<RefCell<SemaphoreState>>,
    ) -> bool {
        let key = Self::semaphore_key(semaphore);
        let Some(owned) = self.owned_semaphores.get_mut(&task_id) else {
            return false;
        };

        let position = owned
            .iter()
            .position(|value| Self::semaphore_key(value) == key);
        let Some(position) = position else {
            return false;
        };

        owned.remove(position);
        if owned.is_empty() {
            self.owned_semaphores.remove(&task_id);
        }
        true
    }

    pub(crate) fn acquire_semaphore(
        shared: &Rc<RefCell<Self>>,
        task_id: usize,
        semaphore: &Rc<RefCell<SemaphoreState>>,
    ) -> Result<bool, RuntimeError> {
        let mut scheduler = shared.borrow_mut();
        let state = scheduler.tasks.get(task_id).and_then(Option::as_ref);
        if state.is_none() && !scheduler.running.contains(&task_id) {
            return Err(RuntimeError::TaskNotFound);
        }

        let can_acquire = {
            let state = semaphore.borrow();
            state.permits > 0 && state.waiters.is_empty()
        };

        if can_acquire {
            semaphore.borrow_mut().permits -= 1;
            scheduler.add_owned_semaphore(task_id, semaphore.clone());
            return Ok(true);
        }

        {
            let mut state = semaphore.borrow_mut();
            if !state.waiters.contains(&task_id) {
                state.waiters.push_back(task_id);
            }
        }
        scheduler
            .waiting_semaphores
            .insert(task_id, semaphore.clone());
        Ok(false)
    }

    pub(crate) fn try_acquire_semaphore(
        shared: &Rc<RefCell<Self>>,
        task_id: usize,
        semaphore: &Rc<RefCell<SemaphoreState>>,
    ) -> Result<bool, RuntimeError> {
        let mut scheduler = shared.borrow_mut();
        let state = scheduler.tasks.get(task_id).and_then(Option::as_ref);
        if state.is_none() && !scheduler.running.contains(&task_id) {
            return Err(RuntimeError::TaskNotFound);
        }

        let can_acquire = {
            let state = semaphore.borrow();
            state.permits > 0 && state.waiters.is_empty()
        };

        if !can_acquire {
            return Ok(false);
        }

        semaphore.borrow_mut().permits -= 1;
        scheduler.add_owned_semaphore(task_id, semaphore.clone());
        Ok(true)
    }

    fn wake_next_semaphore_waiter(&mut self, semaphore: &Rc<RefCell<SemaphoreState>>) {
        loop {
            let next_id = semaphore.borrow_mut().waiters.pop_front();
            let Some(next_id) = next_id else {
                let mut state = semaphore.borrow_mut();
                state.permits = state.permits.saturating_add(1);
                return;
            };

            let should_wake = self
                .tasks
                .get(next_id)
                .and_then(Option::as_ref)
                .is_some_and(|task| {
                    task.status == TaskStateStatus::Waiting
                        && self
                            .waiting_semaphores
                            .get(&next_id)
                            .is_some_and(|waiting| {
                                Self::semaphore_key(waiting) == Self::semaphore_key(semaphore)
                            })
                });

            self.waiting_semaphores.remove(&next_id);

            if !should_wake {
                continue;
            }

            let resumed = {
                let Some(task) = self.tasks.get_mut(next_id).and_then(Option::as_mut) else {
                    continue;
                };

                if task.vm.resume_from_semaphore().is_err() {
                    false
                } else {
                    task.status = TaskStateStatus::Ready;
                    true
                }
            };

            if !resumed {
                continue;
            }

            self.add_owned_semaphore(next_id, semaphore.clone());
            self.ready.push_back(next_id);
            return;
        }
    }

    pub(crate) fn release_semaphore(
        shared: &Rc<RefCell<Self>>,
        task_id: usize,
        semaphore: &Rc<RefCell<SemaphoreState>>,
    ) -> Result<(), RuntimeError> {
        let mut scheduler = shared.borrow_mut();
        if !scheduler.remove_owned_semaphore(task_id, semaphore) {
            return Err(RuntimeError::SemaphoreNotOwner);
        }

        scheduler.wake_next_semaphore_waiter(semaphore);
        Ok(())
    }

    fn unregister_semaphore_wait(&mut self, task_id: usize) {
        let Some(semaphore) = self.waiting_semaphores.remove(&task_id) else {
            return;
        };

        semaphore.borrow_mut().waiters.retain(|id| *id != task_id);
    }

    fn release_owned_semaphores(&mut self, task_id: usize) {
        let semaphores = self.owned_semaphores.remove(&task_id).unwrap_or_default();

        for semaphore in semaphores {
            self.wake_next_semaphore_waiter(&semaphore);
        }
    }

    fn wait_group_key(wait_group: &Rc<RefCell<WaitGroupState>>) -> usize {
        Rc::as_ptr(wait_group) as usize
    }

    pub(crate) fn wait_group_add(
        shared: &Rc<RefCell<Self>>,
        wait_group: &Rc<RefCell<WaitGroupState>>,
        amount: usize,
    ) -> Result<(), RuntimeError> {
        let _scheduler = shared.borrow();
        let mut state = wait_group.borrow_mut();
        state.count = state
            .count
            .checked_add(amount)
            .ok_or(RuntimeError::InvalidFunction)?;
        Ok(())
    }

    pub(crate) fn wait_group_done(
        shared: &Rc<RefCell<Self>>,
        wait_group: &Rc<RefCell<WaitGroupState>>,
    ) -> Result<(), RuntimeError> {
        let mut scheduler = shared.borrow_mut();
        {
            let mut state = wait_group.borrow_mut();
            if state.count == 0 {
                return Err(RuntimeError::WaitGroupUnderflow);
            }
            state.count -= 1;
        }

        if wait_group.borrow().is_done() {
            Self::wake_wait_group_waiters(&mut scheduler, wait_group);
        }

        Ok(())
    }

    pub(crate) fn wait_on_wait_group(
        shared: &Rc<RefCell<Self>>,
        task_id: usize,
        wait_group: Rc<RefCell<WaitGroupState>>,
    ) -> Result<(), RuntimeError> {
        let mut scheduler = shared.borrow_mut();
        let state = scheduler.tasks.get(task_id).and_then(Option::as_ref);
        if state.is_none() && !scheduler.running.contains(&task_id) {
            return Err(RuntimeError::TaskNotFound);
        }

        if wait_group.borrow().is_done() {
            return Ok(());
        }

        let mut group = wait_group.borrow_mut();
        if !group.waiters.contains(&task_id) {
            group.waiters.push_back(task_id);
        }
        drop(group);
        scheduler.waiting_wait_groups.insert(task_id, wait_group);
        Ok(())
    }

    fn wake_wait_group_waiters(scheduler: &mut Self, wait_group: &Rc<RefCell<WaitGroupState>>) {
        let waiters = {
            let mut group = wait_group.borrow_mut();
            std::mem::take(&mut group.waiters)
        };

        for task_id in waiters {
            let should_wake = scheduler
                .tasks
                .get(task_id)
                .and_then(Option::as_ref)
                .is_some_and(|task| {
                    task.status == TaskStateStatus::Waiting
                        && scheduler
                            .waiting_wait_groups
                            .get(&task_id)
                            .is_some_and(|waiting| {
                                Self::wait_group_key(waiting) == Self::wait_group_key(wait_group)
                            })
                });

            scheduler.waiting_wait_groups.remove(&task_id);

            if !should_wake {
                continue;
            }

            let Some(task) = scheduler.tasks.get_mut(task_id).and_then(Option::as_mut) else {
                continue;
            };

            if task.vm.resume_from_wait_group().is_err() {
                continue;
            }

            task.status = TaskStateStatus::Ready;
            scheduler.ready.push_back(task_id);
        }
    }

    fn unregister_wait_group_wait(&mut self, task_id: usize) {
        let Some(wait_group) = self.waiting_wait_groups.remove(&task_id) else {
            return;
        };

        wait_group.borrow_mut().waiters.retain(|id| *id != task_id);
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

    pub(crate) fn wait_on_select(
        shared: &Rc<RefCell<Self>>,
        task_id: usize,
        channels: &[(Rc<RefCell<ChannelState>>, Value)],
    ) -> Result<(), RuntimeError> {
        if channels.is_empty() {
            return Err(RuntimeError::InvalidFunction);
        }

        let mut scheduler = shared.borrow_mut();
        let state = scheduler.tasks.get(task_id).and_then(Option::as_ref);
        if state.is_none() && !scheduler.running.contains(&task_id) {
            return Err(RuntimeError::TaskNotFound);
        }

        let mut keys = Vec::with_capacity(channels.len());

        for (channel, _) in channels {
            let key = Self::channel_key(channel);
            if !keys.contains(&key) {
                keys.push(key);

                let waiters = scheduler.waiting_channels.entry(key).or_default();
                if !waiters.contains(&task_id) {
                    waiters.push_back(task_id);
                }
            }
        }

        scheduler.waiting_selects.insert(task_id, keys);
        Ok(())
    }

    fn unregister_channel_waits(scheduler: &mut Self, task_id: usize) {
        for waiters in scheduler.waiting_channels.values_mut() {
            waiters.retain(|waiting_id| *waiting_id != task_id);
        }
        scheduler
            .waiting_channels
            .retain(|_, waiters| !waiters.is_empty());
        scheduler.waiting_selects.remove(&task_id);
    }

    fn select_index_for_channel(task: &TaskState, channel_key: usize) -> Option<usize> {
        task.vm
            .waiting_select_channels
            .as_ref()?
            .iter()
            .position(|value| {
                let Value::Object(handle) = value else {
                    return false;
                };

                let object = handle.borrow();
                match &*object {
                    Object::Channel(channel) => Self::channel_key(channel) == channel_key,
                    _ => false,
                }
            })
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
        let waiters = scheduler.waiting_channels.remove(&key).unwrap_or_default();

        for task_id in waiters {
            let is_select = scheduler.waiting_selects.contains_key(&task_id);
            let should_wake = {
                let Some(task) = scheduler.tasks.get(task_id).and_then(Option::as_ref) else {
                    continue;
                };

                task.status == TaskStateStatus::Waiting
            };

            if !should_wake {
                continue;
            }

            if is_select {
                let index = {
                    let task = scheduler
                        .tasks
                        .get(task_id)
                        .and_then(Option::as_ref)
                        .ok_or(RuntimeError::TaskNotFound)?;
                    Self::select_index_for_channel(task, key).ok_or(RuntimeError::TaskNotFound)?
                };

                let task = scheduler
                    .tasks
                    .get_mut(task_id)
                    .and_then(Option::as_mut)
                    .ok_or(RuntimeError::TaskNotFound)?;
                task.vm.resume_from_select(index, Value::None, true)?;
                task.status = TaskStateStatus::Ready;
                Self::unregister_channel_waits(&mut scheduler, task_id);
                scheduler.unregister_timer(task_id);
                scheduler.ready.push_back(task_id);
            } else {
                let task = scheduler
                    .tasks
                    .get_mut(task_id)
                    .and_then(Option::as_mut)
                    .ok_or(RuntimeError::TaskNotFound)?;
                task.vm
                    .resume_from_channel_error(RuntimeError::ChannelClosed)?;
                task.status = TaskStateStatus::Ready;
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
                let Some(task) = scheduler.tasks.get(task_id).and_then(Option::as_ref) else {
                    continue;
                };
                task.status == TaskStateStatus::Waiting
            };

            if !should_wake {
                continue;
            }

            if scheduler.waiting_selects.contains_key(&task_id) {
                let index = {
                    let task = scheduler
                        .tasks
                        .get(task_id)
                        .and_then(Option::as_ref)
                        .ok_or(RuntimeError::TaskNotFound)?;
                    Self::select_index_for_channel(task, key).ok_or(RuntimeError::TaskNotFound)?
                };

                let value = pending_value.take().ok_or(RuntimeError::NativeError)?;
                let task = scheduler
                    .tasks
                    .get_mut(task_id)
                    .and_then(Option::as_mut)
                    .ok_or(RuntimeError::TaskNotFound)?;
                task.vm.resume_from_select(index, value, false)?;
                task.status = TaskStateStatus::Ready;
                Self::unregister_channel_waits(&mut scheduler, task_id);
                scheduler.unregister_timer(task_id);
                scheduler.ready.push_back(task_id);
                return Ok(true);
            }

            let value = pending_value.take().ok_or(RuntimeError::NativeError)?;
            let task = scheduler
                .tasks
                .get_mut(task_id)
                .and_then(Option::as_mut)
                .ok_or(RuntimeError::TaskNotFound)?;
            task.vm.resume_from_channel(value)?;
            task.status = TaskStateStatus::Ready;
            scheduler.ready.push_back(task_id);
            return Ok(true);
        }
    }

    pub(crate) fn cancel(shared: &Rc<RefCell<Self>>, id: usize) -> Result<(), RuntimeError> {
        let mut scheduler = shared.borrow_mut();

        // Si la tâche est actuellement en cours d'exécution, on ne peut pas
        // modifier directement son TaskState : il est temporairement sorti
        // de `scheduler.tasks` pendant son quantum.
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
                // Modifier l'état de la tâche dans un bloc séparé afin que
                // l'emprunt mutable de `scheduler.tasks` soit terminé avant
                // toutes les opérations qui modifient le scheduler.
                {
                    let task = scheduler
                        .tasks
                        .get_mut(id)
                        .and_then(Option::as_mut)
                        .ok_or(RuntimeError::TaskNotFound)?;

                    task.status = TaskStateStatus::Cancelled;
                    task.result = None;
                    task.error = None;
                }

                // Une tâche annulée ne doit plus être sélectionnée.
                scheduler.ready.retain(|queued_id| *queued_id != id);

                // Supprimer toute attente enregistrée.
                Self::unregister_channel_waits(&mut scheduler, id);
                scheduler.unregister_mutex_wait(id);
                scheduler.unregister_wait_group_wait(id);
                scheduler.unregister_semaphore_wait(id);
                scheduler.unregister_timer(id);

                // Une annulation immédiate doit libérer toutes les ressources
                // de synchronisation détenues par la tâche.
                scheduler.release_owned_mutexes(id);
                scheduler.release_owned_semaphores(id);

                // Nettoyage de la VM après le nettoyage du scheduler.
                let task = scheduler
                    .tasks
                    .get_mut(id)
                    .and_then(Option::as_mut)
                    .ok_or(RuntimeError::TaskNotFound)?;

                task.vm.release_task_resources();
            }

            TaskStateStatus::Completed | TaskStateStatus::Failed | TaskStateStatus::Cancelled => {}
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
        Ok(matches!(
            Self::status(shared, id)?,
            "done" | "failed" | "cancelled"
        ))
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
            if let Some(semaphore) = &task.vm.waiting_semaphore {
                values.push(semaphore.clone());
            }
            if let Some(wait_group) = &task.vm.waiting_wait_group {
                values.push(wait_group.clone());
            }
            if let Some(channels) = &task.vm.waiting_select_channels {
                values.extend(channels.iter().cloned());
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
