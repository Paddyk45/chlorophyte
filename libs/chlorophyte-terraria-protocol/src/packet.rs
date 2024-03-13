use crate::types::{TerrariaTypesR, TerrariaTypesW};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::fmt::Debug;
use std::io::{Cursor, ErrorKind, Read, Write};
use std::{io, vec};

pub type Rgb = (u8, u8, u8);

pub trait ReadTerrariaPacket {
    fn read_terraria_packet<P: S2CTerrariaPacket>(&mut self) -> io::Result<P>;
}
pub trait WriteTerrariaPacket {
    fn write_terraria_packet<P: C2STerrariaPacket>(&mut self, packet: P) -> io::Result<()>;
}

impl<T: Write> WriteTerrariaPacket for T {
    /// Writes a Terraria packet with length prefix, packet id and fields
    fn write_terraria_packet<P: C2STerrariaPacket>(&mut self, packet: P) -> io::Result<()> {
        let body = packet.serialize();
        let mut packet = vec![];
        let len = 2 + 1 + body.len() as u16;
        packet.extend_from_slice(&len.to_le_bytes());
        packet.push(P::PACKET_ID);
        packet.extend_from_slice(&body);
        self.write_all(&packet)?;

        Ok(())
    }
}

impl<T: Read + Debug> ReadTerrariaPacket for T {
    /// Reads a Terraria packet with length prefix, packet id and fields
    fn read_terraria_packet<P: S2CTerrariaPacket>(&mut self) -> io::Result<P> {
        let len = self.read_u16::<LittleEndian>()?;
        let mut buf = vec![0u8; len as usize];
        let n = self.read(&mut buf)?;
        if n < len as usize - 2 {
            return Err(io::Error::new(ErrorKind::WouldBlock, "Packet is too short"));
        }

        P::deserialize(buf[1..].to_vec()).map(|p| *p)
    }
}

pub trait C2STerrariaPacket {
    const PACKET_ID: u8;
    fn serialize(self) -> Vec<u8>;
}

pub trait S2CTerrariaPacket {
    fn deserialize(bytes: Vec<u8>) -> io::Result<Box<Self>>;
}

pub struct C2SConnect {
    pub version: u32,
}

impl C2STerrariaPacket for C2SConnect {
    const PACKET_ID: u8 = 1;
    fn serialize(self) -> Vec<u8> {
        let mut packet: Vec<u8> = vec![];
        packet
            .write_terraria_string(format!("Terraria{}", self.version))
            .unwrap();
        packet
    }
}

pub struct S2CConnectionApproved {
    pub slot: u8,
}

impl S2CTerrariaPacket for S2CConnectionApproved {
    fn deserialize(bytes: Vec<u8>) -> io::Result<Box<Self>> {
        let mut c = Cursor::new(bytes);
        let slot = c.read_u8()?;
        let _ = c.read_u8();
        Ok(Box::new(Self { slot }))
    }
}

// Packet ID: 2
pub struct S2CFatalError {
    pub error: String,
}

impl S2CTerrariaPacket for S2CFatalError {
    fn deserialize(bytes: Vec<u8>) -> io::Result<Box<Self>> {
        let mut c = Cursor::new(bytes);
        let error = c.read_terraria_string()?;
        Ok(Box::new(Self { error }))
    }
}

// Packet ID: 37
pub struct S2CPasswordRequired;

impl S2CTerrariaPacket for S2CPasswordRequired {
    fn deserialize(_: Vec<u8>) -> io::Result<Box<Self>> {
        Ok(Box::new(Self))
    }
}

#[allow(dead_code)]
pub enum Difficulty {
    Normal,
    Mediumcore,
    Hardcore,
}

pub struct C2SPlayerAppearance {
    pub slot: u8,
    pub skin_variant: u8,
    pub hair: u8,
    pub name: String,
    pub hair_dye: u8,
    pub hide_visible_accessory: [bool; 10],
    pub hide_misc: bool,
    pub hair_color: Rgb,
    pub skin_color: Rgb,
    pub eye_color: Rgb,
    pub shirt_color: Rgb,
    pub undershirt_color: Rgb,
    pub pants_color: Rgb,
    pub shoe_color: Rgb,

    pub difficulty: Difficulty,
    pub extra_accessory: bool,

    pub using_biome_torches: bool,
    pub happy_fun_torch_time: bool,
    pub unlocked_biome_torches: bool,
    pub unlocked_super_cart: bool,
    pub enabled_super_cart: bool,

    pub used_aegis_crystal: bool,
    pub used_aegis_fruit: bool,
    pub used_arcane_crystal: bool,
    pub used_galaxy_pearl: bool,
    pub used_gummy_worm: bool,
    pub used_ambrosia: bool,
    pub ate_artisan_bread: bool,
}

impl C2STerrariaPacket for C2SPlayerAppearance {
    const PACKET_ID: u8 = 4;

    fn serialize(self) -> Vec<u8> {
        let mut packet = vec![];
        packet.push(self.slot);
        packet.push(self.skin_variant);
        packet.push(self.hair);
        packet.write_terraria_string(self.name).unwrap();
        packet.push(self.hair_dye);
        let mut hva = 0u16;
        for i in 0..10 {
            if self.hide_visible_accessory[i] {
                hva |= 1 << i;
            }
        }
        packet.write_u16::<LittleEndian>(hva).unwrap();
        packet.push(self.hide_misc as u8);
        packet.write_terraria_rgb(self.hair_color).unwrap();
        packet.write_terraria_rgb(self.skin_color).unwrap();
        packet.write_terraria_rgb(self.eye_color).unwrap();
        packet.write_terraria_rgb(self.shirt_color).unwrap();
        packet.write_terraria_rgb(self.undershirt_color).unwrap();
        packet.write_terraria_rgb(self.pants_color).unwrap();
        packet.write_terraria_rgb(self.shoe_color).unwrap();

        let mut bits1 = 0u8;
        match self.difficulty {
            Difficulty::Normal => bits1 |= 0b1000_0000,
            Difficulty::Mediumcore => bits1 |= 0b0100_0000,
            Difficulty::Hardcore => bits1 |= 0b0001_0000,
        }
        if self.extra_accessory {
            bits1 |= 0b0010_0000;
        }
        packet.push(bits1);

        let mut bits2 = 0u8;
        if self.unlocked_biome_torches {
            bits2 |= 0b1000_0000;
        }
        if self.happy_fun_torch_time {
            bits2 |= 0b0100_0000;
        }
        if self.unlocked_biome_torches {
            bits2 |= 0b0010_0000;
        }
        if self.unlocked_super_cart {
            bits2 |= 0b0001_0000;
        }
        if self.enabled_super_cart {
            bits2 |= 0b0000_1000;
        }
        packet.push(bits2);

        let mut bits3 = 0u8;
        if self.used_aegis_crystal {
            bits3 |= 0b1000_0000;
        }
        if self.used_aegis_fruit {
            bits3 |= 0b0100_0000;
        }
        if self.used_arcane_crystal {
            bits3 |= 0b0010_0000;
        }
        if self.used_galaxy_pearl {
            bits3 |= 0b0001_0000;
        }
        if self.used_gummy_worm {
            bits3 |= 0b0000_1000;
        }
        if self.used_ambrosia {
            bits3 |= 0b0000_0100;
        }
        if self.ate_artisan_bread {
            bits3 |= 0b0000_0010;
        }
        packet.push(bits3);
        packet
    }
}

pub struct C2SSetHealth {
    pub slot: u8,
    pub stat_life: i16,
    pub stat_life_max: i16,
}

impl C2STerrariaPacket for C2SSetHealth {
    const PACKET_ID: u8 = 16;

    fn serialize(self) -> Vec<u8> {
        let mut packet = vec![];
        packet.push(self.slot);
        packet.write_i16::<LittleEndian>(self.stat_life).unwrap();
        packet
            .write_i16::<LittleEndian>(self.stat_life_max)
            .unwrap();
        packet
    }
}

pub struct C2SSetMana {
    pub slot: u8,
    pub stat_mana: i16,
    pub stat_mana_max: i16,
}

impl C2STerrariaPacket for C2SSetMana {
    const PACKET_ID: u8 = 42;

    fn serialize(self) -> Vec<u8> {
        let mut packet = vec![];
        packet.push(self.slot);
        packet.write_i16::<LittleEndian>(self.stat_mana).unwrap();
        packet
            .write_i16::<LittleEndian>(self.stat_mana_max)
            .unwrap();
        packet
    }
}

pub struct C2SSetBuffs {
    pub slot: u8,
    pub buffs: Vec<u16>,
}

impl C2STerrariaPacket for C2SSetBuffs {
    const PACKET_ID: u8 = 50;

    fn serialize(self) -> Vec<u8> {
        let mut packet = vec![];
        if self.buffs.len() > 44 {
            panic!("Too many buffs")
        }
        for b in self.buffs.iter() {
            packet.write_u16::<LittleEndian>(*b).unwrap();
        }
        for _ in 0..(44 - self.buffs.len()) {
            packet.write_u16::<LittleEndian>(0).unwrap();
        }
        packet
    }
}

pub struct C2SSetInvSlot {
    pub slot: u8,
    pub inv_slot: u8,
    pub stack: i16,
    pub prefix_id: u8,
    pub net_id: i16,
}

impl C2STerrariaPacket for C2SSetInvSlot {
    const PACKET_ID: u8 = 5;

    fn serialize(self) -> Vec<u8> {
        let mut packet = vec![];
        packet.push(self.slot);
        packet.push(self.inv_slot);
        packet.extend_from_slice(&self.stack.to_le_bytes());
        packet.push(self.prefix_id);
        packet.extend_from_slice(&self.net_id.to_le_bytes());
        packet
    }
}

pub struct C2SRequestWorldInfo;

impl C2STerrariaPacket for C2SRequestWorldInfo {
    const PACKET_ID: u8 = 6;

    fn serialize(self) -> Vec<u8> {
        vec![]
    }
}

pub struct C2SClientUuid {
    pub uuid: String,
}

impl C2STerrariaPacket for C2SClientUuid {
    const PACKET_ID: u8 = 68;

    fn serialize(self) -> Vec<u8> {
        let mut packet = vec![];
        packet.write_terraria_string(self.uuid).unwrap();
        packet
    }
}

pub struct C2SRequestInitialTileData {
    pub x: i32,
    pub y: i32,
}

impl C2STerrariaPacket for C2SRequestInitialTileData {
    const PACKET_ID: u8 = 8;

    fn serialize(self) -> Vec<u8> {
        let mut packet = vec![];
        packet.extend_from_slice(&self.x.to_le_bytes());
        packet.extend_from_slice(&self.y.to_le_bytes());
        packet
    }
}

pub struct C2SNetMessageCommand {
    pub command: String,
    pub text: String,
}

impl C2STerrariaPacket for C2SNetMessageCommand {
    const PACKET_ID: u8 = 82;

    fn serialize(self) -> Vec<u8> {
        let mut packet = vec![];
        packet.extend_from_slice(&1u16.to_le_bytes());
        packet.write_terraria_string(self.command).unwrap();
        packet.write_terraria_string(self.text).unwrap();
        packet
    }
}