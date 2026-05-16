use crate::keyboard_parser::{KittyKeyboardParser, KittyParseOutcome};
use crate::os_input_output::ClientOsApi;
use crate::stdin_ansi_parser::StdinAnsiParser;
#[cfg(windows)]
use crate::stdin_handler_windows::enable_vt_input;
use crate::InputInstruction;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use zellij_utils::{
    channels::SenderWithContext,
    vendored::termwiz::input::{InputEvent, InputParser},
};

struct TerminalApcParser {
    pending: Vec<u8>,
}

impl TerminalApcParser {
    fn new() -> Self {
        Self { pending: vec![] }
    }

    fn parse(&mut self, bytes: &[u8]) -> (Vec<Vec<u8>>, Vec<Vec<u8>>) {
        self.pending.extend_from_slice(bytes);
        let mut passthrough = vec![];
        let mut apcs = vec![];
        loop {
            if self.pending.is_empty() {
                break;
            }
            let apc_start = self
                .pending
                .windows(2)
                .position(|window| window == b"\x1b_");
            let Some(apc_start) = apc_start else {
                if self.pending.last() == Some(&b'\x1b') {
                    if self.pending.len() > 1 {
                        passthrough.push(self.pending.drain(..self.pending.len() - 1).collect());
                    }
                } else {
                    passthrough.push(self.pending.drain(..).collect());
                }
                break;
            };
            if apc_start > 0 {
                passthrough.push(self.pending.drain(..apc_start).collect());
                continue;
            }
            let apc_end = self
                .pending
                .windows(2)
                .enumerate()
                .skip(2)
                .find_map(|(index, window)| (window == b"\x1b\\").then_some(index));
            let Some(apc_end) = apc_end else {
                break;
            };
            let payload = self.pending[2..apc_end].to_vec();
            self.pending.drain(..apc_end + 2);
            if payload.first() == Some(&b'G') {
                apcs.push(payload);
            }
        }
        (passthrough, apcs)
    }

    fn finalize(&mut self) -> Vec<u8> {
        self.pending.drain(..).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{send_kitty_apcs_then_completed_forward, TerminalApcParser};
    use crate::InputInstruction;
    use zellij_utils::channels::{unbounded, SenderWithContext};

    #[test]
    fn terminal_apc_parser_extracts_complete_kitty_apc_and_passes_other_bytes_through() {
        let mut parser = TerminalApcParser::new();

        let (passthrough, apcs) = parser.parse(b"abc\x1b_Gi=1;OK\x1b\\def");

        assert_eq!(passthrough, vec![b"abc".to_vec(), b"def".to_vec()]);
        assert_eq!(apcs, vec![b"Gi=1;OK".to_vec()]);
    }

    #[test]
    fn terminal_apc_parser_handles_split_kitty_apc() {
        let mut parser = TerminalApcParser::new();

        let (passthrough, apcs) = parser.parse(b"abc\x1b_Gi=1");
        assert_eq!(passthrough, vec![b"abc".to_vec()]);
        assert!(apcs.is_empty());

        let (passthrough, apcs) = parser.parse(b";OK\x1b\\def");
        assert_eq!(passthrough, vec![b"def".to_vec()]);
        assert_eq!(apcs, vec![b"Gi=1;OK".to_vec()]);
    }

    #[test]
    fn terminal_apc_parser_does_not_swallow_split_escape_keys() {
        let mut parser = TerminalApcParser::new();

        let (passthrough, apcs) = parser.parse(b"\x1b");
        assert!(passthrough.is_empty());
        assert!(apcs.is_empty());

        let (passthrough, apcs) = parser.parse(b"[A");
        assert_eq!(passthrough, vec![b"\x1b[A".to_vec()]);
        assert!(apcs.is_empty());
    }

    #[test]
    fn kitty_apc_responses_are_forwarded_before_the_da_barrier_completion() {
        let (sender, receiver) = unbounded();
        let sender = SenderWithContext::new(sender);

        send_kitty_apcs_then_completed_forward(
            sender,
            vec![b"Gi=9;OK".to_vec()],
            Some((9, Vec::new())),
        );

        let first = receiver.recv().unwrap().0;
        let second = receiver.recv().unwrap().0;

        assert!(matches!(
            first,
            InputInstruction::KittyImageTerminalResponse(ref payload)
                if payload == b"Gi=9;OK"
        ));
        assert!(matches!(
            second,
            InputInstruction::ForwardedReplyFromHostComplete {
                token: 9,
                reply_bytes
            } if reply_bytes.is_empty()
        ));
    }
}

fn send_kitty_apcs_then_completed_forward(
    send_input_instructions: SenderWithContext<InputInstruction>,
    kitty_apcs: Vec<Vec<u8>>,
    completed_forward: Option<(u32, Vec<u8>)>,
) {
    for kitty_apc in kitty_apcs {
        let _ =
            send_input_instructions.send(InputInstruction::KittyImageTerminalResponse(kitty_apc));
    }
    if let Some((token, reply_bytes)) = completed_forward {
        let _ = send_input_instructions
            .send(InputInstruction::ForwardedReplyFromHostComplete { token, reply_bytes });
    }
}

pub(crate) fn stdin_loop(
    mut os_input: Box<dyn ClientOsApi>,
    send_input_instructions: SenderWithContext<InputInstruction>,
    stdin_ansi_parser: Arc<Mutex<StdinAnsiParser>>,
    explicitly_disable_kitty_keyboard_protocol: bool,
    resize_sender: Option<std::sync::mpsc::Sender<()>>,
) {
    // On Windows, choose between two input strategies early — we need this
    // decision before the startup ANSI query below.
    //
    // 1. Native console (no TERM env var): Use crossterm's event::read() which
    //    reads INPUT_RECORDs via ReadConsoleInput. Works in cmd.exe, PowerShell,
    //    and Windows Terminal where ALT is reported as a modifier flag.
    //
    // 2. Terminal emulator (TERM is set, e.g. Alacritty): Enable
    //    ENABLE_VIRTUAL_TERMINAL_INPUT so ReadFile on stdin returns raw VT bytes,
    //    bypassing conpty's lossy VT→INPUT_RECORD translation. Then use the
    //    termwiz byte parser (same as Unix) which understands ESC-prefixed ALT.
    #[cfg(windows)]
    let use_vt_reader = std::env::var("TERM").is_ok() && enable_vt_input();

    // Send the startup host query string so the host terminal replies
    // with its live pixel dimensions, fg/bg, sync-output support, and
    // palette registers. These replies will be classified by the
    // continuous parser as they arrive and routed via `InputInstruction::
    // AnsiStdinInstructions` — no deadline, no cache, no loading gate.
    {
        // On Windows native console, the crossterm event::read() loop
        // reads INPUT_RECORDs via ReadConsoleInput — not raw bytes — so
        // ANSI query responses can never be read on that path.
        #[cfg(windows)]
        let can_query_terminal = use_vt_reader;
        #[cfg(not(windows))]
        let can_query_terminal = true;

        if can_query_terminal {
            let query_string = build_startup_query_string();
            let _ = os_input
                .get_stdout_writer()
                .write(query_string.as_bytes())
                .unwrap();
        }
    }

    #[cfg(windows)]
    if !use_vt_reader {
        crate::stdin_handler_windows::native_console_stdin_loop(
            send_input_instructions,
            resize_sender,
        );
        return;
    }

    // Drop the resize sender so the signal handler thread falls back to
    // polling. Only the Windows native console path (above) keeps it alive;
    // the VT reader path and Unix don't produce crossterm resize events.
    drop(resize_sender);

    // Byte reader + termwiz/kitty parser path.
    // Used on Unix always, and on Windows inside terminal emulators (Alacritty,
    // etc.) with ENABLE_VIRTUAL_TERMINAL_INPUT enabled so stdin delivers raw VT
    // byte sequences.
    let mut input_parser = InputParser::new();
    // Kitty keyboard parser is long-lived so a Kitty CSI sequence split
    // across stdin reads still resolves on a follow-up chunk instead of
    // silently degrading to a legacy CSI form (and losing modifier
    // metadata).
    let mut kitty_parser = KittyKeyboardParser::new();
    let mut terminal_apc_parser = TerminalApcParser::new();
    let mut current_buffer = vec![];
    let (stdin_tx, stdin_rx) = mpsc::sync_channel(32);
    let _stdin_pump = std::thread::Builder::new()
        .name("stdin_pump".to_string())
        .spawn({
            move || loop {
                match os_input.read_from_stdin() {
                    Ok(buf) => {
                        if stdin_tx.send(Ok(buf)).is_err() {
                            break; // receiver dropped
                        }
                    },
                    Err(e) => {
                        let _ = stdin_tx.send(Err(e));
                        break;
                    },
                }
            }
        });
    let mut needs_finalization = false;
    loop {
        match if needs_finalization {
            stdin_rx.recv_timeout(Duration::from_millis(50))
        } else {
            stdin_rx
                .recv()
                .map_err(|_| mpsc::RecvTimeoutError::Disconnected)
        } {
            Ok(result) => {
                match result {
                    Ok(buf) => {
                        // Strip + classify any host-reply sequences
                        // continuously. The residue is the byte stream
                        // the keyboard parser should see.
                        let parse_output = {
                            let mut p = stdin_ansi_parser.lock().unwrap();
                            p.feed(&buf)
                        };
                        if !parse_output.replies.is_empty() {
                            let _ = send_input_instructions.send(
                                InputInstruction::AnsiStdinInstructions(parse_output.replies),
                            );
                        }
                        let completed_forward = parse_output.completed_forward;
                        for payload in parse_output.desktop_notifications {
                            let _ = send_input_instructions
                                .send(InputInstruction::DesktopNotificationResponse(payload));
                        }
                        let has_partial = parse_output.has_partial_state;
                        let residue = parse_output.residue;
                        if residue.is_empty() {
                            send_kitty_apcs_then_completed_forward(
                                send_input_instructions.clone(),
                                Vec::new(),
                                completed_forward,
                            );
                            // If all bytes were consumed by the host-reply
                            // parser, nothing to feed to the keyboard
                            // parser. But if the host-reply parser is
                            // sitting on a buffered partial sequence
                            // (e.g. a lone trailing ESC waiting to be
                            // disambiguated), schedule a finalize tick
                            // so the idle drain releases it as keyboard
                            // residue when no follow-up arrives.
                            if has_partial {
                                needs_finalization = true;
                            }
                            continue;
                        }
                        let (passthrough_chunks, kitty_apcs) = terminal_apc_parser.parse(&residue);
                        send_kitty_apcs_then_completed_forward(
                            send_input_instructions.clone(),
                            kitty_apcs,
                            completed_forward,
                        );
                        if passthrough_chunks.is_empty() && !residue.is_empty() {
                            needs_finalization = true;
                            continue;
                        }
                        for buf in passthrough_chunks {
                            current_buffer.append(&mut buf.to_vec());

                            if !explicitly_disable_kitty_keyboard_protocol {
                                // first we try to parse with the KittyKeyboardParser
                                // if we fail, we try to parse normally.
                                // Incomplete and NoMatch both fall through to the
                                // termwiz parser below; on Incomplete the Kitty
                                // parser keeps its state so the next chunk's
                                // continuation completes the sequence.
                                match kitty_parser.feed(&buf) {
                                    KittyParseOutcome::Complete(key_with_modifier) => {
                                        send_input_instructions
                                            .send(InputInstruction::KeyWithModifierEvent(
                                                key_with_modifier,
                                                current_buffer.drain(..).collect(),
                                                true,
                                            ))
                                            .unwrap();
                                        continue;
                                    },
                                    KittyParseOutcome::Incomplete | KittyParseOutcome::NoMatch => {
                                    },
                                }
                            }

                            // Parse with maybe_more = true - complete events sent immediately
                            //
                            // Ambiguous events (if any) will be finalized later only if 50ms
                            // passes with no new input
                            let maybe_more = true;
                            let mut events = vec![];
                            input_parser.parse(
                                &buf,
                                |input_event: InputEvent| {
                                    events.push(input_event);
                                },
                                maybe_more,
                            );

                            for input_event in events.into_iter() {
                                match input_event {
                                    InputEvent::OperatingSystemCommand(ref payload) => {
                                        if payload.starts_with(b"99;") {
                                            let notification_payload =
                                                payload.get(3..).unwrap_or_default().to_vec();
                                            let _ = send_input_instructions.send(
                                                InputInstruction::DesktopNotificationResponse(
                                                    notification_payload,
                                                ),
                                            );
                                        }
                                        // Other OSC types at runtime: silently drop.
                                    },
                                    other => {
                                        send_input_instructions
                                            .send(InputInstruction::KeyEvent(
                                                other,
                                                current_buffer.drain(..).collect(),
                                            ))
                                            .unwrap();
                                    },
                                }
                            }
                            needs_finalization = true;
                        }
                    },
                    Err(e) => {
                        if e == "Session ended" {
                            log::debug!("Switched sessions, signing this thread off...");
                        } else {
                            log::error!("Failed to read from STDIN: {}", e);
                        }
                        let _ = send_input_instructions.send(InputInstruction::Exit);
                        break;
                    },
                }
            },
            Err(mpsc::RecvTimeoutError::Timeout) => {
                finalize_events(
                    &mut input_parser,
                    &mut terminal_apc_parser,
                    &mut current_buffer,
                    send_input_instructions.clone(),
                    &stdin_ansi_parser,
                );
                needs_finalization = false;
            },
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                log::debug!("STDIN pump disconnected");
                let _ = send_input_instructions.send(InputInstruction::Exit);
                break;
            },
        }
    }
}

fn finalize_events(
    input_parser: &mut InputParser,
    terminal_apc_parser: &mut TerminalApcParser,
    current_buffer: &mut Vec<u8>,
    send_input_instructions: SenderWithContext<InputInstruction>,
    stdin_ansi_parser: &Arc<Mutex<StdinAnsiParser>>,
) {
    // Drain any speculatively-buffered partial host-reply bytes (a
    // lone trailing ESC, or an unterminated OSC/CSI prefix whose
    // follow-up never arrived). They become keyboard residue — same
    // path real keypress bytes take. Without this drain, a real Esc
    // press whose byte was parked under partial_osc would be lost.
    let mut drained = stdin_ansi_parser.lock().unwrap().finalize();
    if !drained.is_empty() {
        let (passthrough_chunks, kitty_apcs) = terminal_apc_parser.parse(&drained);
        for kitty_apc in kitty_apcs {
            let _ = send_input_instructions
                .send(InputInstruction::KittyImageTerminalResponse(kitty_apc));
        }
        drained = passthrough_chunks.into_iter().flatten().collect();
    }
    drained.extend(terminal_apc_parser.finalize());
    if !drained.is_empty() {
        current_buffer.extend_from_slice(&drained);
    }

    let mut events = vec![];
    input_parser.parse(
        &drained,
        |input_event: InputEvent| {
            events.push(input_event);
        },
        false,
    );
    // Residue contains no OSC or whitelisted CSI reports — every
    // termwiz event drained on idle is a key/mouse/paste/etc.
    for input_event in events {
        send_input_instructions
            .send(InputInstruction::KeyEvent(
                input_event,
                current_buffer.drain(..).collect(),
            ))
            .unwrap();
    }
}

/// Build the fire-and-forget host-query batch sent at client startup.
/// The host's replies refine `Screen`'s cached state asynchronously as
/// they arrive; the UI does not block on them.
fn build_startup_query_string() -> String {
    // <ESC>[14t => get text area size in pixels,
    // <ESC>[16t => get character cell size in pixels
    // <ESC>]11;?<ESC>\ => get background color
    // <ESC>]10;?<ESC>\ => get foreground color
    // <ESC>[?2026$p => get synchronised output mode
    let mut query_string = String::from(
        "\u{1b}[14t\u{1b}[16t\u{1b}]11;?\u{1b}\u{5c}\u{1b}]10;?\u{1b}\u{5c}\u{1b}[?2026$p",
    );
    // query colors
    // eg. <ESC>]4;5;?<ESC>\ => query color register number 5
    for i in 0..256 {
        query_string.push_str(&format!("\u{1b}]4;{};?\u{1b}\u{5c}", i));
    }
    query_string
}
