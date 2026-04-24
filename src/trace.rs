//! Execution trace support.
//!
//! Trace output is intentionally JSON but does not depend on serde. Keeping the
//! format small and explicit makes it suitable for later replay and diff tools.

use crate::vm::TrapCode;
use std::fmt::Write;

/// One VM instruction execution event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceEvent {
    pub cycle: u64,
    pub pc: usize,
    pub opcode: String,
    pub arg1: i32,
    pub arg2: i32,
    pub stack_before: Vec<i64>,
    pub stack_after: Vec<i64>,
    pub frame_depth_before: usize,
    pub frame_depth_after: usize,
    pub next_pc: usize,
    pub outcome: TraceOutcome,
}

/// Result of executing one traced instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceOutcome {
    Continue,
    Jump,
    Exit,
    Trap { code: TrapCode, message: String },
}

/// Serialize trace events as a stable JSON object.
pub fn events_to_json(backend: &str, events: &[TraceEvent]) -> String {
    let mut out = String::new();
    out.push_str("{\n  \"backend\": ");
    push_json_string(&mut out, backend);
    out.push_str(",\n  \"events\": [");

    for (index, event) in events.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str("\n    ");
        push_event_json(&mut out, event);
    }

    out.push_str("\n  ]\n}");
    out
}

fn push_event_json(out: &mut String, event: &TraceEvent) {
    out.push('{');
    write!(out, "\"cycle\":{}", event.cycle).expect("write to string cannot fail");
    write!(out, ",\"pc\":{}", event.pc).expect("write to string cannot fail");
    out.push_str(",\"opcode\":");
    push_json_string(out, &event.opcode);
    write!(out, ",\"arg1\":{}", event.arg1).expect("write to string cannot fail");
    write!(out, ",\"arg2\":{}", event.arg2).expect("write to string cannot fail");
    out.push_str(",\"stack_before\":");
    push_i64_array(out, &event.stack_before);
    out.push_str(",\"stack_after\":");
    push_i64_array(out, &event.stack_after);
    write!(out, ",\"frame_depth_before\":{}", event.frame_depth_before)
        .expect("write to string cannot fail");
    write!(out, ",\"frame_depth_after\":{}", event.frame_depth_after)
        .expect("write to string cannot fail");
    write!(out, ",\"next_pc\":{}", event.next_pc).expect("write to string cannot fail");
    out.push_str(",\"outcome\":");
    push_outcome_json(out, &event.outcome);
    out.push('}');
}

fn push_outcome_json(out: &mut String, outcome: &TraceOutcome) {
    match outcome {
        TraceOutcome::Continue => out.push_str("{\"kind\":\"continue\"}"),
        TraceOutcome::Jump => out.push_str("{\"kind\":\"jump\"}"),
        TraceOutcome::Exit => out.push_str("{\"kind\":\"exit\"}"),
        TraceOutcome::Trap { code, message } => {
            out.push_str("{\"kind\":\"trap\",\"code\":");
            write!(out, "{}", *code as u8).expect("write to string cannot fail");
            out.push_str(",\"trap\":");
            push_json_string(out, &format!("{:?}", code));
            out.push_str(",\"message\":");
            push_json_string(out, message);
            out.push('}');
        }
    }
}

fn push_i64_array(out: &mut String, values: &[i64]) {
    out.push('[');
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        write!(out, "{}", value).expect("write to string cannot fail");
    }
    out.push(']');
}

fn push_json_string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch <= '\u{1f}' => {
                write!(out, "\\u{:04x}", ch as u32).expect("write to string cannot fail");
            }
            ch => out.push(ch),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_json_strings() {
        let event = TraceEvent {
            cycle: 1,
            pc: 0,
            opcode: "LoadConst\"x".to_string(),
            arg1: 42,
            arg2: 0,
            stack_before: vec![],
            stack_after: vec![42],
            frame_depth_before: 1,
            frame_depth_after: 1,
            next_pc: 1,
            outcome: TraceOutcome::Continue,
        };

        let json = events_to_json("VM", &[event]);
        assert!(json.contains("LoadConst\\\"x"));
    }
}
