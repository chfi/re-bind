use futures::{future::RemoteHandle, task::SpawnExt, Future, FutureExt};

use crossbeam::atomic::AtomicCell;
use futures_timer::Delay;
use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::Arc;

use parking_lot::Mutex;

use anyhow::Result;

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateId(usize);

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutputId(usize);

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InputId(usize);

pub enum Output {
    Callback(Box<dyn Fn() + Send + Sync + 'static>),
}

#[derive(Default, Clone, Copy)]
pub struct TransitionDef {
    tgt: StateId,
    input: InputId,
    output: Option<OutputId>,
}

// pub enum Input

// pub struct InputDef {
//     id: InputId,
//     button: 
// }

#[derive(Clone)]
pub struct StateDef {
    id: StateId,
    transitions: Arc<Mutex<Vec<TransitionDef>>>,
}

impl StateDef {
    pub fn transition(&mut self, input: InputId, tgt: StateId, output: OutputId) {
        let mut transitions = self.transitions.lock();
        let def = TransitionDef {
            tgt,
            input,
            output: Some(output),
        };
        transitions.push(def);
    }

    pub fn silent(&mut self, input: InputId, tgt: StateId) {
        let mut transitions = self.transitions.lock();
        let def = TransitionDef {
            tgt,
            input,
            output: None,
        };
        transitions.push(def);
    }
}

#[derive(Clone)]
pub struct OutputDef {
    id: OutputId,
    output: Arc<Mutex<Option<Output>>>,
}

#[derive(Default, Clone)]
pub struct State {
    // transitions: Vec<(StateId, Option<Output>)>,
    transitions: Vec<(StateId, Option<OutputId>)>,
    // the outputId version may be faster, by having all the output
    // callbacks live on the same thread, and be called from the same place, signaled from here
}

impl State {
    pub fn from_def(def: StateDef) -> Self {
        let mut ts_lock = def.transitions.lock();
        let mut ts: Vec<_> = Vec::new();
        std::mem::swap(ts_lock.as_mut(), &mut ts);

        let transitions = ts.iter().map(|d| (d.tgt, d.output)).collect();

        Self { transitions }
    }
}

// this could be a more efficient representation, maybe -- or at least fast
pub struct LilAutomata {
    active: u8,
    states: [State; 256],
    outputs: [Option<OutputId>; 256],
}

pub struct Automata<Btn = sdl2::controller::Button> {
    active: StateId,
    states: Vec<State>,
    inputs: FxHashMap<Btn, InputId>,
    outputs: Vec<Output>,
    // outputs: Arc<Vec<Output>>,
}

// impl<Btn: std::hash::Hash + Eq> Automata<Btn> {
impl Automata {
    pub fn step(&mut self, input: sdl2::controller::Button) -> Option<OutputId> {
        let input = self.inputs.get(&input)?;
        let state = self.states.get(self.active.0)?;
        let (tgt, out) = state.transitions.get(input.0)?;
        let out = *out;
        self.active = *tgt;
        return out;
    }

    pub fn from_builder(builder: AutomataBuilder) -> Self {
        let active = StateId(0);

        let input_count = builder.inputs.len();
        let mut inputs: FxHashMap<_, InputId> = FxHashMap::default();
        let mut input_map: FxHashMap<InputId, usize> = FxHashMap::default();
        for (&input, def) in builder.inputs.iter() {
            input_map.insert(input, input_map.len());
            inputs.insert(def.button, input);
        }

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
        for (&id, state) in builder.states.iter() {
            state_map.insert(id, state_map.len());
        }

        let mut states = Vec::new();

        let mut state_ids = state_map.iter().map(|(a, b)| (*a, *b)).collect::<Vec<_>>();
        state_ids.sort_by_key(|(_, b)| *b);

        let mut count = 0;

        for (id, ix) in state_ids {
            assert!(count == ix);
            count += 1;
            let def = builder.states.get(&id).unwrap();
            let state = State::from_def(def.to_owned());
            states.push(state);
        }

        Automata {
            active,
            states,
            inputs,
            outputs,
        }
    }
}

pub struct InputDef<Btn = sdl2::controller::Button> {
    id: InputId,
    button: Btn,
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
            transitions: Arc::new(Mutex::new(Vec::new())),
        };
        self.states.insert(id, def.clone());
        def
    }

    pub fn new_input(&mut self, button: sdl2::controller::Button) -> InputId {
        let id = InputId(self.inputs.len());
        let def = InputDef {
            id,
            button,
        };
        self.inputs.insert(id, def);
        id
    }

    pub fn new_output(&mut self) -> OutputDef {
        let id = OutputId(self.states.len());
        let def = OutputDef {
            id,
            output: Arc::new(Mutex::new(None)),
        };
        self.outputs.insert(id, def.clone());
        def
    }
}

/*

pub struct Auto<const N: usize> {
    states: [[Option<usize>; N]; N],
    outputs: [[Option<Output>; N]; N],
}


// pub struct State {
//     transitions: Vec<Option<usize>>,
//     outputs: Vec<Option<Output>>,
// }

// pub struct StateId(pub usize);

// #[derive(Default)]
// pub struct Automaton {
//     states: Vec<State>,
// }

pub struct StateId(usize);
pub struct Transition { tgt: StateId, out: Option<Output> }


#[derive(Default)]
pub struct AutoBuilder {
    states: FxHashMap<usize, Vec<Transition>>,

    transitions: Vec<Transition>,
}

impl AutoBuilder {
    pub fn get(&self, id: StateId) -> Option<&Vec<Transition>> {
        self.states.get(&id.0)
    }

    pub fn transition(&mut self, src: StateId, tgt: StateId, out: Option<Output>) {
        let transition = Transition { tgt, out };
        self.states.entry(src.0).or_default().push(transition);
    }

    pub fn new_state(&mut self) -> StateId {
        let id =self.states.len();
        self.states.insert(id, Vec::new());
        StateId(id)
    }
}
*/
