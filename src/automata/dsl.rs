use parking_lot::Mutex;
use std::sync::Arc;

use anyhow::Result;

use super::*;

type LockMap<K, V> = Arc<Mutex<FxHashMap<K, V>>>;

#[derive(Clone, Default)]
pub struct DslState {
    // builder: Arc<Mutex<AutomataBuilder>>,
    states: LockMap<String, StateId>,
    inputs: LockMap<String, InputId>,
    outputs: LockMap<String, OutputId>,

    state_transitions: LockMap<StateId, LockMap<InputId, (StateId, Option<OutputId>)>>,
}

impl DslState {
    pub fn build<T: Copy + std::hash::Hash + Eq>(&self) -> Automata<T> {
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

        let output_names = builder.inputs.lock();
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
                    let ts = ts.iter().map(|(input, &(tgt, out))| {
                        (*input, (tgt, out))
                    }).collect::<FxHashMap<_,_>>();

                    (*state_id, ts)
                })
                .collect::<FxHashMap<_, _>>();

            state_ts
        };

        let mut state_map: FxHashMap<StateId, usize> = FxHashMap::default();
        let mut states: Vec<State> = Vec::new();

        for (state_id, state_ts) in state_transitions.iter() {

            let ix = states.len();
            
            let ts: &FxHashMap<InputId, (StateId, Option<OutputId>)> = state_ts;


            state_map.insert(*state_id, ix);

        }

        let active = 0;
        let inputs = unimplemented!();
        let outputs = unimplemented!();

        // let states =

        /*



        let input_count = inputs.len();
        let mut inputs: FxHashMap<_, InputId> = FxHashMap::default();
        let mut input_map: FxHashMap<InputId, usize> = FxHashMap::default();
        for (&input, def) in inputs.iter() {
            input_map.insert(input, input_map.len());
            inputs.insert((def.button, def.down), input);

            println!("{:?}", inputs.get(&(def.button, true)));
            println!("{:?}", inputs.get(&(def.button, false)));
        }
        */

        /*
        let mut inputs_by_ix = input_map.iter().map(|(a, b)| (*a, *b)).collect::<Vec<_>>();
        inputs_by_ix.sort_by_key(|(_, a)| *a);
        let inputs_by_ix = inputs_by_ix.into_iter().map(|(id, _)| id).collect::<Vec<_>>();

        println!("input_map.len() {}", input_map.len());
        println!("inputs.len() {}", inputs.len());
        let mut outputs: Vec<Output> = Vec::new();
        */

        /*
        let mut output_map: FxHashMap<OutputId, usize> = FxHashMap::default();
        for (&output_id, output) in outputs.iter() {
            /
            let mut out_l = output.output.lock();
            if let Some(out) = out_l.take() {
                let ix = outputs.len();
                let id = OutputId(ix);

                outputs.push(out);
                output_map.insert(id, ix);
            }
        }
        */
        /*
        let output_count = outputs.len();

        let state_count = builder.states.len();
        let mut state_map: FxHashMap<StateId, usize> = FxHashMap::default();
        let mut counttt =0;
        for (&id, state) in builder.states.iter() {
            state_map.insert(id, state_map.len());
            counttt += 1;
        }
        println!("{}\t{}", counttt, state_map.len());
        */

        /*
        let mut states = Vec::new();

        let mut state_ids = state_map.iter().map(|(a, b)| (*a, *b)).collect::<Vec<_>>();
        state_ids.sort_by_key(|(_, b)| *b);

        // println!("what the fuuuuck");
        println!("{:?}", state_ids);

        let mut count = 0;


        for (state_id, ix) in state_ids {
            assert!(count == ix);
            count += 1;
            let def = builder.states.get(&state_id).unwrap();

            let ts = def.transitions.lock();

            let transitions = ts.iter().map(|(k, v)| {

                let tgt = state_map.get(&v.tgt).unwrap();

                let out = v.output;

                (*k, (*tgt, out))
            }).collect();


            let state = State { transitions };

            states.push(state);
        }

        */

        Automata {
            active: 0,
            states,
            inputs,
            outputs,
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
