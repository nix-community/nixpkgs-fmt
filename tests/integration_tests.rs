use std::fs::File;
use std::path::Path;
use std::process::{Command, Stdio};

// It's an important Unix convention that text processing tools such as one might
// use in a pipeline terminate cleanly if stdout is closed prematurely.  If grep
// didn't do this, there'd be an error message every time somebody piped it into
// head.  Unix processes even do this by default -- when a process writes to a
// closed pipe, it's sent a SIGPIPE signal, which by default terminates the
// process.  But the Rust runtime ignores SIGPIPE, so Rust programs have to go out
// of their way to restore the default SIGPIPE behaviour (or emulate it by checking
// for EPIPE every time they write) to be good Unix citizens.
fn test_stdout_closed(args: &[&str]) {
    use libc::{waitpid, SIGPIPE, WIFSIGNALED, WTERMSIG};
    use std::io::copy;
    use std::thread::spawn;

    // Drop the child's stdout to close it.
    let (pid, mut stdin) = {
        let child = Command::new(env!("CARGO_BIN_EXE_nixpkgs-fmt"))
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        (child.id(), child.stdin.unwrap())
    };

    spawn(move || {
        let input_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("test_data/binop_wrap_before.bad.nix");
        let mut input = File::open(input_path).unwrap();
        copy(&mut input, &mut stdin).unwrap();
        drop(stdin)
    });

    // We have to use libc because we don't have a Child struct any more.
    let mut wstatus = 0;
    assert_eq!(unsafe { waitpid(pid as i32, &mut wstatus, 0) }, pid as i32);

    assert!(WIFSIGNALED(wstatus));
    assert_eq!(WTERMSIG(wstatus), SIGPIPE);
}

#[test]
fn stdout_closed() {
    test_stdout_closed(&[]);
}

#[test]
fn stdout_closed_parse() {
    test_stdout_closed(&["--parse"]);
}

#[test]
fn stdout_closed_explain() {
    test_stdout_closed(&["--explain"]);
}
