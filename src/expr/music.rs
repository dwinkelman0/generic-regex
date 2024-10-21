use super::core::{CoreExpr, ExprExtension, TerminalMatcher};

#[derive(Debug, Clone)]
pub enum NoteEvent {
    Interval(i8),
    Rest,
    Last,
}

#[derive(Debug, Clone)]
pub struct Note {
    event: NoteEvent,
    duration: u8, // use MIDI ticks
}

#[derive(Debug, Clone)]
pub enum IntervalAmount {
    Unison,
    MinorSecond,
    MajorSecond,
    MinorThird,
    MajorThird,
    PerfectFourth,
    Tritone,
    PerfectFifth,
    MinorSixth,
    MajorSixth,
    MinorSeventh,
    MajorSeventh,
    Octave,
}

impl IntervalAmount {
    fn as_half_steps(&self) -> i8 {
        match self {
            IntervalAmount::Unison => 0,
            IntervalAmount::MinorSecond => 1,
            IntervalAmount::MajorSecond => 2,
            IntervalAmount::MinorThird => 3,
            IntervalAmount::MajorThird => 4,
            IntervalAmount::PerfectFourth => 5,
            IntervalAmount::Tritone => 6,
            IntervalAmount::PerfectFifth => 7,
            IntervalAmount::MinorSixth => 8,
            IntervalAmount::MajorSixth => 9,
            IntervalAmount::MinorSeventh => 10,
            IntervalAmount::MajorSeventh => 11,
            IntervalAmount::Octave => 12,
        }
    }
}

#[derive(Debug, Clone)]
pub enum IntervalRule {
    Any,
    Up(IntervalAmount),
    Down(IntervalAmount),
    Rest,
    Last,
}

#[derive(Debug, Clone)]
pub enum Duration {
    Whole,
    Half,
    Third,
    Quarter,
    Sixth,
    Eighth,
    Twelfth,
    Sixteenth,
    TwentyFourth,
    ThirtySecond,
    FortyEighth,
    SixtyFourth,
}

impl Duration {
    fn as_ticks(&self) -> u8 {
        // Quarter note is 48
        match self {
            Duration::Whole => 192,
            Duration::Half => 96,
            Duration::Third => 64,
            Duration::Quarter => 48,
            Duration::Sixth => 32,
            Duration::Eighth => 24,
            Duration::Twelfth => 16,
            Duration::Sixteenth => 12,
            Duration::TwentyFourth => 8,
            Duration::ThirtySecond => 6,
            Duration::FortyEighth => 4,
            Duration::SixtyFourth => 3,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DurationRule {
    Any,
    Exact(Duration),
    MultipleOf(Duration),
    DoublingOf(Duration),
    ExactPlusMultipleOf(Duration, Duration),
}

#[derive(Debug, Clone)]
pub struct NoteRule {
    interval: Vec<IntervalRule>,
    duration: Vec<DurationRule>,
}

#[derive(Debug, Clone)]
pub struct NoteMatcher {
    rule: NoteRule,
}

impl TerminalMatcher for NoteMatcher {
    type Terminal = Note;

    fn matches(&self, terminal: &Self::Terminal) -> bool {
        self.rule.interval.iter().any(|interval_rule| match interval_rule {
            IntervalRule::Any => true,
            IntervalRule::Up(interval) => match terminal.event {
                NoteEvent::Interval(interval_amount) => interval.as_half_steps() == interval_amount,
                _ => false,
            },
            IntervalRule::Down(interval) => match terminal.event {
                NoteEvent::Interval(interval_amount) => -interval.as_half_steps() == interval_amount,
                _ => false,
            },
            IntervalRule::Rest => matches!(terminal.event, NoteEvent::Rest),
            IntervalRule::Last => matches!(terminal.event, NoteEvent::Last),
        }) && self.rule.duration.iter().any(|duration_rule| match duration_rule {
            DurationRule::Any => true,
            DurationRule::Exact(duration) => terminal.duration == duration.as_ticks(),
            DurationRule::MultipleOf(duration) => terminal.duration % duration.as_ticks() == 0,
            DurationRule::DoublingOf(duration) => terminal.duration % (duration.as_ticks() * 2) == 0,
            DurationRule::ExactPlusMultipleOf(duration, multiple) => {
                terminal.duration == duration.as_ticks() || terminal.duration % multiple.as_ticks() == 0
            }
        })
    }
}

pub enum NoteExpr {
    Note(NoteRule),
    Sequence(Vec<NoteExpr>),
    Choice(Vec<NoteExpr>),
    OneOrMore(Box<NoteExpr>),
    ZeroOrOne(Box<NoteExpr>),
    Repeat(Box<NoteExpr>),
    Null,
}

impl ExprExtension<'_, NoteMatcher> for NoteExpr {
    fn into_core_expr(&self) -> CoreExpr<NoteMatcher> {
        match self {
            NoteExpr::Note(rule) => CoreExpr::Terminal(NoteMatcher { rule: rule.clone() }),
            NoteExpr::Sequence(exprs) => CoreExpr::Sequence(exprs.iter().map(|expr| expr.into_core_expr()).collect()),
            NoteExpr::Choice(exprs) => CoreExpr::Choice(exprs.iter().map(|expr| expr.into_core_expr()).collect()),
            NoteExpr::OneOrMore(expr) => CoreExpr::OneOrMore(Box::new(expr.into_core_expr())),
            NoteExpr::ZeroOrOne(expr) => CoreExpr::ZeroOrOne(Box::new(expr.into_core_expr())),
            NoteExpr::Repeat(expr) => CoreExpr::Repeat(Box::new(expr.into_core_expr())),
            NoteExpr::Null => CoreExpr::Null,
        }
    }
}

impl std::ops::Add for NoteExpr {
    type Output = NoteExpr;

    fn add(self, other: NoteExpr) -> NoteExpr {
        NoteExpr::Sequence(vec![self, other])
    }
}

impl std::ops::BitOr for NoteExpr {
    type Output = NoteExpr;

    fn bitor(self, other: NoteExpr) -> NoteExpr {
        NoteExpr::Choice(vec![self, other])
    }
}
