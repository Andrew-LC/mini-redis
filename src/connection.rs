use tokio::net::TcpStream;
use mini_redis::{Frame, Result};
use mini_redis::frame::Error::Incomplete;
use bytes::{BytesMut, Buf};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::io::BufWriter;
use std::io::Cursor;

// Example of how the Frame would look like
// enum Frame {
//     Simple(String),
//     Error(String),
//     Integer(u64),
//     Bulk(Bytes),
//     Null,
//     Array(Vec<Frame>),
// }

pub struct  Connection {
    stream: TcpStream,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
	Connection {
	    stream: BufWriter::new(stream),
	    buffer: BytesMut::with_capacity(4096),
	}
    }

    pub async fn read_frame(&mut self)
			    -> Result<Option<Frame>>
    {
	loop {
	    if let Some(frame) = self.parse_frame()? {
		return Ok(Some(frame))
	    }
	    // There is not enough buffered data to read a frame.
	    // Attempt to read more data from the socket.
	    //
	    // On success, the number of bytes is returned. `0`
	    // indicates "end of stream".
	    if 0 == self.stream.read_buf(&mut self.buffer).await? {
		// The remote closed the connection. For this to be
		// a clean shutdown, there should be no data in the
		// read buffer. If there is, this means that the
		// peer closed the socket while sending a frame.
		if self.buffer.is_empty() {
		    return Ok(None);
		} else {
		    return Err("connection reset by peer".into());
		}
	    }
	}
    }

    pub fn parse_frame(&mut self) {
	let mut buf = Cursor::new(&self.buffer[..]);

	match Frame::check(&mut buf) {
	    Ok(_) => {
		let len = buf.position() as usize;

		buf.set_position(0);

		let frame = Frame::parse(&mut buf);

		self.buffer.advance(len);

		OK(Some(frame))
	    },
	    Err(Incomplete) => Ok(None),
	    Err(e) => Err(e.into()),
	}
    }

    pub async fn write_frame(&mut self, frame: &Frame)
			 -> io::Result<()>
    {
	match frame {
            Frame::Simple(val) => {
		self.stream.write_u8(b'+').await?;
		self.stream.write_all(val.as_bytes()).await?;
		self.stream.write_all(b"\r\n").await?;
            }
            Frame::Error(val) => {
		self.stream.write_u8(b'-').await?;
		self.stream.write_all(val.as_bytes()).await?;
		self.stream.write_all(b"\r\n").await?;
            }
            Frame::Integer(val) => {
		self.stream.write_u8(b':').await?;
		self.write_decimal(*val).await?;
            }
            Frame::Null => {
		self.stream.write_all(b"$-1\r\n").await?;
            }
            Frame::Bulk(val) => {
		let len = val.len();

		self.stream.write_u8(b'$').await?;
		self.write_decimal(len as u64).await?;
		self.stream.write_all(val).await?;
		self.stream.write_all(b"\r\n").await?;
            }
            Frame::Array(_val) => unimplemented!(),
	}

	self.stream.flush().await;

	Ok(())
    }
}
