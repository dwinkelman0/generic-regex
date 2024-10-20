use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
};

pub trait TerminalMatcher: Debug {
    type Terminal: Debug;
    fn matches(&self, terminal: &Self::Terminal) -> bool;
}

#[derive(Debug)]
pub enum CoreExpr<T: TerminalMatcher> {
    Terminal(T),
    Sequence(Vec<CoreExpr<T>>),
    Choice(Vec<CoreExpr<T>>),
    Repeat(Box<CoreExpr<T>>),
    Null,
}

impl<T: TerminalMatcher> CoreExpr<T> {
    pub fn compile(&self) -> Matcher<T> {
        Matcher::new(self)
    }
}

pub trait ExprExtension<'a, T: TerminalMatcher> {
    fn into_core_expr(&self) -> CoreExpr<T>;
}

#[derive(Debug)]
struct TransitionFunc<'a, T: TerminalMatcher> {
    terminals: BTreeMap<usize, &'a T>,
    epsilons: BTreeSet<usize>,
}

impl<'a, T: TerminalMatcher> TransitionFunc<'a, T> {
    fn get_terminal_transitions(&self, terminal: &T::Terminal) -> BTreeSet<usize> {
        self.terminals
            .iter()
            .filter_map(|(state, matcher)| matcher.matches(terminal).then_some(*state))
            .collect()
    }
}

#[derive(Debug)]
pub struct Matcher<'a, T: TerminalMatcher> {
    transition_funcs: Vec<TransitionFunc<'a, T>>,
    start_state: usize,
    end_state: usize,
}

impl<'a, T: TerminalMatcher> Matcher<'a, T> {
    pub fn new(expr: &'a CoreExpr<T>) -> Self {
        let mut matcher = Matcher {
            transition_funcs: Vec::new(),
            start_state: 0,
            end_state: 1,
        };
        let start_state = matcher.create_new_state();
        let end_state = matcher.create_new_state();
        assert_eq!(start_state, matcher.start_state);
        assert_eq!(end_state, matcher.end_state);
        matcher.expand(expr, start_state, end_state);
        matcher
    }

    fn create_new_state(&mut self) -> usize {
        let state = self.transition_funcs.len();
        self.transition_funcs.push(TransitionFunc {
            terminals: BTreeMap::new(),
            epsilons: BTreeSet::new(),
        });
        state
    }

    fn expand(&mut self, expr: &'a CoreExpr<T>, start_state: usize, end_state: usize) {
        match expr {
            CoreExpr::Terminal(matcher) => {
                self.transition_funcs[start_state].terminals.insert(end_state, matcher);
            }
            CoreExpr::Sequence(exprs) => {
                let mut prev_state = start_state;
                for expr in exprs {
                    let next_state = self.create_new_state();
                    self.expand(expr, prev_state, next_state);
                    prev_state = next_state;
                }
                self.transition_funcs[prev_state].epsilons.insert(end_state);
            }
            CoreExpr::Choice(exprs) => {
                for expr in exprs {
                    self.expand(expr, start_state, end_state);
                }
            }
            CoreExpr::Repeat(expr) => {
                self.transition_funcs[start_state].epsilons.insert(end_state);
                self.transition_funcs[end_state].epsilons.insert(start_state);
                self.expand(expr, start_state, end_state);
            }
            CoreExpr::Null => {
                self.transition_funcs[start_state].epsilons.insert(end_state);
            }
        }
    }

    pub fn match_sequence(&self, string: &[T::Terminal]) -> bool {
        println!("Matching sequence: {:?}", string);
        let extend_epsilons = |states: &mut BTreeSet<usize>| {
            // Expand epsilon transitions until no more states are added
            // TODO: resolve epsilon expansion when the matcher is constructed
            loop {
                let mut new_states = BTreeSet::new();
                for state in states.iter() {
                    new_states.extend(self.transition_funcs[*state].epsilons.iter());
                }
                if new_states.is_subset(states) {
                    break;
                }
                states.extend(new_states);
            }
        };

        let mut current_states = BTreeSet::new();
        current_states.insert(0);
        for terminal in string {
            if current_states.is_empty() {
                return false;
            }
            let mut next_states = BTreeSet::new();
            for state in current_states {
                let transition_func = &self.transition_funcs[state];
                next_states.extend(transition_func.get_terminal_transitions(terminal));
            }
            current_states = next_states;
            extend_epsilons(&mut current_states);
            println!("Current states: {:?}", current_states);
        }
        current_states.contains(&self.end_state)
    }
}
