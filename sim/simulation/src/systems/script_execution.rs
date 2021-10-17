use crate::{
    components::{game_config::GameConfig, CompiledScriptComponent, EntityScript, OwnedEntity},
    indices::{ConfigKey, EntityId, ScriptId, UserId},
    intents::*,
    prelude::World,
    profile,
    storage::views::{FromWorld, UnwrapView},
};
use cao_alloc::linear::LinearAllocator;
use cao_lang::prelude::*;
use rayon::prelude::*;
use std::{
    cell::RefCell,
    convert::Infallible,
    fmt::{self, Display, Formatter},
    rc::Rc,
};
use thiserror::Error;
use tracing::{debug, trace, warn};

pub type ExecutionResult = Result<BotIntents, ExecutionError>;

#[derive(Debug, Error, Clone)]
pub enum ExecutionError {
    #[error("{0:?} was not found")]
    ScriptNotFound(ScriptId),
    #[error(" {script_id:?} of {entity_id:?} failed {error:?}")]
    RuntimeError {
        script_id: ScriptId,
        entity_id: EntityId,
        error: cao_lang::prelude::ExecutionError,
    },
}

pub fn execute_scripts(
    workload: &[(EntityId, EntityScript)],
    storage: &World,
) -> Result<Vec<BotIntents>, Infallible> {
    profile!("execute_scripts");

    let owners_table = storage.view::<EntityId, OwnedEntity>().reborrow();

    let n_scripts = workload.len();

    let chunk_size = n_scripts.clamp(8, 256);

    debug!(
        "Executing {} scripts in chunks of {}",
        n_scripts, chunk_size
    );

    #[derive(Default)]
    struct RunResult {
        intents: Vec<BotIntents>,
        num_scripts_ran: u64,
        num_scripts_errored: u64,
    }

    let run_result = workload
        .par_iter()
        .chunks(chunk_size)
        .map(|entity_scripts| {
            let mut results = RunResult {
                intents: Vec::with_capacity(chunk_size),
                num_scripts_ran: 0,
                num_scripts_errored: 0,
            };
            let data = ScriptExecutionData::new(
                storage,
                Default::default(),
                EntityId::default(),
                None,
                get_alloc(),
            );

            let conf = UnwrapView::<ConfigKey, GameConfig>::from_world(storage);
            let mut vm = Vm::new(data).expect("Failed to initialize VM");
            vm.runtime_data.set_memory_limit(40 * 1024 * 1024);
            vm.max_instr = conf.execution_limit as u64;
            crate::scripting_api::make_import().execute_imports(&mut vm);

            for (entity_id, script) in entity_scripts {
                let owner_id = owners_table
                    .get(*entity_id)
                    .map(|OwnedEntity { owner_id }| *owner_id);

                let s = tracing::error_span!(
                    "script_execution",
                    entity_id = entity_id.to_string().as_str()
                );
                let _e = s.enter();

                vm.clear();
                match execute_single_script(*entity_id, script.0, owner_id, storage, &mut vm) {
                    Ok(ints) => results.intents.push(ints),
                    Err(err) => {
                        results.num_scripts_errored += 1;
                        debug!(
                            "Execution failure in {:?} of {:?}:\n{:?}",
                            script, entity_id, err
                        );
                    }
                }
                results.num_scripts_ran += 1;
            }
            results
        })
        .reduce(RunResult::default, |mut res, intermediate| {
            res.intents.extend(intermediate.intents);
            res.num_scripts_ran += intermediate.num_scripts_ran;
            res.num_scripts_errored += intermediate.num_scripts_errored;
            res
        });

    debug!(
        "Executing scripts done. Returning {:?} intents",
        run_result.intents.len()
    );

    Ok(run_result.intents)
}

pub(crate) fn get_alloc() -> Rc<RefCell<LinearAllocator>> {
    Rc::new(RefCell::new(LinearAllocator::new(100_000_000)))
}

pub fn execute_single_script<'a>(
    entity_id: EntityId,
    script_id: ScriptId,
    user_id: Option<UserId>,
    storage: &'a World,
    vm: &mut Vm<'a, ScriptExecutionData>,
) -> ExecutionResult {
    let program = storage
        .view::<ScriptId, CompiledScriptComponent>()
        .reborrow()
        .get(script_id)
        .ok_or_else(|| {
            warn!("Script by ID {:?} does not exist", script_id);
            ExecutionError::ScriptNotFound(script_id)
        })?;

    vm.auxiliary_data.reset(entity_id, user_id);
    trace!("Starting script execution");

    vm.run(&program.0).map_err(|err| {
        warn!("Error while executing script {:?} {:?}", script_id, err);
        ExecutionError::RuntimeError {
            script_id,
            entity_id,
            error: err,
        }
    })?;

    let intents = std::mem::take(&mut vm.auxiliary_data.intents);
    trace!("Script execution completed, intents:{:?}", intents);
    Ok(intents)
}

pub struct ScriptExecutionData {
    pub entity_id: EntityId,
    pub user_id: Option<UserId>,
    pub intents: BotIntents,
    pub alloc: Rc<RefCell<LinearAllocator>>,
    storage: *const World,
}

impl Display for ScriptExecutionData {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self.entity_id)?;
        if let Some(ref user_id) = self.user_id {
            write!(f, " UserId: {}", user_id.0)?
        }
        Ok(())
    }
}

impl ScriptExecutionData {
    pub fn reset(&mut self, entity_id: EntityId, user_id: Option<UserId>) {
        self.intents.entity_id = entity_id;
        self.entity_id = entity_id;
        self.user_id = user_id;
    }

    pub fn new(
        storage: &World,
        intents: BotIntents,
        entity_id: EntityId,
        user_id: Option<UserId>,
        alloc: Rc<RefCell<LinearAllocator>>,
    ) -> Self {
        Self {
            storage: storage as *const _,
            intents,
            entity_id,
            user_id,
            alloc,
        }
    }

    pub fn storage(&self) -> &World {
        unsafe { &*self.storage }
    }
}
