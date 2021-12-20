use parking_lot::Mutex;
use std::sync::Arc;

use anyhow::Result;

use super::*;

type LockMap<K, V> = Arc<Mutex<FxHashMap<K, V>>>;

#[derive(Clone)]
pub struct DslState<Btn = (sdl2::controller::Button, bool)> {
    // builder: Arc<Mutex<AutomataBuilder>>,
    states: LockMap<String, StateId>,
    inputs: LockMap<String, InputId>,
    outputs: LockMap<String, OutputId>,

    state_transitions: LockMap<StateId, LockMap<InputId, (StateId, Option<OutputId>)>>,

    input_map: LockMap<Btn, InputId>,
}

impl DslState {
    pub fn bind_ast(&self, ast: &rhai::AST) -> Result<()> {
        let mut engine = self.create_engine();

        let mut scope = rhai::Scope::new();

        let module = rhai::Module::eval_ast_as_new(scope, ast, &engine)?;

        let mut input_map = self.input_map.lock();

        println!("{:?}", ast);

        for (var_name, var) in module.iter_var() {
            let var = var.clone();
            if let Some(input_id) = var.try_cast::<InputId>() {
                println!("iter vars: {}, {:?}", var_name, input_id);

                use sdl2::controller::Button;

                match var_name {
                    "RIGHT_SHOULDER_DOWN" => {
                        dbg!();
                        input_map.insert((Button::RightShoulder, true), input_id);
                    }
                    "RIGHT_SHOULDER_UP" => {
                        dbg!();
                        input_map.insert((Button::RightShoulder, false), input_id);
                    }
                    "A_DOWN" => {
                        dbg!();
                        input_map.insert((Button::A, true), input_id);
                    }
                    "A_UP" => {
                        dbg!();
                        input_map.insert((Button::A, false), input_id);
                    }
                    _ => (),
                }
            }
        }

        Ok(())
    }
}

impl<T> std::default::Default for DslState<T> {
    fn default() -> Self {
        Self {
            states: Arc::new(Mutex::new(FxHashMap::default())),
            inputs: Arc::new(Mutex::new(FxHashMap::default())),
            outputs: Arc::new(Mutex::new(FxHashMap::default())),
            state_transitions: Arc::new(Mutex::new(FxHashMap::default())),
            input_map: Arc::new(Mutex::new(FxHashMap::default())),
        }
    }
}

impl<T: Copy + std::hash::Hash + Eq> DslState<T> {
    pub fn build(&self) -> Automata<T> {
        let mut builder = self.clone();

        let state_names = builder.states.lock();
        let state_names = state_names
            .iter()
            .map(|(s, i)| (s.to_string(), *i))
            .collect::<FxHashMap<_, _>>();

        let input_names = builder.inputs.lock();
        let input_names = input_names
            .iter()
            .map(|(s, i)| (s.to_string(), *i))
            .collect::<FxHashMap<_, _>>();

        let output_names = builder.outputs.lock();
        let output_names = output_names
            .iter()
            .map(|(s, i)| (s.to_string(), *i))
            .collect::<FxHashMap<_, _>>();

        let state_transitions = {
            let state_ts_ = builder.state_transitions.lock();
            let state_ts = state_ts_
                .iter()
                .map(|(state_id, ts)| {
                    let ts = ts.lock();
                    let ts = ts
                        .iter()
                        .map(|(input, &(tgt, out))| (*input, (tgt, out)))
                        .collect::<FxHashMap<_, _>>();

                    (*state_id, ts)
                })
                .collect::<FxHashMap<_, _>>();

            state_ts
        };

        let mut state_map: FxHashMap<StateId, usize> = FxHashMap::default();
        let mut states: Vec<State> = Vec::new();

        for (state_id, state_ts) in state_transitions.iter() {
            let ix = states.len();
            state_map.insert(*state_id, ix);
            states.push(State { transitions: FxHashMap::default() });
        }

        for (state_id, state_ts) in state_transitions.iter() {
            // let ix = states.len();

            let ts: &FxHashMap<InputId, (StateId, Option<OutputId>)> = state_ts;

            let state = states.get_mut(*state_map.get(state_id).unwrap()).unwrap();
            // let transitions = FxHashMap::default();

            let transitions = &mut state.transitions;

            for (input, (tgt, out)) in ts {

                let tgt_ix =state_map.get(tgt).unwrap();
                transitions.insert(*input, (*tgt_ix, *out));
            }

            // states.push(State {transitions: ts.clone()});

            // state_map.insert(*state_id, ix);
        }

        let input_map = self.input_map.lock().clone();

        Automata {
            active: 0,
            states,
            input_map,

            state_names,
            input_names,
            output_names,
        }
    }

    pub fn transition(&self, src: StateId, input: InputId, tgt: StateId, out: Option<OutputId>) {
        let mut states = self.state_transitions.lock();
        let mut transitions = states.entry(src).or_default().lock();
        transitions.insert(input, (tgt, out));
    }

    pub fn debug_print(&self) {
        let states = self.states.lock();
        let inputs = self.inputs.lock();
        let outputs = self.outputs.lock();

        println!("States:");
        for (name, id) in states.iter() {
            println!(" {} -> {:?}", name, id);
        }

        println!("Inputs:");
        for (name, id) in inputs.iter() {
            println!(" {} -> {:?}", name, id);
        }

        println!("Outputs:");
        for (name, id) in outputs.iter() {
            println!(" {} -> {:?}", name, id);
        }
    }

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

        let cloned = self.clone();

        engine.on_var(move |name, index, ctx| {
            println!("on_var called with {}", name);
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

        let cloned = self.clone();

        engine.register_fn("emit", move |src, input, tgt, output| {
            let src: StateId = rhai::Dynamic::cast(src);
            let input: InputId = rhai::Dynamic::cast(input);
            let tgt: StateId = rhai::Dynamic::cast(tgt);
            let output: OutputId = rhai::Dynamic::cast(output);

            cloned.transition(src, input, tgt, Some(output));
        });

        let cloned = self.clone();

        engine.register_fn("silent", move |src, input, tgt| {
            let src: StateId = rhai::Dynamic::cast(src);
            let input: InputId = rhai::Dynamic::cast(input);
            let tgt: StateId = rhai::Dynamic::cast(tgt);
            cloned.transition(src, input, tgt, None);
        });

        engine
    }
}
