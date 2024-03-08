use byteorder::ReadBytesExt;
use std::io;
use std::io::{Read, Write};

pub type Rgb = (u8, u8, u8);

#[allow(dead_code)]
pub trait TerrariaTypesR {
    fn read_terraria_string(&mut self) -> io::Result<String>;
    fn read_terraria_rgb(&mut self) -> io::Result<Rgb>;
}

pub trait TerrariaTypesW {
    fn write_terraria_string(&mut self, value: String) -> io::Result<()>;
    fn write_terraria_rgb(&mut self, value: Rgb) -> io::Result<()>;
}

impl<T: Read> TerrariaTypesR for T {
    fn read_terraria_string(&mut self) -> io::Result<String> {
        let len = self.read_u8()? - 2;
        let mut buf = vec![0u8; len as usize];
        self.read_exact(&mut buf)?;
        Ok(String::from_utf8_lossy(&buf).to_string())
    }

    fn read_terraria_rgb(&mut self) -> io::Result<Rgb> {
        let r = self.read_u8()?;
        let g = self.read_u8()?;
        let b = self.read_u8()?;
        Ok((r, g, b))
    }
}

impl<T: Write> TerrariaTypesW for T {
    fn write_terraria_string(&mut self, value: String) -> io::Result<()> {
        let mut bytes = vec![];
        let mut len = value.len() as i32;
        while len >= 128 {
            bytes.push((len | 0x80) as u8);
            len >>= 7;
        }
        bytes.push(len as u8);
        bytes.extend(value.as_bytes());
        self.write_all(&bytes)
    }

    fn write_terraria_rgb(&mut self, value: Rgb) -> io::Result<()> {
        let bytes = vec![value.0, value.1, value.2];
        self.write_all(&bytes)
    }
}
