//! Runtime audit reports built on top of execution traces.
//!
//! These helpers are deliberately independent of the CLI so fuzzers, tests, and
//! CI artifact tooling can ask the same questions as a human operator.

use crate::compiler::CompiledProgram;
use crate::gc_vm::{GcVm, GcVmResult};
use crate::trace::{
    first_semantic_trace_divergence, first_trace_divergence, push_json_string,
    push_trace_summary_json, stable_fingerprint_bytes, summarize_trace, TraceDivergence,
    TraceSummary,
};
use crate::vm::{TrapCode, Vm, VmResult};
use std::fmt::{self, Write};

/// Minimal observable result used when comparing traced runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionSummary {
    pub success: bool,
    pub return_value: i64,
    pub trap_code: TrapCode,
    pub output: Vec<String>,
}

/// Deterministic replay check for the reference VM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceReplayReport {
    pub replayable: bool,
    pub event_count: usize,
    pub original_trace_summary: TraceSummary,
    pub replay_trace_summary: TraceSummary,
    pub original_outcome: ExecutionSummary,
    pub replay_outcome: ExecutionSummary,
    pub divergence: Option<TraceDivergence>,
    pub outcome_mismatch: Option<String>,
}

/// Instruction trace diff between the standard VM and GC VM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendTraceDiffReport {
    pub equivalent: bool,
    pub left_backend: String,
    pub right_backend: String,
    pub left_events: usize,
    pub right_events: usize,
    pub left_trace_summary: TraceSummary,
    pub right_trace_summary: TraceSummary,
    pub left_outcome: ExecutionSummary,
    pub right_outcome: ExecutionSummary,
    pub divergence: Option<TraceDivergence>,
    pub outcome_mismatch: Option<String>,
}

/// Run the reference VM twice and verify that its instruction trace replays.
pub fn replay_vm_trace(program: &CompiledProgram) -> TraceReplayReport {
    let mut original_vm = Vm::new(program).with_trace();
    let original_result = original_vm.run();
    let original_trace = original_vm.trace_events().to_vec();

    let mut replay_vm = Vm::new(program).with_trace();
    let replay_result = replay_vm.run();
    let replay_trace = replay_vm.trace_events();

    let original_trace_summary = summarize_trace(&original_trace);
    let replay_trace_summary = summarize_trace(replay_trace);
    let original_outcome = ExecutionSummary::from_vm(original_result);
    let replay_outcome = ExecutionSummary::from_vm(replay_result);
    let outcome_mismatch = outcome_mismatch(&original_outcome, &replay_outcome);
    let divergence = first_trace_divergence(&original_trace, replay_trace);

    TraceReplayReport {
        replayable: outcome_mismatch.is_none() && divergence.is_none(),
        event_count: original_trace_summary.event_count,
        original_trace_summary,
        replay_trace_summary,
        original_outcome,
        replay_outcome,
        divergence,
        outcome_mismatch,
    }
}

/// Compare the standard VM and GC VM on the same bytecode at trace granularity.
pub fn diff_vm_gc_traces(program: &CompiledProgram) -> BackendTraceDiffReport {
    let mut vm = Vm::new(program).with_trace();
    let vm_result = vm.run();
    let vm_trace = vm.trace_events().to_vec();

    let mut gc_vm = GcVm::new(program).with_trace();
    let gc_result = gc_vm.run();
    let gc_trace = gc_vm.trace_events();

    let left_trace_summary = summarize_trace(&vm_trace);
    let right_trace_summary = summarize_trace(gc_trace);
    let left_outcome = ExecutionSummary::from_vm(vm_result);
    let right_outcome = ExecutionSummary::from_gc_vm(gc_result);
    let outcome_mismatch = outcome_mismatch(&left_outcome, &right_outcome);
    let divergence = first_semantic_trace_divergence(&vm_trace, gc_trace);

    BackendTraceDiffReport {
        equivalent: outcome_mismatch.is_none() && divergence.is_none(),
        left_backend: "VM".to_string(),
        right_backend: "GC VM".to_string(),
        left_events: left_trace_summary.event_count,
        right_events: right_trace_summary.event_count,
        left_trace_summary,
        right_trace_summary,
        left_outcome,
        right_outcome,
        divergence,
        outcome_mismatch,
    }
}

impl ExecutionSummary {
    fn from_vm(result: VmResult) -> Self {
        Self {
            success: result.success,
            return_value: result.return_value,
            trap_code: result.trap_code,
            output: result.output,
        }
    }

    fn from_gc_vm(result: GcVmResult) -> Self {
        Self {
            success: result.success,
            return_value: result.return_value,
            trap_code: result.trap_code,
            output: result.output,
        }
    }
}

fn outcome_mismatch(left: &ExecutionSummary, right: &ExecutionSummary) -> Option<String> {
    if left == right {
        None
    } else {
        Some(format!(
            "{} vs {}",
            format_outcome(left),
            format_outcome(right)
        ))
    }
}

fn format_outcome(outcome: &ExecutionSummary) -> String {
    format!(
        "success={}, return={}, trap={:?}, output={:?}",
        outcome.success, outcome.return_value, outcome.trap_code, outcome.output
    )
}

impl TraceReplayReport {
    /// Deterministic fingerprint of the machine-readable report payload.
    pub fn fingerprint(&self) -> u64 {
        stable_fingerprint_bytes(trace_replay_evidence_json(self, false).as_bytes())
    }

    /// Return the report fingerprint as fixed-width lowercase hexadecimal.
    pub fn fingerprint_hex(&self) -> String {
        format!("{:016x}", self.fingerprint())
    }

    /// Serialize the audit evidence as a stable JSON object.
    pub fn evidence_json(&self) -> String {
        trace_replay_evidence_json(self, true)
    }
}

impl BackendTraceDiffReport {
    /// Deterministic fingerprint of the machine-readable report payload.
    pub fn fingerprint(&self) -> u64 {
        stable_fingerprint_bytes(backend_trace_diff_evidence_json(self, false).as_bytes())
    }

    /// Return the report fingerprint as fixed-width lowercase hexadecimal.
    pub fn fingerprint_hex(&self) -> String {
        format!("{:016x}", self.fingerprint())
    }

    /// Serialize the audit evidence as a stable JSON object.
    pub fn evidence_json(&self) -> String {
        backend_trace_diff_evidence_json(self, true)
    }
}

fn trace_replay_evidence_json(report: &TraceReplayReport, include_fingerprint: bool) -> String {
    let mut out = String::new();
    out.push('{');
    out.push_str("\"kind\":\"trace_replay\"");
    out.push_str(",\"schema_version\":1");
    if include_fingerprint {
        out.push_str(",\"fingerprint\":");
        push_json_string(&mut out, &report.fingerprint_hex());
    }
    out.push_str(",\"replayable\":");
    push_bool(&mut out, report.replayable);
    write!(out, ",\"event_count\":{}", report.event_count).expect("write to string cannot fail");
    out.push_str(",\"original_trace\":");
    push_trace_summary_json(&mut out, &report.original_trace_summary);
    out.push_str(",\"replay_trace\":");
    push_trace_summary_json(&mut out, &report.replay_trace_summary);
    out.push_str(",\"original_outcome\":");
    push_execution_summary_json(&mut out, &report.original_outcome);
    out.push_str(",\"replay_outcome\":");
    push_execution_summary_json(&mut out, &report.replay_outcome);
    out.push_str(",\"outcome_mismatch\":");
    push_optional_string(&mut out, report.outcome_mismatch.as_deref());
    out.push_str(",\"divergence\":");
    push_divergence_json(&mut out, report.divergence.as_ref());
    out.push('}');
    out
}

fn backend_trace_diff_evidence_json(
    report: &BackendTraceDiffReport,
    include_fingerprint: bool,
) -> String {
    let mut out = String::new();
    out.push('{');
    out.push_str("\"kind\":\"backend_trace_diff\"");
    out.push_str(",\"schema_version\":1");
    if include_fingerprint {
        out.push_str(",\"fingerprint\":");
        push_json_string(&mut out, &report.fingerprint_hex());
    }
    out.push_str(",\"equivalent\":");
    push_bool(&mut out, report.equivalent);
    out.push_str(",\"left_backend\":");
    push_json_string(&mut out, &report.left_backend);
    out.push_str(",\"right_backend\":");
    push_json_string(&mut out, &report.right_backend);
    write!(out, ",\"left_events\":{}", report.left_events).expect("write to string cannot fail");
    write!(out, ",\"right_events\":{}", report.right_events).expect("write to string cannot fail");
    out.push_str(",\"left_trace\":");
    push_trace_summary_json(&mut out, &report.left_trace_summary);
    out.push_str(",\"right_trace\":");
    push_trace_summary_json(&mut out, &report.right_trace_summary);
    out.push_str(",\"left_outcome\":");
    push_execution_summary_json(&mut out, &report.left_outcome);
    out.push_str(",\"right_outcome\":");
    push_execution_summary_json(&mut out, &report.right_outcome);
    out.push_str(",\"outcome_mismatch\":");
    push_optional_string(&mut out, report.outcome_mismatch.as_deref());
    out.push_str(",\"divergence\":");
    push_divergence_json(&mut out, report.divergence.as_ref());
    out.push('}');
    out
}

fn push_execution_summary_json(out: &mut String, summary: &ExecutionSummary) {
    out.push('{');
    out.push_str("\"success\":");
    push_bool(out, summary.success);
    write!(out, ",\"return_value\":{}", summary.return_value).expect("write to string cannot fail");
    write!(out, ",\"trap_code\":{}", summary.trap_code as u8).expect("write to string cannot fail");
    out.push_str(",\"trap\":");
    push_json_string(out, &format!("{:?}", summary.trap_code));
    out.push_str(",\"output\":");
    push_string_array(out, &summary.output);
    out.push('}');
}

fn push_divergence_json(out: &mut String, divergence: Option<&TraceDivergence>) {
    let Some(divergence) = divergence else {
        out.push_str("null");
        return;
    };

    out.push('{');
    write!(out, "\"event_index\":{}", divergence.event_index).expect("write to string cannot fail");
    out.push_str(",\"field\":");
    push_json_string(out, &divergence.field);
    out.push_str(",\"left\":");
    push_json_string(out, &divergence.left);
    out.push_str(",\"right\":");
    push_json_string(out, &divergence.right);
    out.push('}');
}

fn push_string_array(out: &mut String, values: &[String]) {
    out.push('[');
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        push_json_string(out, value);
    }
    out.push(']');
}

fn push_optional_string(out: &mut String, value: Option<&str>) {
    match value {
        Some(value) => push_json_string(out, value),
        None => out.push_str("null"),
    }
}

fn push_bool(out: &mut String, value: bool) {
    out.push_str(if value { "true" } else { "false" });
}

impl fmt::Display for TraceReplayReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Trace Replay")?;
        writeln!(
            f,
            "  status: {}",
            if self.replayable {
                "replayable"
            } else {
                "diverged"
            }
        )?;
        writeln!(f, "  events: {}", self.event_count)?;
        writeln!(f, "  original: {}", format_outcome(&self.original_outcome))?;
        writeln!(f, "  replay:   {}", format_outcome(&self.replay_outcome))?;

        if let Some(mismatch) = &self.outcome_mismatch {
            writeln!(f)?;
            writeln!(f, "Outcome Mismatch")?;
            writeln!(f, "  {}", mismatch)?;
        }

        if let Some(divergence) = &self.divergence {
            writeln!(f)?;
            writeln!(f, "First Trace Divergence")?;
            writeln!(f, "  {}", divergence)?;
        }

        Ok(())
    }
}

impl fmt::Display for BackendTraceDiffReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Backend Trace Diff")?;
        writeln!(
            f,
            "  status: {}",
            if self.equivalent {
                "equivalent"
            } else {
                "mismatch"
            }
        )?;
        writeln!(f, "  {} events: {}", self.left_backend, self.left_events)?;
        writeln!(f, "  {} events: {}", self.right_backend, self.right_events)?;
        writeln!(
            f,
            "  {}: {}",
            self.left_backend,
            format_outcome(&self.left_outcome)
        )?;
        writeln!(
            f,
            "  {}: {}",
            self.right_backend,
            format_outcome(&self.right_outcome)
        )?;

        if let Some(mismatch) = &self.outcome_mismatch {
            writeln!(f)?;
            writeln!(f, "Outcome Mismatch")?;
            writeln!(f, "  {}", mismatch)?;
        }

        if let Some(divergence) = &self.divergence {
            writeln!(f)?;
            writeln!(f, "First Trace Divergence")?;
            writeln!(f, "  {}", divergence)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::Compiler;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::sema::SemanticAnalyzer;

    fn compile_source(source: &str) -> CompiledProgram {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().expect("parse failed");
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&program).expect("sema failed");
        Compiler::new().compile(&program).0
    }

    #[test]
    fn replays_reference_vm_trace() {
        let program = compile_source("func main() { int x = 40 + 2; return x; }");
        let report = replay_vm_trace(&program);

        assert!(report.replayable, "{report:#?}");
        assert!(report.event_count > 0);
    }

    #[test]
    fn replay_report_has_stable_evidence_json() {
        let program = compile_source("func main() { int x = 40 + 2; return x; }");
        let report = replay_vm_trace(&program);
        let first_json = report.evidence_json();
        let second_json = report.evidence_json();

        assert_eq!(first_json, second_json);
        assert!(first_json.contains("\"kind\":\"trace_replay\""));
        assert!(first_json.contains("\"replayable\":true"));
        assert!(first_json.contains("\"original_trace\":"));
        assert!(first_json.contains(&format!("\"fingerprint\":\"{}\"", report.fingerprint_hex())));
        assert_eq!(
            report.original_trace_summary.fingerprint,
            report.replay_trace_summary.fingerprint
        );
    }

    #[test]
    fn diffs_vm_and_gc_trace_for_simple_program() {
        let program = compile_source("func main() { int x = 5; return x * 3; }");
        let report = diff_vm_gc_traces(&program);

        assert!(report.equivalent, "{report:#?}");
        assert!(report.left_events > 0);
        assert_eq!(report.left_events, report.right_events);
    }

    #[test]
    fn backend_diff_report_has_machine_readable_trace_summaries() {
        let program = compile_source("func main() { int x = 5; return x * 3; }");
        let report = diff_vm_gc_traces(&program);
        let json = report.evidence_json();

        assert!(json.contains("\"kind\":\"backend_trace_diff\""));
        assert!(json.contains("\"equivalent\":true"));
        assert!(json.contains("\"left_backend\":\"VM\""));
        assert!(json.contains("\"right_backend\":\"GC VM\""));
        assert!(json.contains("\"left_trace\":"));
        assert!(json.contains("\"right_trace\":"));
        assert!(json.contains(&format!("\"fingerprint\":\"{}\"", report.fingerprint_hex())));
    }
}
