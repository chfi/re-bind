use parking_lot::Mutex;
use std::sync::Arc;

use anyhow::Result;

use super::*;

#[derive(Clone, Default)]
pub struct DslState {
    // builder: Arc<Mutex<AutomataBuilder>>,

    // states: Arc<Mutex<FxHashMap<StateId, String>>>,
    // inputs: Arc<Mutex<FxHashMap<InputId, String>>>,
    // outputs: Arc<Mutex<FxHashMap<OutputId, String>>>,
    states: Arc<Mutex<FxHashMap<String, StateId>>>,
    inputs: Arc<Mutex<FxHashMap<String, InputId>>>,
    outputs: Arc<Mutex<FxHashMap<String, OutputId>>>,
}

impl DslState {
    pub fn get_state(&self, name: &str) -> StateId {
        let mut states = self.states.lock();

        if let Some(id) = states.get(name) {
            *id
        } else {
            let id = StateId(states.len());
            states.insert(name.to_string(), id);
            id
        }
    }

    pub fn get_input(&self, name: &str) -> InputId {
        let mut inputs = self.inputs.lock();

        if let Some(id) = inputs.get(name) {
            *id
        } else {
            let id = InputId(inputs.len());
            inputs.insert(name.to_string(), id);
            id
        }
    }

    pub fn get_output(&self, name: &str) -> OutputId {
        let mut outputs = self.outputs.lock();

        if let Some(id) = outputs.get(name) {
            *id
        } else {
            let id = OutputId(outputs.len());
            outputs.insert(name.to_string(), id);
            id
        }
    }
}

impl DslState {
    pub fn create_engine(&self) -> rhai::Engine {
        let mut engine = rhai::Engine::new();

        let cloned =self.clone();

        engine.on_var(move |name, index, ctx| {
            if name.starts_with("S_") {
                let state_id = cloned.get_state(name);
                Ok(Some(rhai::Dynamic::from(state_id)))
            } else if name.starts_with("In_") {
                let in_id = cloned.get_input(name);
                Ok(Some(rhai::Dynamic::from(in_id)))
            } else if name.starts_with("Out_") {
                let out_id = cloned.get_output(name);
                Ok(Some(rhai::Dynamic::from(out_id)))
            } else {
                Ok(None)
            }
        });

        engine
    }
}
