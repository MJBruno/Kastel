//! Expressions régulières natives de `std.regex`.
//!
//! Ce moteur est entièrement implémenté dans Kastel/Rust et ne dépend d'aucune
//! crate externe. Il fournit un sous-ensemble volontairement explicite des
//! regex classiques, avec une exécution bornée pour éviter une consommation
//! incontrôlée du CPU ou de la mémoire.

use crate::{error::runtime_error::RuntimeError, runtime::value::Value};
use std::collections::HashMap;

const MAX_PATTERN_BYTES: usize = 1024 * 1024;
const MAX_TEXT_BYTES: usize = 64 * 1024 * 1024;
const MAX_MATCHES: usize = 1_000_000;
const MAX_PATTERN_NODES: usize = 100_000;
const MAX_PARSER_DEPTH: usize = 256;
const MAX_QUANTIFIER: usize = 1_000_000;

fn module_error(message: impl Into<String>) -> RuntimeError {
    RuntimeError::ModuleError(format!("regex: {}", message.into()))
}

fn expect_string(value: &Value) -> Result<String, RuntimeError> {
    value.as_string_value().ok_or(RuntimeError::TypeError)
}

fn expect_args(args: &[Value], expected: usize) -> Result<(), RuntimeError> {
    if args.len() != expected {
        return Err(RuntimeError::WrongArgumentCount {
            expected,
            found: args.len(),
        });
    }
    Ok(())
}

fn result_error(message: impl Into<String>) -> Value {
    Value::new_err(Value::new_string(format!("regex: {}", message.into())))
}

#[derive(Clone, Debug)]
enum PredefClass {
    Digit,
    NotDigit,
    Word,
    NotWord,
    Space,
    NotSpace,
}

impl PredefClass {
    fn matches(&self, ch: char) -> bool {
        match self {
            Self::Digit => ch.is_ascii_digit(),
            Self::NotDigit => !ch.is_ascii_digit(),
            Self::Word => ch.is_ascii_alphanumeric() || ch == '_',
            Self::NotWord => !(ch.is_ascii_alphanumeric() || ch == '_'),
            Self::Space => ch.is_whitespace(),
            Self::NotSpace => !ch.is_whitespace(),
        }
    }
}

#[derive(Clone, Debug)]
enum ClassItem {
    Char(char),
    Range(char, char),
    Predef(PredefClass),
}

impl ClassItem {
    fn matches(&self, ch: char) -> bool {
        match self {
            Self::Char(value) => *value == ch,
            Self::Range(start, end) => *start <= ch && ch <= *end,
            Self::Predef(class) => class.matches(ch),
        }
    }
}

#[derive(Clone, Debug)]
enum Node {
    Empty,
    Literal(char),
    Any,
    Class {
        negated: bool,
        items: Vec<ClassItem>,
    },
    AnchorStart,
    AnchorEnd,
    WordBoundary(bool),
    Sequence(Vec<Node>),
    Alternation(Vec<Node>),
    Group {
        index: usize,
        inner: Box<Node>,
    },
    Repeat {
        node: Box<Node>,
        min: usize,
        max: usize,
        greedy: bool,
    },
}

#[derive(Clone, Debug)]
struct Pattern {
    root: Node,
    capture_count: usize,
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
    capture_count: usize,
    nodes: usize,
    depth: usize,
}

impl Parser {
    fn new(pattern: &str) -> Self {
        Self {
            chars: pattern.chars().collect(),
            pos: 0,
            capture_count: 0,
            nodes: 0,
            depth: 0,
        }
    }

    fn parse(mut self) -> Result<Pattern, String> {
        let root = self.parse_expression(false)?;
        if self.peek() == Some(')') {
            return Err("unmatched ')'".into());
        }
        if self.pos != self.chars.len() {
            return Err("unexpected token".into());
        }
        Ok(Pattern {
            root,
            capture_count: self.capture_count,
        })
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn next(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += 1;
        Some(ch)
    }

    fn add_node(&mut self) -> Result<(), String> {
        self.nodes += 1;
        if self.nodes > MAX_PATTERN_NODES {
            return Err("pattern is too complex".into());
        }
        Ok(())
    }

    fn parse_expression(&mut self, inside_group: bool) -> Result<Node, String> {
        let mut alternatives = Vec::new();
        loop {
            let sequence = self.parse_sequence(inside_group)?;
            alternatives.push(sequence);
            if self.peek() == Some('|') {
                self.next();
                continue;
            }
            break;
        }

        if alternatives.len() == 1 {
            Ok(alternatives.pop().unwrap())
        } else {
            self.add_node()?;
            Ok(Node::Alternation(alternatives))
        }
    }

    fn parse_sequence(&mut self, inside_group: bool) -> Result<Node, String> {
        let mut nodes = Vec::new();
        while let Some(ch) = self.peek() {
            if ch == '|' || (inside_group && ch == ')') {
                break;
            }
            nodes.push(self.parse_quantified_atom()?);
        }
        if nodes.is_empty() {
            self.add_node()?;
            Ok(Node::Empty)
        } else if nodes.len() == 1 {
            Ok(nodes.pop().unwrap())
        } else {
            self.add_node()?;
            Ok(Node::Sequence(nodes))
        }
    }

    fn parse_quantified_atom(&mut self) -> Result<Node, String> {
        let atom = self.parse_atom()?;
        match self.peek() {
            Some('*') => {
                self.next();
                let greedy = !self.consume_lazy_suffix();
                self.add_node()?;
                Ok(Node::Repeat {
                    node: Box::new(atom),
                    min: 0,
                    max: usize::MAX,
                    greedy,
                })
            }
            Some('+') => {
                self.next();
                let greedy = !self.consume_lazy_suffix();
                self.add_node()?;
                Ok(Node::Repeat {
                    node: Box::new(atom),
                    min: 1,
                    max: usize::MAX,
                    greedy,
                })
            }
            Some('?') => {
                self.next();
                let greedy = !self.consume_lazy_suffix();
                self.add_node()?;
                Ok(Node::Repeat {
                    node: Box::new(atom),
                    min: 0,
                    max: 1,
                    greedy,
                })
            }
            Some('{') => {
                let (min, max) = self.parse_braced_quantifier()?;
                let greedy = !self.consume_lazy_suffix();
                self.add_node()?;
                Ok(Node::Repeat {
                    node: Box::new(atom),
                    min,
                    max,
                    greedy,
                })
            }
            _ => Ok(atom),
        }
    }

    fn consume_lazy_suffix(&mut self) -> bool {
        if self.peek() == Some('?') {
            self.next();
            true
        } else {
            false
        }
    }

    fn parse_braced_quantifier(&mut self) -> Result<(usize, usize), String> {
        debug_assert_eq!(self.peek(), Some('{'));
        self.next();

        let min = self.parse_decimal("expected repetition count")?;
        if min > MAX_QUANTIFIER {
            return Err("repetition count is too large".into());
        }

        match self.peek() {
            Some('}') => {
                self.next();
                Ok((min, min))
            }
            Some(',') => {
                self.next();
                if self.peek() == Some('}') {
                    self.next();
                    Ok((min, usize::MAX))
                } else {
                    let max = self.parse_decimal("expected upper repetition count")?;
                    if max > MAX_QUANTIFIER || max < min {
                        return Err("invalid repetition range".into());
                    }
                    if self.next() != Some('}') {
                        return Err("expected '}'".into());
                    }
                    Ok((min, max))
                }
            }
            _ => Err("expected ',' or '}' in repetition".into()),
        }
    }

    fn parse_decimal(&mut self, message: &str) -> Result<usize, String> {
        let mut value = 0usize;
        let mut found = false;
        while let Some(ch) = self.peek() {
            if !ch.is_ascii_digit() {
                break;
            }
            found = true;
            self.next();
            let digit = (ch as u8 - b'0') as usize;
            value = value
                .checked_mul(10)
                .and_then(|value| value.checked_add(digit))
                .ok_or_else(|| "repetition count is too large".to_string())?;
            if value > MAX_QUANTIFIER {
                return Err("repetition count is too large".into());
            }
        }
        if found {
            Ok(value)
        } else {
            Err(message.into())
        }
    }

    fn parse_atom(&mut self) -> Result<Node, String> {
        let ch = self.next().ok_or_else(|| "expected pattern atom".to_string())?;
        match ch {
            '.' => {
                self.add_node()?;
                Ok(Node::Any)
            }
            '^' => {
                self.add_node()?;
                Ok(Node::AnchorStart)
            }
            '$' => {
                self.add_node()?;
                Ok(Node::AnchorEnd)
            }
            '(' => self.parse_group(),
            '[' => self.parse_class(),
            '\\' => {
                let escape = self.parse_escape(false)?;
                self.add_node()?;
                Ok(escape)
            }
            '*' | '+' | '?' => Err(format!("quantifier '{ch}' has no preceding atom")),
            '{' => Err("'{...' is only valid as a quantifier after an atom".into()),
            ')' => Err("unmatched ')'".into()),
            '|' => Err("unexpected '|'".into()),
            other => {
                self.add_node()?;
                Ok(Node::Literal(other))
            }
        }
    }

    fn parse_group(&mut self) -> Result<Node, String> {
        if self.peek() == Some('?') {
            return Err("non-capturing groups and lookarounds are not supported".into());
        }
        self.depth += 1;
        if self.depth > MAX_PARSER_DEPTH {
            return Err("pattern nesting is too deep".into());
        }

        self.capture_count += 1;
        let index = self.capture_count;
        let inner = self.parse_expression(true)?;
        if self.next() != Some(')') {
            return Err("unclosed group".into());
        }
        self.depth -= 1;
        self.add_node()?;
        Ok(Node::Group {
            index,
            inner: Box::new(inner),
        })
    }

    fn parse_class(&mut self) -> Result<Node, String> {
        let mut negated = false;
        if self.peek() == Some('^') {
            self.next();
            negated = true;
        }

        let mut items: Vec<ClassItem> = Vec::new();
        let mut first_item = true;
        loop {
            match self.peek() {
                None => return Err("unclosed character class".into()),
                Some(']') if !first_item => {
                    self.next();
                    break;
                }
                _ => {}
            }

            let first = self.parse_class_token()?;
            first_item = false;

            if self.peek() == Some('-') {
                let save = self.pos;
                self.next();
                if self.peek() == Some(']') || self.peek().is_none() {
                    self.pos = save;
                    items.push(first.into_item());
                    continue;
                }
                let second = self.parse_class_token()?;
                match (first, second) {
                    (ClassToken::Char(start), ClassToken::Char(end)) if start <= end => {
                        items.push(ClassItem::Range(start, end));
                    }
                    _ => return Err("invalid character range".into()),
                }
            } else {
                items.push(first.into_item());
            }
        }

        if items.is_empty() {
            return Err("empty character class".into());
        }
        self.add_node()?;
        Ok(Node::Class { negated, items })
    }

    fn parse_class_token(&mut self) -> Result<ClassToken, String> {
        let ch = self.next().ok_or_else(|| "unclosed character class".to_string())?;
        if ch != '\\' {
            return Ok(ClassToken::Char(ch));
        }
        self.parse_class_escape()
    }

    fn parse_class_escape(&mut self) -> Result<ClassToken, String> {
        let ch = self.next().ok_or_else(|| "dangling escape".to_string())?;
        match ch {
            'd' => Ok(ClassToken::Predef(PredefClass::Digit)),
            'D' => Ok(ClassToken::Predef(PredefClass::NotDigit)),
            'w' => Ok(ClassToken::Predef(PredefClass::Word)),
            'W' => Ok(ClassToken::Predef(PredefClass::NotWord)),
            's' => Ok(ClassToken::Predef(PredefClass::Space)),
            'S' => Ok(ClassToken::Predef(PredefClass::NotSpace)),
            'n' => Ok(ClassToken::Char('\n')),
            'r' => Ok(ClassToken::Char('\r')),
            't' => Ok(ClassToken::Char('\t')),
            'f' => Ok(ClassToken::Char('\x0c')),
            'v' => Ok(ClassToken::Char('\x0b')),
            '0' => Ok(ClassToken::Char('\0')),
            'b' => Ok(ClassToken::Char('\x08')),
            other => Ok(ClassToken::Char(other)),
        }
    }

    fn parse_escape(&mut self, _inside_class: bool) -> Result<Node, String> {
        let ch = self.next().ok_or_else(|| "dangling escape".to_string())?;
        match ch {
            'd' => Ok(Node::Class {
                negated: false,
                items: vec![ClassItem::Predef(PredefClass::Digit)],
            }),
            'D' => Ok(Node::Class {
                negated: false,
                items: vec![ClassItem::Predef(PredefClass::NotDigit)],
            }),
            'w' => Ok(Node::Class {
                negated: false,
                items: vec![ClassItem::Predef(PredefClass::Word)],
            }),
            'W' => Ok(Node::Class {
                negated: false,
                items: vec![ClassItem::Predef(PredefClass::NotWord)],
            }),
            's' => Ok(Node::Class {
                negated: false,
                items: vec![ClassItem::Predef(PredefClass::Space)],
            }),
            'S' => Ok(Node::Class {
                negated: false,
                items: vec![ClassItem::Predef(PredefClass::NotSpace)],
            }),
            'n' => Ok(Node::Literal('\n')),
            'r' => Ok(Node::Literal('\r')),
            't' => Ok(Node::Literal('\t')),
            'f' => Ok(Node::Literal('\x0c')),
            'v' => Ok(Node::Literal('\x0b')),
            '0' => Ok(Node::Literal('\0')),
            'b' => Ok(Node::WordBoundary(false)),
            'B' => Ok(Node::WordBoundary(true)),
            'A' => Ok(Node::AnchorStart),
            'z' => Ok(Node::AnchorEnd),
            'p' | 'P' => Err("Unicode property classes are not supported".into()),
            other => Ok(Node::Literal(other)),
        }
    }
}

enum ClassToken {
    Char(char),
    Predef(PredefClass),
}

impl ClassToken {
    fn into_item(self) -> ClassItem {
        match self {
            Self::Char(ch) => ClassItem::Char(ch),
            Self::Predef(class) => ClassItem::Predef(class),
        }
    }
}

#[derive(Clone, Debug)]
struct State {
    pos: usize,
    captures: Vec<Option<(usize, usize)>>,
}

#[derive(Clone, Debug)]
struct MatchResult {
    start: usize,
    end: usize,
    captures: Vec<Option<(usize, usize)>>,
}

struct Matcher<'a> {
    text: &'a [char],
    steps: usize,
    max_steps: usize,
}

impl<'a> Matcher<'a> {
    fn new(text: &'a [char], capture_count: usize) -> Self {
        let max_steps = (10_000_000usize)
            .saturating_add(text.len().saturating_mul(8))
            .min(100_000_000);
        let _ = capture_count;
        Self {
            text,
            steps: 0,
            max_steps,
        }
    }

    fn tick(&mut self) -> Result<(), RuntimeError> {
        self.steps = self.steps.saturating_add(1);
        if self.steps > self.max_steps {
            return Err(module_error("match exceeded the execution limit"));
        }
        Ok(())
    }

    fn is_word(ch: Option<char>) -> bool {
        ch.is_some_and(|value| value.is_ascii_alphanumeric() || value == '_')
    }

    fn match_nodes(
        &mut self,
        nodes: &[Node],
        index: usize,
        state: State,
    ) -> Result<Vec<State>, RuntimeError> {
        self.tick()?;
        if index >= nodes.len() {
            return Ok(vec![state]);
        }

        if let Node::Repeat {
            node,
            min,
            max,
            greedy,
        } = &nodes[index]
        {
            return self.match_repeat_with_rest(
                node,
                *min,
                *max,
                *greedy,
                nodes,
                index + 1,
                state,
            );
        }

        let mut results = Vec::new();
        for next in self.match_node(&nodes[index], state)? {
            results.extend(self.match_nodes(nodes, index + 1, next)?);
            if results.len() >= MAX_MATCHES {
                break;
            }
        }
        Ok(results)
    }

    fn match_node(&mut self, node: &Node, state: State) -> Result<Vec<State>, RuntimeError> {
        self.tick()?;
        match node {
            Node::Empty => Ok(vec![state]),
            Node::Literal(expected) => {
                if self.text.get(state.pos) == Some(expected) {
                    Ok(vec![State {
                        pos: state.pos + 1,
                        ..state
                    }])
                } else {
                    Ok(Vec::new())
                }
            }
            Node::Any => {
                if state.pos < self.text.len() {
                    Ok(vec![State {
                        pos: state.pos + 1,
                        ..state
                    }])
                } else {
                    Ok(Vec::new())
                }
            }
            Node::Class { negated, items } => {
                let Some(ch) = self.text.get(state.pos).copied() else {
                    return Ok(Vec::new());
                };
                let matched = items.iter().any(|item| item.matches(ch));
                if matched != *negated {
                    Ok(vec![State {
                        pos: state.pos + 1,
                        ..state
                    }])
                } else {
                    Ok(Vec::new())
                }
            }
            Node::AnchorStart => {
                if state.pos == 0 {
                    Ok(vec![state])
                } else {
                    Ok(Vec::new())
                }
            }
            Node::AnchorEnd => {
                if state.pos == self.text.len() {
                    Ok(vec![state])
                } else {
                    Ok(Vec::new())
                }
            }
            Node::WordBoundary(negated) => {
                let before = Self::is_word(self.text.get(state.pos.wrapping_sub(1)).copied());
                let after = Self::is_word(self.text.get(state.pos).copied());
                let boundary = before != after;
                if boundary != *negated {
                    Ok(vec![state])
                } else {
                    Ok(Vec::new())
                }
            }
            Node::Sequence(nodes) => self.match_nodes(nodes, 0, state),
            Node::Alternation(branches) => {
                let mut results = Vec::new();
                for branch in branches {
                    results.extend(self.match_node(branch, state.clone())?);
                    if results.len() >= MAX_MATCHES {
                        break;
                    }
                }
                Ok(results)
            }
            Node::Group { index, inner } => {
                let start = state.pos;
                let mut results = Vec::new();
                for mut next in self.match_node(inner, state.clone())? {
                    if let Some(slot) = next.captures.get_mut(*index - 1) {
                        *slot = Some((start, next.pos));
                    }
                    results.push(next);
                    if results.len() >= MAX_MATCHES {
                        break;
                    }
                }
                Ok(results)
            }
            Node::Repeat { node, min, max, greedy } => {
                self.repeat_candidates(node, *min, *max, *greedy, state)
            }
        }
    }

    fn effective_max(&self, max: usize) -> usize {
        if max == usize::MAX {
            self.text.len().saturating_add(1)
        } else {
            max
        }
    }

    fn repeat_candidates(
        &mut self,
        node: &Node,
        min: usize,
        max: usize,
        greedy: bool,
        state: State,
    ) -> Result<Vec<State>, RuntimeError> {
        let max = self.effective_max(max);
        let mut results = Vec::new();
        let mut stack = vec![(0usize, state)];

        while let Some((count, current)) = stack.pop() {
            self.tick()?;
            if results.len() >= MAX_MATCHES {
                break;
            }

            if greedy {
                if count >= min {
                    results.push(current.clone());
                }
                if count < max {
                    let mut next_states = self.match_node(node, current.clone())?;
                    next_states.retain(|next| next.pos != current.pos);
                    for next in next_states.into_iter().rev() {
                        stack.push((count + 1, next));
                    }
                }
            } else {
                if count < max {
                    let mut next_states = self.match_node(node, current.clone())?;
                    next_states.retain(|next| next.pos != current.pos);
                    for next in next_states.into_iter().rev() {
                        stack.push((count + 1, next));
                    }
                }
                if count >= min {
                    results.push(current.clone());
                }
            }
        }

        // A standalone greedy repeat (for example `[0-9]+`) is consumed by
        // `match_node` directly through `repeat_candidates`. The candidate
        // states are generated from the shallowest repetition to the deepest
        // one, so return them in reverse order for true greedy semantics.
        if greedy {
            results.reverse();
        }
        Ok(results)
    }

    fn match_repeat_with_rest(
        &mut self,
        node: &Node,
        min: usize,
        max: usize,
        greedy: bool,
        nodes: &[Node],
        rest_index: usize,
        state: State,
    ) -> Result<Vec<State>, RuntimeError> {
        let max = self.effective_max(max);

        // Greedy matching must try deeper repetitions before trying the rest
        // of the sequence. A simple LIFO stack with immediate tail matching
        // would otherwise make `+` behave like a one-character repetition.
        // `try_tail` is a deferred continuation marker.
        if greedy {
            let mut stack = vec![(0usize, state, false)];

            while let Some((count, current, try_tail)) = stack.pop() {
                self.tick()?;

                if try_tail {
                    if count >= min {
                        let tails = self.match_nodes(nodes, rest_index, current)?;
                        if !tails.is_empty() {
                            return Ok(tails);
                        }
                    }
                    continue;
                }

                if count >= min {
                    // Defer this tail until every deeper repetition candidate
                    // reachable from `current` has been explored.
                    stack.push((count, current.clone(), true));
                }

                if count < max {
                    let mut next_states = self.match_node(node, current.clone())?;
                    next_states.retain(|next| next.pos != current.pos);
                    for next in next_states.into_iter().rev() {
                        stack.push((count + 1, next, false));
                    }
                }
            }

            return Ok(Vec::new());
        }

        let mut stack = vec![(0usize, state)];
        while let Some((count, current)) = stack.pop() {
            self.tick()?;

            if count >= min {
                let tails = self.match_nodes(nodes, rest_index, current.clone())?;
                if !tails.is_empty() {
                    return Ok(tails);
                }
            }
            if count < max {
                let mut next_states = self.match_node(node, current.clone())?;
                next_states.retain(|next| next.pos != current.pos);
                for next in next_states.into_iter().rev() {
                    stack.push((count + 1, next));
                }
            }
        }
        Ok(Vec::new())
    }
}

fn compile(pattern: &str) -> Result<Pattern, String> {
    if pattern.len() > MAX_PATTERN_BYTES {
        return Err("pattern is too large".into());
    }
    Parser::new(pattern).parse()
}

fn validate_text(text: &str) -> Result<(), RuntimeError> {
    if text.len() > MAX_TEXT_BYTES {
        return Err(module_error("input text is too large"));
    }
    Ok(())
}

fn char_slice(text: &[char], start: usize, end: usize) -> String {
    text[start..end].iter().collect()
}

fn run_first_from(
    pattern: &Pattern,
    text: &[char],
    start_at: usize,
) -> Result<Option<MatchResult>, RuntimeError> {
    let mut matcher = Matcher::new(text, pattern.capture_count);
    for start in start_at..=text.len() {
        let state = State {
            pos: start,
            captures: vec![None; pattern.capture_count],
        };
        let states = matcher.match_node(&pattern.root, state)?;
        if let Some(state) = states.into_iter().next() {
            return Ok(Some(MatchResult {
                start,
                end: state.pos,
                captures: state.captures,
            }));
        }
    }
    Ok(None)
}

fn find_all_matches(pattern: &Pattern, text: &str) -> Result<Vec<MatchResult>, RuntimeError> {
    let chars = text.chars().collect::<Vec<_>>();
    let mut result = Vec::new();
    let mut search_start = 0usize;

    while search_start <= chars.len() {
        let Some(matched) = run_first_from(pattern, &chars, search_start)? else {
            break;
        };
        if result.len() >= MAX_MATCHES {
            return Err(module_error("too many matches"));
        }

        let next_search = if matched.start == matched.end {
            matched.end.saturating_add(1)
        } else {
            matched.end
        };
        result.push(matched);
        if next_search > chars.len() {
            break;
        }
        search_start = next_search;
    }

    Ok(result)
}

fn match_record(text: &[char], matched: &MatchResult) -> Value {
    let groups = matched
        .captures
        .iter()
        .map(|capture| match capture {
            Some((start, end)) => Value::new_some(Value::new_string(char_slice(text, *start, *end))),
            None => Value::None,
        })
        .collect::<Vec<_>>();

    Value::new_record(vec![
        ("start".into(), Value::Integer(matched.start as i64)),
        ("end".into(), Value::Integer(matched.end as i64)),
        (
            "text".into(),
            Value::new_string(char_slice(text, matched.start, matched.end)),
        ),
        ("groups".into(), Value::new_array(groups)),
    ])
}

fn expand_replacement(
    replacement: &str,
    text: &[char],
    matched: &MatchResult,
) -> Result<String, String> {
    let chars = replacement.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut index = 0usize;

    while index < chars.len() {
        let ch = chars[index];
        if ch != '$' {
            output.push(ch);
            index += 1;
            continue;
        }

        index += 1;
        if index >= chars.len() {
            output.push('$');
            break;
        }

        if chars[index] == '$' {
            output.push('$');
            index += 1;
            continue;
        }

        if !chars[index].is_ascii_digit() {
            output.push('$');
            output.push(chars[index]);
            index += 1;
            continue;
        }

        let mut group = 0usize;
        while index < chars.len() && chars[index].is_ascii_digit() {
            group = group
                .checked_mul(10)
                .and_then(|value| value.checked_add((chars[index] as u8 - b'0') as usize))
                .ok_or_else(|| "replacement group number is too large".to_string())?;
            index += 1;
        }

        if group == 0 {
            output.push_str(&char_slice(text, matched.start, matched.end));
            continue;
        }

        let Some(capture) = matched.captures.get(group - 1) else {
            return Err(format!("replacement references missing capture group ${group}"));
        };
        if let Some((start, end)) = capture {
            output.push_str(&char_slice(text, *start, *end));
        }
    }

    Ok(output)
}

fn regex_escape_text(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\\' | '.' | '^' | '$' | '*' | '+' | '?' | '{' | '}' | '[' | ']' | '(' | ')' | '|' => {
                output.push('\\');
                output.push(ch);
            }
            _ => output.push(ch),
        }
    }
    output
}

pub fn native_regex_is_match(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 2)?;
    let pattern = expect_string(&args[0])?;
    let text = expect_string(&args[1])?;
    validate_text(&text)?;

    let pattern = match compile(&pattern) {
        Ok(pattern) => pattern,
        Err(error) => return Ok(result_error(error)),
    };
    let chars = text.chars().collect::<Vec<_>>();
    let mut matcher = Matcher::new(&chars, pattern.capture_count);
    for start in 0..=chars.len() {
        let state = State {
            pos: start,
            captures: vec![None; pattern.capture_count],
        };
        if !matcher.match_node(&pattern.root, state)?.is_empty() {
            return Ok(Value::new_ok(Value::Boolean(true)));
        }
    }
    Ok(Value::new_ok(Value::Boolean(false)))
}

pub fn native_regex_find(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 2)?;
    let pattern_text = expect_string(&args[0])?;
    let text = expect_string(&args[1])?;
    validate_text(&text)?;

    let pattern = match compile(&pattern_text) {
        Ok(pattern) => pattern,
        Err(error) => return Ok(result_error(error)),
    };
    let chars = text.chars().collect::<Vec<_>>();
    let result = run_first_from(&pattern, &chars, 0)?.map(|matched| match_record(&chars, &matched));
    Ok(Value::new_ok(Value::new_option(result)))
}

pub fn native_regex_find_all(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 2)?;
    let pattern_text = expect_string(&args[0])?;
    let text = expect_string(&args[1])?;
    validate_text(&text)?;

    let pattern = match compile(&pattern_text) {
        Ok(pattern) => pattern,
        Err(error) => return Ok(result_error(error)),
    };
    let chars = text.chars().collect::<Vec<_>>();
    let matches = match find_all_matches(&pattern, &text) {
        Ok(matches) => matches,
        Err(error) => return Ok(result_error(error.to_string())),
    };
    Ok(Value::new_ok(Value::new_array(
        matches
            .iter()
            .map(|matched| match_record(&chars, matched))
            .collect(),
    )))
}

pub fn native_regex_replace(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 3)?;
    let pattern_text = expect_string(&args[0])?;
    let text = expect_string(&args[1])?;
    let replacement = expect_string(&args[2])?;
    validate_text(&text)?;
    if replacement.len() > MAX_TEXT_BYTES {
        return Ok(result_error("replacement is too large"));
    }

    let pattern = match compile(&pattern_text) {
        Ok(pattern) => pattern,
        Err(error) => return Ok(result_error(error)),
    };
    let chars = text.chars().collect::<Vec<_>>();
    let matches = match find_all_matches(&pattern, &text) {
        Ok(matches) => matches,
        Err(error) => return Ok(result_error(error.to_string())),
    };

    let mut output = String::new();
    let mut cursor = 0usize;
    for matched in matches {
        output.push_str(&char_slice(&chars, cursor, matched.start));
        match expand_replacement(&replacement, &chars, &matched) {
            Ok(value) => output.push_str(&value),
            Err(error) => return Ok(result_error(error)),
        }
        cursor = matched.end;
        if output.len() > MAX_TEXT_BYTES {
            return Ok(result_error("replacement result is too large"));
        }
    }
    output.push_str(&char_slice(&chars, cursor, chars.len()));
    if output.len() > MAX_TEXT_BYTES {
        return Ok(result_error("replacement result is too large"));
    }

    Ok(Value::new_ok(Value::new_string(output)))
}

pub fn native_regex_split(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 2)?;
    let pattern_text = expect_string(&args[0])?;
    let text = expect_string(&args[1])?;
    validate_text(&text)?;

    let pattern = match compile(&pattern_text) {
        Ok(pattern) => pattern,
        Err(error) => return Ok(result_error(error)),
    };
    let chars = text.chars().collect::<Vec<_>>();
    let matches = match find_all_matches(&pattern, &text) {
        Ok(matches) => matches,
        Err(error) => return Ok(result_error(error.to_string())),
    };

    if matches.len() >= MAX_MATCHES {
        return Ok(result_error("too many split pieces"));
    }

    let mut pieces = Vec::with_capacity(matches.len() + 1);
    let mut cursor = 0usize;
    for matched in matches {
        pieces.push(Value::new_string(char_slice(&chars, cursor, matched.start)));
        cursor = matched.end;
    }
    pieces.push(Value::new_string(char_slice(&chars, cursor, chars.len())));
    Ok(Value::new_ok(Value::new_array(pieces)))
}

pub fn native_regex_escape(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 1)?;
    let text = expect_string(&args[0])?;
    Ok(Value::new_string(regex_escape_text(&text)))
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert("regex_is_match".into(), Value::NativeFunction(native_regex_is_match));
    globals.insert("regex_find".into(), Value::NativeFunction(native_regex_find));
    globals.insert("regex_find_all".into(), Value::NativeFunction(native_regex_find_all));
    globals.insert("regex_replace".into(), Value::NativeFunction(native_regex_replace));
    globals.insert("regex_split".into(), Value::NativeFunction(native_regex_split));
    globals.insert("regex_escape".into(), Value::NativeFunction(native_regex_escape));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::object::Object;

    fn s(value: &str) -> Value {
        Value::new_string(value.to_string())
    }

    fn ok_value(value: Value) -> Value {
        let Value::Object(handle) = value else { panic!("Result attendu"); };
        let object = handle.borrow();
        let Object::Result { ok: true, value } = &*object else { panic!("Ok attendu"); };
        value.clone()
    }

    fn is_err_result(value: Value) -> bool {
        let Value::Object(handle) = value else { return false; };
        matches!(&*handle.borrow(), Object::Result { ok: false, .. })
    }

    #[test]
    fn regex_matches_and_reports_unicode_character_offsets() {
        let result = native_regex_find(&[s("é."), s("Aéx")]).unwrap();
        let inner = ok_value(result);
        let Value::Object(option) = inner else { panic!("Option attendue"); };
        let Object::Option(Some(inner)) = &*option.borrow() else { panic!("Some attendu"); };
        let Value::Object(record) = inner else { panic!("Record attendu"); };
        let Object::Record(fields) = &*record.borrow() else { panic!("Record attendu"); };
        let start = fields.iter().find(|(name, _)| name == "start").unwrap().1.clone();
        let end = fields.iter().find(|(name, _)| name == "end").unwrap().1.clone();
        assert_eq!(start, Value::Integer(1));
        assert_eq!(end, Value::Integer(3));
    }

    #[test]
    fn regex_rejects_invalid_pattern_as_result_error() {
        let result = native_regex_is_match(&[s("["), s("abc")]).unwrap();
        assert!(is_err_result(result));
    }

    #[test]
    fn regex_replace_and_split_are_deterministic() {
        let replaced = native_regex_replace(&[s("[0-9]+"), s("a12b34"), s("X")]).unwrap();
        let value = ok_value(replaced);
        assert_eq!(value.as_string_value().unwrap(), "aXbX");

        let split = native_regex_split(&[s(",\\s*"), s("a, b,c")]).unwrap();
        let value = ok_value(split);
        let Value::Object(array) = value else { panic!("Array attendu"); };
        let Object::Array(items) = &*array.borrow() else { panic!("Array attendu"); };
        let actual = items.iter().map(|item| item.as_string_value().unwrap()).collect::<Vec<_>>();
        assert_eq!(actual, vec!["a", "b", "c"]);
    }

    #[test]
    fn regex_supports_quantifiers_classes_anchors_and_captures() {
        let value = ok_value(
            native_regex_find(&[s(r"^(ab)+([0-9]{2})$"), s("abab42")]).unwrap(),
        );
        let Value::Object(option) = value else { panic!(); };
        let Object::Option(Some(inner)) = &*option.borrow() else { panic!(); };
        let Value::Object(record) = inner else { panic!(); };
        let Object::Record(fields) = &*record.borrow() else { panic!(); };
        let groups = fields.iter().find(|(name, _)| name == "groups").unwrap().1.clone();
        let Value::Object(groups_handle) = groups else { panic!("groups attendu"); };
        let Object::Array(items) = &*groups_handle.borrow() else { panic!("groups attendu"); };
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].to_string(), "Some(ab)");
        assert_eq!(items[1].to_string(), "Some(42)");
    }

    #[test]
    fn regex_repetition_is_greedy_and_non_overlapping() {
        let value = ok_value(
            native_regex_find(&[s(r"[0-9]+"), s("a123b45")]).unwrap(),
        );
        let Value::Object(option) = value else { panic!(); };
        let Object::Option(Some(inner)) = &*option.borrow() else { panic!(); };
        let Value::Object(record) = inner else { panic!(); };
        let Object::Record(fields) = &*record.borrow() else { panic!(); };
        let matched = fields.iter().find(|(name, _)| name == "text").unwrap().1.clone();
        assert_eq!(matched.as_string_value().unwrap(), "123");

        let split = ok_value(
            native_regex_split(&[s(r"[0-9]+"), s("a123b45")]).unwrap(),
        );
        let Value::Object(array) = split else { panic!(); };
        let Object::Array(items) = &*array.borrow() else { panic!(); };
        let actual = items
            .iter()
            .map(|item| item.as_string_value().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(actual, vec!["a", "b", ""]);
    }

    #[test]
    fn regex_supports_replacement_captures() {
        let value = ok_value(
            native_regex_replace(&[s(r"([a-z]+)-([0-9]+)"), s("id-42"), s("$2:$1")]).unwrap(),
        );
        assert_eq!(value.as_string_value().unwrap(), "42:id");
    }

    #[test]
    fn regex_replacement_rejects_missing_group() {
        let value = native_regex_replace(&[s("a"), s("a"), s("$1")]).unwrap();
        assert!(is_err_result(value));
    }

    #[test]
    fn regex_escape_is_literal() {
        let value = native_regex_escape(&[s("a+b")]).unwrap();
        assert_eq!(value.as_string_value().unwrap(), "a\\+b");
    }
}
