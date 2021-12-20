use futures::{future::RemoteHandle, task::SpawnExt, Future, FutureExt};

use crossbeam::atomic::AtomicCell;
use futures_timer::Delay;
use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::Arc;

use parking_lot::Mutex;

pub mod dsl;


use anyhow::Result;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateId(usize);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutputId(pub usize);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InputId(usize);

pub enum Output {
    Callback(Box<dyn Fn() + Send + Sync + 'static>),
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TransitionDef {
    pub tgt: StateId,
    // pub input: InputId,
    pub output: Option<OutputId>,
}

#[derive(Clone)]
pub struct StateDef {
    pub id: StateId,
    transitions: Arc<Mutex<FxHashMap<InputId, TransitionDef>>>,
}

impl StateDef {
    pub fn transition(&self, input: InputId, tgt: StateId, output: OutputId) {
        println!("transition: {:?} ({:?}) -> {:?}\t{:?}", self.id, input, tgt, output);
        let mut transitions = self.transitions.lock();
        let def = TransitionDef {
            tgt,
            output: Some(output),
        };
        transitions.insert(input, def);
    }

    pub fn silent(&self, input: InputId, tgt: StateId) {
        println!("transition: {:?} ({:?}) -> {:?}\t(Silent)", self.id, input, tgt);
        let mut transitions = self.transitions.lock();
        let def = TransitionDef {
            tgt,
            output: None,
        };
        transitions.insert(input, def);
    }
}

#[derive(Clone)]
pub struct OutputDef {
    pub id: OutputId,
    output: Arc<Mutex<Option<Output>>>,
}

#[derive(Default, Clone)]
pub struct State {
    // transitions: Vec<(StateId, Option<Output>)>,
    // transitions: Vec<(usize, Option<OutputId>)>,
    transitions: FxHashMap<InputId, (usize, Option<OutputId>)>,
    // the outputId version may be faster, by having all the output
    // callbacks live on the same thread, and be called from the same place, signaled from here
}


/*
// this could be a more efficient representation, maybe -- or at least fast
pub struct LilAutomata {
    active: u8,
    states: [State; 256],
    outputs: [Option<OutputId>; 256],
}
*/

pub struct Automata<Btn = sdl2::controller::Button> {
    // active: StateId,
    active: usize,
    states: Vec<State>,
    inputs: FxHashMap<(Btn, bool), InputId>,
    outputs: Vec<Output>,
    // outputs: Arc<Vec<Output>>,
}

// impl<Btn: std::hash::Hash + Eq> Automata<Btn> {
impl Automata {

    pub fn map_input(&self, input: sdl2::controller::Button, down: bool) -> Option<InputId> {
        self.inputs.get(&(input, down)).copied()
    }

    pub fn step(&mut self, input: sdl2::controller::Button, down: bool) -> Option<OutputId> {
        let prev_state = self.active;

        println!("{:?}, {:?}, {}", prev_state, input, down);
        let input = self.inputs.get(&(input, down))?;

        let state = self.states.get(self.active)?;

        let (tgt, out) = state.transitions.get(input)?;
        let out = *out;

        self.active = *tgt;

        println!(" > {:?} -> {:?}\t{:?}, {}\t", prev_state, tgt, input, down);

        return out;
    }

    pub fn from_builder(builder: AutomataBuilder) -> Self {
        let input_count = builder.inputs.len();
        let mut inputs: FxHashMap<_, InputId> = FxHashMap::default();
        let mut input_map: FxHashMap<InputId, usize> = FxHashMap::default();
        for (&input, def) in builder.inputs.iter() {
            input_map.insert(input, input_map.len());
            inputs.insert((def.button, def.down), input);
            
            println!("{:?}", inputs.get(&(def.button, true)));
            println!("{:?}", inputs.get(&(def.button, false)));
        }

        let mut inputs_by_ix = input_map.iter().map(|(a, b)| (*a, *b)).collect::<Vec<_>>();
        inputs_by_ix.sort_by_key(|(_, a)| *a);
        let inputs_by_ix = inputs_by_ix.into_iter().map(|(id, _)| id).collect::<Vec<_>>();

        println!("input_map.len() {}", input_map.len());
        println!("inputs.len() {}", inputs.len());
        let mut outputs: Vec<Output> = Vec::new();

        let mut output_map: FxHashMap<OutputId, usize> = FxHashMap::default();
        for (&output_id, output) in builder.outputs.iter() {
            let mut out_l = output.output.lock();
            if let Some(out) = out_l.take() {
                let ix = outputs.len();
                let id = OutputId(ix);

                outputs.push(out);
                output_map.insert(id, ix);
            }
        }
        let output_count = outputs.len();

        let state_count = builder.states.len();
        let mut state_map: FxHashMap<StateId, usize> = FxHashMap::default();
        let mut counttt =0;
        for (&id, state) in builder.states.iter() {
            state_map.insert(id, state_map.len());
            counttt += 1;
        }
        println!("{}\t{}", counttt, state_map.len());

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

        Automata {
            active: 0,
            states,
            inputs,
            outputs,
        }
    }
}

pub struct InputDef<Btn = sdl2::controller::Button> {
    pub id: InputId,
    pub button: Btn,
    pub down: bool,
}

#[derive(Default)]
pub struct AutomataBuilder {
    states: FxHashMap<StateId, StateDef>,
    // inputs: FxHashSet<InputId>,
    inputs: FxHashMap<InputId, InputDef>,
    outputs: FxHashMap<OutputId, OutputDef>,
}

impl AutomataBuilder {
    pub fn new_state(&mut self) -> StateDef {
        let id = StateId(self.states.len());
        let def = StateDef {
            id,
            transitions: Arc::new(Mutex::new(FxHashMap::default())),
        };
        self.states.insert(id, def.clone());
        def
    }

    pub fn new_input(&mut self, button: sdl2::controller::Button) -> (InputId, InputId) {

        let id_down = InputId(self.inputs.len());
        let def = InputDef {
            id: id_down,
            button,
            down: true,
        };
        self.inputs.insert(id_down, def);

        let id_up = InputId(self.inputs.len());
        let def = InputDef {
            id: id_up,
            button,
            down: false,
        };
        self.inputs.insert(id_up, def);

        println!("{:?} down -> {:?}", button, id_down);
        println!("{:?} up -> {:?}", button, id_up);

        (id_down, id_up)
    }

    pub fn new_output(&mut self) -> OutputDef {
        let id = OutputId(self.outputs.len());
        let def = OutputDef {
            id,
            output: Arc::new(Mutex::new(None)),
        };
        self.outputs.insert(id, def.clone());
        def
    }
}
