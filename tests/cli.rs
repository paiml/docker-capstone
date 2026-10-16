use shipgate::{parse, run, run_all, run_gate, Gate};

fn plan(name: &str, body: &str) -> String {
    let p = std::env::temp_dir().join(format!("shipgate-{name}"));
    std::fs::write(&p, body).unwrap();
    p.to_string_lossy().into_owned()
}

#[test]
fn parse_skips_comments_and_blanks() {
    let g = parse("# c\n\na: true\n b : false \n").unwrap();
    assert_eq!(
        g,
        vec![
            Gate {
                name: "a".into(),
                command: "true".into()
            },
            Gate {
                name: "b".into(),
                command: "false".into()
            },
        ]
    );
}

#[test]
fn parse_rejects_a_line_without_colon() {
    assert!(parse("nonsense\n").unwrap_err().contains("line 1"));
}

#[test]
fn gate_passes_on_zero_and_fails_otherwise() {
    assert!(run_gate(&Gate {
        name: "t".into(),
        command: "true".into()
    }));
    assert!(!run_gate(&Gate {
        name: "f".into(),
        command: "exit 3".into()
    }));
}

#[test]
fn run_all_lists_failures() {
    let gates = parse("ok: true\nbad: false\n").unwrap();
    let mut out = Vec::new();
    let failed = run_all(&gates, &mut out).unwrap();
    assert_eq!(failed, vec!["bad".to_string()]);
    assert_eq!(String::from_utf8(out).unwrap(), "PASS ok\nFAIL bad\n");
}

#[test]
fn all_pass_ships() {
    let (mut o, mut e) = (Vec::new(), Vec::new());
    let code = run(&[plan("ok", "a: true\nb: true\n")], &mut o, &mut e);
    assert_eq!(code, 0);
    assert!(String::from_utf8(o)
        .unwrap()
        .ends_with("SHIP: 2 gates passed\n"));
}

#[test]
fn one_failure_holds() {
    let (mut o, mut e) = (Vec::new(), Vec::new());
    let code = run(&[plan("hold", "a: true\nb: false\n")], &mut o, &mut e);
    assert_eq!(code, 1);
    assert!(String::from_utf8(o)
        .unwrap()
        .ends_with("HOLD: 1 of 2 gates failed\n"));
}

#[test]
fn usage_missing_file_and_bad_plan_exit_two() {
    let (mut o, mut e) = (Vec::new(), Vec::new());
    assert_eq!(run(&[], &mut o, &mut e), 2);
    assert_eq!(run(&["/nonexistent/plan".into()], &mut o, &mut e), 2);
    assert_eq!(run(&[plan("bad", "oops\n")], &mut o, &mut e), 2);
}

#[test]
#[ignore]
fn calls_nothing() {}

struct Broken;
impl std::io::Write for Broken {
    fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("closed"))
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn unwritable_output_exits_two() {
    let mut e = Vec::new();
    assert_eq!(run(&[plan("ok2", "a: true\n")], &mut Broken, &mut e), 2);
}
