use std::io::{self, BufRead, Write};

use scrysynth_lib::visual::bevy_runtime::run_visible_runtime;
use scrysynth_lib::visual::protocol::AppToVisualMessage;
use scrysynth_lib::visual::sidecar::{error_response, MinimalVisualRuntime};

fn main() {
    let run_minimal = std::env::args().any(|arg| arg == "--minimal")
        || std::env::var("SCRYSYNTH_VISUAL_MODE").as_deref() == Ok("minimal");

    if !run_minimal {
        run_visible_runtime();
        return;
    }

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut runtime = MinimalVisualRuntime::new();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(err) => {
                write_message(&mut stdout, &error_response(None, err.to_string()));
                continue;
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        let message = match serde_json::from_str::<AppToVisualMessage>(&line) {
            Ok(message) => message,
            Err(err) => {
                write_message(&mut stdout, &error_response(None, err.to_string()));
                continue;
            }
        };

        let responses = runtime.handle_message(message);
        for response in responses {
            write_message(&mut stdout, &response);
        }

        if runtime.should_shutdown() {
            break;
        }
    }
}

fn write_message<W: Write>(
    writer: &mut W,
    message: &scrysynth_lib::visual::protocol::VisualToAppMessage,
) {
    if serde_json::to_writer(&mut *writer, message).is_ok() {
        let _ = writer.write_all(b"\n");
        let _ = writer.flush();
    }
}
