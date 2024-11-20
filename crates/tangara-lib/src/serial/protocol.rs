use core::slice;
use std::io::IoSlice;

use futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use thiserror::Error;

const MAX_UNFRAMED_LINE: usize = 1024;

const FRAME_BEGIN: u8 = 0xfd;
const FRAME_END: u8 = 0xfe;

const OPCODE_MASK: u8 = 0xf0;
const OPCODE_DATA: u8 = 0x00;
const OPCODE_SHUTDOWN: u8 = 0x10;
const OPCODE_OPEN: u8 = 0x20;

const CHANNEL_MASK: u8 = 0x0f;

pub struct Protocol<T> {
    io: T,
    yielded: Option<Yielded>,
    unframed: Vec<u8>,
    frame_data: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum SendError {
    #[error("frame data too long")]
    DataTooLong,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum ReceiveError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl<T: AsyncRead + AsyncWrite + Unpin> Protocol<T> {
    pub fn new(io: T) -> Self {
        Protocol {
            io,
            yielded: None,
            unframed: Vec::with_capacity(128),
            frame_data: Vec::with_capacity(256),
        }
    }

    pub async fn send<'a>(&mut self, frame: &Frame<'a>) -> Result<(), SendError> {
        // add 1 for header byte
        let len = frame.data.len() + 1;

        // validate frame length
        let len = u8::try_from(len).map_err(|_| SendError::DataTooLong)?;

        let header = frame.channel.to_header_byte()
                   | frame.opcode.to_header_byte();

        let cksum = frame.data.iter()
            .fold(header, |a, b| a.wrapping_add(*b));

        let begin = [FRAME_BEGIN, len, header];
        let end = [cksum, FRAME_END];

        self.io.write_vectored(&[
            IoSlice::new(&begin),
            IoSlice::new(frame.data),
            IoSlice::new(&end),
        ]).await?;

        Ok(())
    }

    pub async fn receive(&mut self) -> Result<Event<'_>, ReceiveError> {
        // if the previous call to receive yielded some data
        // owned by self, clear that data now
        match self.yielded.take() {
            Some(Yielded::UnframedLine) => { self.unframed.clear(); }
            None => {}
        }

        // This slightly funny code where receive_one returns an EventKind
        // that does not borrow from self, and then we parse that here to
        // borrow from self, is a workaround for a limitation in the current
        // borrow checker. It is sound to return the Event<'_> directly from
        // these methods, but the borrow checker rejects this currently.
        let event = loop {
            if let Some(event) = self.receive_one().await? {
                break event;
            }
        };

        match event {
            EventKind::Frame(frame) => Ok(Event::Frame(Frame {
                opcode: frame.opcode,
                channel: frame.channel,
                data: &self.frame_data[1..],
            })),
            EventKind::UnframedLine => Ok(Event::UnframedLine(&self.unframed))
        }
    }

    async fn receive_one(&mut self) -> Result<Option<EventKind>, ReceiveError> {
        let unframed_len = self.unframed.len();

        let byte = self.read_byte().await?;

        if byte == FRAME_BEGIN {
            if let Some(frame) = self.receive_frame().await? {
                return Ok(Some(EventKind::Frame(frame)));
            }

            Ok(None)
        } else {
            if unframed_len < MAX_UNFRAMED_LINE {
                self.unframed.push(byte);
            }

            if byte == b'\n' {
                // end of line
                self.yielded.insert(Yielded::UnframedLine);
                return Ok(Some(EventKind::UnframedLine));
            }

            Ok(None)
        }
    }

    async fn receive_frame(&mut self) -> Result<Option<FrameEvent>, ReceiveError> {
        // clear internal frame data buffer for new frame
        self.frame_data.clear();

        // frame begin byte read
        // read length
        let len = self.read_byte().await?;
        let len = usize::from(len);

        // read data and calculate checksum on the fly
        let mut cksum = 0u8;
        for _ in 0..len {
            let byte = self.read_byte().await?;
            self.frame_data.push(byte);
            cksum = cksum.wrapping_add(byte);
        }

        // read checksum byte from the wire
        let wire_cksum = self.read_byte().await?;

        // read + check frame end byte
        if self.read_byte().await? != FRAME_END {
            // invalid frame, drop it
            return Ok(None);
        }

        if cksum != wire_cksum {
            // invalid checksum, drop frame
            return Ok(None);
        }

        if len < 1 {
            // all frames must contain a header byte, drop invalid frame
            return Ok(None);
        }

        let header = self.frame_data[0];

        let Some(opcode) = Opcode::from_header_byte(header) else {
            return Ok(None);
        };

        let channel = ChannelId::from_header_byte(header);

        return Ok(Some(FrameEvent { opcode, channel }));
    }

    async fn read_byte(&mut self) -> Result<u8, ReceiveError> {
        let mut byte = 0u8;
        self.io.read_exact(slice::from_mut(&mut byte)).await?;
        Ok(byte)
    }
}

enum Yielded {
    UnframedLine,
}

enum EventKind {
    UnframedLine,
    Frame(FrameEvent),
}

struct FrameEvent {
    opcode: Opcode,
    channel: ChannelId,
}

pub enum Event<'a> {
    UnframedLine(&'a [u8]),
    Frame(Frame<'a>),
}

pub struct Frame<'a> {
    pub opcode: Opcode,
    pub channel: ChannelId,
    pub data: &'a [u8],
}

#[derive(Clone, Copy, Debug)]
pub enum Opcode {
    Data,
    Shutdown,
    Open,
}

impl Opcode {
    pub fn from_header_byte(byte: u8) -> Option<Self> {
        match byte & OPCODE_MASK {
            OPCODE_DATA => Some(Opcode::Data),
            OPCODE_SHUTDOWN => Some(Opcode::Shutdown),
            OPCODE_OPEN => Some(Opcode::Open),
            _ => None,
        }
    }

    pub fn to_header_byte(&self) -> u8 {
        match self {
            Opcode::Data => OPCODE_DATA,
            Opcode::Shutdown => OPCODE_SHUTDOWN,
            Opcode::Open => OPCODE_OPEN,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ChannelId(pub u8);

impl ChannelId {
    pub fn from_header_byte(byte: u8) -> Self {
        ChannelId(byte & CHANNEL_MASK)
    }

    pub fn to_header_byte(&self) -> u8 {
        self.0 & CHANNEL_MASK
    }
}
