use std::sync::mpsc::{channel, Sender, Receiver};
use std::io::{Read, Write, self};

pub struct ChannelWriter {
    tx: Sender<u8>,
}

impl Write for ChannelWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut count = 0;
        for &b in buf {
            self.tx.send(b).map_err(|e| {
                eprintln!("Send error: {}", e);
                io::Error::new(io::ErrorKind::ConnectionRefused, e)
            })?;
            count += 1;
        }
        Ok(count)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub struct ChannelReader {
    rx: Receiver<u8>,
    blocking: bool,
}

impl Read for ChannelReader {
     fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut count = 0;
        if self.blocking {
            for b in self.rx.iter() {
                buf[count] = b;
                count += 1;
                if count >= buf.len() {break;}
            }
        } else {
            for b in self.rx.try_iter() {
                buf[count] = b;
                count += 1;
                if count >= buf.len() {break;}
            }
        }
        Ok(count)
     }
}

pub fn channel_io() -> (ChannelWriter, ChannelReader) {
    let (sender, receiver) = channel();
    (
        ChannelWriter {
            tx: sender,
        },
        ChannelReader {
            rx: receiver,
            blocking: false,
        }
    )
}
