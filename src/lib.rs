//! `shipgate`: reads a plan of `name: command` lines, runs each command in `sh -c`,
//! and reports PASS or FAIL per gate. Ship only when every gate passes.

use std::io::Write;
use std::process::{Command, Stdio};

/// One named gate from a plan.
#[derive(Debug, PartialEq, Eq)]
pub struct Gate {
    pub name: String,
    pub command: String,
}

/// Parses a plan. Blank lines and `#` comments are skipped; a line without `: ` is an error.
pub fn parse(plan: &str) -> Result<Vec<Gate>, String> {
    plan.lines()
        .enumerate()
        .map(|(i, raw)| (i, raw.trim()))
        .filter(|(_, l)| !l.is_empty() && !l.starts_with('#'))
        .map(|(i, l)| match l.split_once(": ") {
            Some((name, command)) => Ok(Gate {
                name: name.trim().to_string(),
                command: command.trim().to_string(),
            }),
            None => Err(format!("line {}: expected `name: command`", i + 1)),
        })
        .collect()
}

/// Runs one gate quietly; true when its command exits 0.
pub fn run_gate(gate: &Gate) -> bool {
    Command::new("sh")
        .args(["-c", &gate.command])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Runs every gate in order, writing one PASS/FAIL line each; returns the failed gate names.
pub fn run_all(gates: &[Gate], out: &mut dyn Write) -> std::io::Result<Vec<String>> {
    let mut failed = Vec::new();
    for g in gates {
        let ok = run_gate(g);
        writeln!(out, "{} {}", if ok { "PASS" } else { "FAIL" }, g.name)?;
        if !ok {
            failed.push(g.name.clone());
        }
    }
    Ok(failed)
}

/// Command line: `shipgate <plan-file>`. Exit 0 all pass, 1 any fail, 2 usage or bad plan.
pub fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    let Some(path) = args.first() else {
        let _ = writeln!(err, "usage: shipgate <plan-file>");
        return 2;
    };
    let plan = match std::fs::read_to_string(path) {
        Ok(p) => p,
        Err(e) => {
            let _ = writeln!(err, "{path}: {e}");
            return 2;
        }
    };
    let gates = match parse(&plan) {
        Ok(g) => g,
        Err(e) => {
            let _ = writeln!(err, "{e}");
            return 2;
        }
    };
    match run_all(&gates, out) {
        Ok(failed) if failed.is_empty() => {
            let _ = writeln!(out, "SHIP: {} gates passed", gates.len());
            0
        }
        Ok(failed) => {
            let _ = writeln!(
                out,
                "HOLD: {} of {} gates failed",
                failed.len(),
                gates.len()
            );
            1
        }
        Err(_) => 2,
    }
}
