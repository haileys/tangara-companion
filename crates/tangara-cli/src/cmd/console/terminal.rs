use std::task::{Context, Poll};
use std::pin::Pin;
use std::io::{self, Write};

use console::{Key, Term};
use futures::ready;
use tokio::io::{AsyncWrite, AsyncRead, ReadBuf};
use tokio::sync::mpsc;

pub struct AsyncTerminal {
    rx: mpsc::Receiver<io::Result<u8>>,
}

impl AsyncTerminal {
    pub fn new(term: Term) -> Self {
        let (tx, rx) = mpsc::channel(64);
        std::thread::spawn(move || read_key_thread(term, tx));
        AsyncTerminal { rx }
    }
}

impl AsyncRead for AsyncTerminal {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
        if let Some(byte) = ready!(self.as_mut().rx.poll_recv(cx)).transpose()? {
            buf.put_slice(&[byte]);
        }

        Poll::Ready(Ok(()))
    }
}

impl AsyncWrite for AsyncTerminal {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        let mut stdout = std::io::stdout();
        stdout.write_all(buf)?;
        stdout.flush()?;
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        Poll::Ready(std::io::stdout().flush())
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>
    ) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }
}

fn read_key_thread(term: Term, tx: mpsc::Sender<io::Result<u8>>) {
    loop {
        let key = match term.read_key() {
            Ok(key) => key,
            Err(err) => {
                let _ = tx.blocking_send(Err(err));
                break;
            }
        };

        for byte in encode_key(key) {
            let Ok(()) = tx.blocking_send(Ok(byte)) else { return };
        }
    }
}

fn encode_key(key: Key) -> Vec<u8> {
    let mut buff = [0u8; 4];

    let bytes = match key {
        Key::Char(c) => c.encode_utf8(&mut buff).as_bytes(),
        Key::Enter => b"\n",
        Key::ArrowLeft => b"\x1b[D",
        Key::ArrowRight => b"\x1b[C",
        Key::ArrowUp => b"\x1b[A",
        Key::ArrowDown => b"\x1b[B",
        Key::End => b"\x1b[F",
        Key::Home => b"\x1b[H",
        Key::Tab => b"\t",
        Key::Del => b"\x1b[3~",
        Key::Backspace => &[8],
        k => {
            if cfg!(debug_assertions) {
                eprintln!("debug: unknown key: {k:?}");
            }
            return Vec::new();
        }
    };

    Vec::from(bytes)
}
