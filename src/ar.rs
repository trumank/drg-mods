use std::convert::TryFrom;
use std::io::{Read, Write};

use anyhow::{anyhow, Result};
use byteorder::{ReadBytesExt, WriteBytesExt, BE, LE};

pub trait Readable<R> {
    fn read(reader: &mut R) -> Result<Self>
    where
        Self: Sized;
}
pub trait Writable<W> {
    fn write(&self, writer: &mut W) -> Result<()>;
}

type Guid = [u8; 16];
impl<R: Read> Readable<R> for Guid {
    fn read(reader: &mut R) -> Result<Self> {
        let mut buf = [0; 16];
        reader.read_exact(&mut buf)?;
        Ok(buf)
    }
}
impl<W: Write> Writable<W> for Guid {
    fn write(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self[..])?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NameIndex(usize);
impl<R: Read> Readable<R> for NameIndex {
    fn read(reader: &mut R) -> Result<Self> {
        Ok(NameIndex(reader.read_u32::<LE>()? as usize))
    }
}
impl<W: Write> Writable<W> for NameIndex {
    fn write(&self, writer: &mut W) -> Result<()> {
        writer.write_u32::<LE>(self.0 as u32)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NameIndexFlagged(pub usize, pub Option<u32>);
impl<R: Read> Readable<R> for NameIndexFlagged {
    fn read(reader: &mut R) -> Result<Self> {
        let n = reader.read_u32::<LE>()?;
        Ok(if n & 0x80000000 != 0 {
            let flag = Some(reader.read_u32::<LE>()?);
            NameIndexFlagged((n << 1 >> 1) as usize, flag)
        } else {
            NameIndexFlagged(n as usize, None)
        })
    }
}
impl<W: Write> Writable<W> for NameIndexFlagged {
    fn write(&self, writer: &mut W) -> Result<()> {
        if let Some(flag) = self.1 {
            writer.write_u32::<LE>(self.0 as u32 | 0x80000000)?;
            writer.write_u32::<LE>(flag)?;
        } else {
            writer.write_u32::<LE>(self.0 as u32)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Asset {
    pub object_path: NameIndexFlagged,
    pub package_path: NameIndexFlagged,
    pub asset_class: NameIndexFlagged,
}
impl<R: Read> Readable<R> for Asset {
    fn read(reader: &mut R) -> Result<Self> {
        Ok(Asset {
            object_path: NameIndexFlagged::read(reader)?,
            package_path: NameIndexFlagged::read(reader)?,
            asset_class: NameIndexFlagged::read(reader)?,
        })
    }
}
impl<W: Write> Writable<W> for Asset {
    fn write(&self, writer: &mut W) -> Result<()> {
        self.object_path.write(writer)?;
        self.package_path.write(writer)?;
        self.asset_class.write(writer)?;
        Ok(())
    }
}

fn read_array<R, T, E>(
    length: u32,
    reader: &mut R,
    f: fn(&mut R) -> Result<T, E>,
) -> Result<Vec<T>, E> {
    (0..length).map(|_| f(reader)).collect()
}
fn write_array<W, T>(
    writer: &mut W,
    array: &Vec<T>,
    f: fn(&mut W, item: &T) -> Result<()>,
) -> Result<()> {
    for item in array {
        f(writer, item)?;
    }
    Ok(())
}

#[derive(Debug)]
pub struct Pair {
    pub name: NameIndex,
    pub type_: Type,
    pub index: u32,
}

impl<R: Read> Readable<R> for Pair {
    fn read(reader: &mut R) -> Result<Self> {
        let name = NameIndex::read(reader)?;
        let n = reader.read_u32::<LE>()?;
        let type_: Type = (n >> 29).try_into()?;
        let index = n << 3 >> 3;
        Ok(Pair { name, type_, index })
    }
}
impl<W: Write> Writable<W> for Pair {
    fn write(&self, writer: &mut W) -> Result<()> {
        self.name.write(writer)?;
        writer.write_u32::<LE>((self.type_ as u32) << 29 | self.index)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Type {
    AnsiString = 0,
    WideString = 1,
    NumberlessName = 2,
    Name = 3,
    NumberlessExportPath = 4,
    ExportPath = 5,
    LocalizedText = 6,
}
impl TryFrom<u32> for Type {
    type Error = anyhow::Error;

    fn try_from(v: u32) -> Result<Self> {
        match v {
            x if x == Type::AnsiString as u32 => Ok(Type::AnsiString),
            x if x == Type::WideString as u32 => Ok(Type::WideString),
            x if x == Type::NumberlessName as u32 => Ok(Type::NumberlessName),
            x if x == Type::Name as u32 => Ok(Type::Name),
            x if x == Type::NumberlessExportPath as u32 => Ok(Type::NumberlessExportPath),
            x if x == Type::ExportPath as u32 => Ok(Type::ExportPath),
            x if x == Type::LocalizedText as u32 => Ok(Type::LocalizedText),
            _ => Err(anyhow!("invalid AssetRegistry type: {v}")),
        }
    }
}

#[derive(Debug)]
pub struct AssetData {
    pub object_path: NameIndexFlagged,
    pub package_path: NameIndexFlagged,
    pub asset_class: NameIndexFlagged,
    pub package_name: NameIndexFlagged,
    pub asset_name: NameIndexFlagged,
    pub tag_meta: u64,
    pub bundle_count: u32,
    pub chunk_ids: Vec<u32>,
    pub flags: u32,
}

impl<R: Read> Readable<R> for AssetData {
    fn read(reader: &mut R) -> Result<Self> {
        Ok(AssetData {
            object_path: NameIndexFlagged::read(reader)?,
            package_path: NameIndexFlagged::read(reader)?,
            asset_class: NameIndexFlagged::read(reader)?,
            package_name: NameIndexFlagged::read(reader)?,
            asset_name: NameIndexFlagged::read(reader)?,
            tag_meta: reader.read_u64::<LE>()?,
            bundle_count: reader.read_u32::<LE>()?,
            chunk_ids: read_array(reader.read_u32::<LE>()?, reader, R::read_u32::<LE>)?,
            flags: reader.read_u32::<LE>()?,
        })
    }
}
impl<W: Write> Writable<W> for AssetData {
    fn write(&self, writer: &mut W) -> Result<()> {
        self.object_path.write(writer)?;
        self.package_path.write(writer)?;
        self.asset_class.write(writer)?;
        self.package_name.write(writer)?;
        self.asset_name.write(writer)?;
        writer.write_u64::<LE>(self.tag_meta)?;
        writer.write_u32::<LE>(self.bundle_count)?;
        writer.write_u32::<LE>(self.chunk_ids.len() as u32)?;
        for c in self.chunk_ids.iter() {
            writer.write_u32::<LE>(*c)?;
        }
        writer.write_u32::<LE>(self.flags)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Dependencies {
    pub dependencies_size: u64,
    pub dependencies: Vec<u32>,
    pub package_data_buffer_size: u32,
}
impl<R: Read> Readable<R> for Dependencies {
    fn read(reader: &mut R) -> Result<Self> {
        Ok(Dependencies {
            dependencies_size: reader.read_u64::<LE>()?,
            dependencies: read_array(reader.read_u32::<LE>()?, reader, R::read_u32::<LE>)?,
            package_data_buffer_size: reader.read_u32::<LE>()?,
        })
    }
}
impl<W: Write> Writable<W> for Dependencies {
    fn write(&self, writer: &mut W) -> Result<()> {
        writer.write_u64::<LE>(self.dependencies_size)?;
        writer.write_u32::<LE>(self.dependencies.len() as u32)?;
        write_array(
            writer,
            &self.dependencies,
            |w, i| Ok(w.write_u32::<LE>(*i)?),
        )?;
        writer.write_u32::<LE>(self.package_data_buffer_size)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct AssetRegistry {
    pub version: Guid,
    pub unknown1: u32,
    pub unknown2: u32,
    pub unknown3: u16,
    pub unknown4: u16,
    pub unknown5: u32,
    pub idk: Vec<u64>,
    pub names: Vec<String>,
    pub magic_start: u32,
    pub unknown7: u32,
    pub unknown8: u32,
    pub unknown9: u32,
    pub unknown10: u32,
    pub texts: Vec<String>,
    pub numberless_names: Vec<NameIndexFlagged>,
    pub names_idk: Vec<NameIndexFlagged>,
    pub assets: Vec<Asset>,
    pub export_paths: Vec<Asset>,
    pub ansi_strings: Vec<String>,
    pub wide_strings: Vec<String>,
    pub pairs: Vec<Pair>,
    pub magic_end: u32,
    pub asset_data: Vec<AssetData>,
    pub dependencies: Dependencies,
}

impl<R: Read> Readable<R> for AssetRegistry {
    fn read(reader: &mut R) -> Result<Self> {
        let version = Guid::read(reader)?;

        let unknown1 = reader.read_u32::<LE>()?;
        let name_count = reader.read_u32::<LE>()?;
        let unknown2 = reader.read_u32::<LE>()?;
        let unknown3 = reader.read_u16::<LE>()?;
        let unknown4 = reader.read_u16::<LE>()?;
        let unknown5 = reader.read_u32::<LE>()?;

        let idk = read_array(name_count, reader, R::read_u64::<LE>)?;
        let name_lengths = read_array(name_count, reader, R::read_u16::<BE>)?;

        let names = name_lengths
            .into_iter()
            .map(|l| -> Result<String> {
                let mut chars = vec![0; l as usize];
                reader.read_exact(&mut chars)?;
                Ok(String::from_utf8_lossy(&chars).into_owned())
            })
            .collect::<Result<Vec<String>, _>>()?;

        let magic_start = reader.read_u32::<LE>()?;
        assert_eq!(magic_start, 0x12345679);

        let numberless_names_count = reader.read_u32::<LE>()?;
        let names_idk_count = reader.read_u32::<LE>()?;
        let asset_count = reader.read_u32::<LE>()?;
        let export_paths_count = reader.read_u32::<LE>()?;
        let texts_count = reader.read_u32::<LE>()?;
        let ansi_strings_count = reader.read_u32::<LE>()?;
        let wide_strings_count = reader.read_u32::<LE>()?;
        let unknown7 = reader.read_u32::<LE>()?;
        let unknown8 = reader.read_u32::<LE>()?;
        let pair_count = reader.read_u32::<LE>()?;
        let unknown9 = reader.read_u32::<LE>()?;

        let unknown10 = reader.read_u32::<LE>()?;
        let texts = read_array(texts_count, reader, |r| -> Result<String> {
            let mut chars = vec![0; r.read_u32::<LE>()? as usize - 1];
            r.read_exact(&mut chars)?;
            r.read_u8()?;
            Ok(String::from_utf8_lossy(&chars).into_owned())
        })?;

        let numberless_names = read_array(numberless_names_count, reader, NameIndexFlagged::read)?;
        let names_idk = read_array(names_idk_count, reader, NameIndexFlagged::read)?;

        let assets = read_array(asset_count, reader, Asset::read)?;
        let export_paths = read_array(export_paths_count, reader, Asset::read)?;

        let _ansi_string_offsets = read_array(ansi_strings_count, reader, R::read_u32::<LE>)?;
        let _wide_string_offets = read_array(wide_strings_count, reader, R::read_u32::<LE>)?;

        let ansi_strings = read_array(ansi_strings_count, reader, |r| -> Result<String> {
            let mut chars = vec![];
            loop {
                let next = r.read_u8()?;
                if next == 0 {
                    break;
                }
                chars.push(next);
            }
            Ok(String::from_utf8_lossy(&chars).into_owned())
        })?;

        let wide_strings = read_array(wide_strings_count, reader, |reader| -> Result<String> {
            let mut chars = vec![];
            loop {
                let next = reader.read_u16::<LE>()?;
                if next == 0 {
                    break;
                }
                chars.push(char::from_u32(next.into()).unwrap_or(char::REPLACEMENT_CHARACTER));
            }
            Ok(chars.iter().collect::<String>())
        })?;

        let pairs = read_array(pair_count, reader, Pair::read)?;

        let magic_end = reader.read_u32::<LE>()?;
        assert_eq!(magic_end, 0x87654321);

        let asset_data = read_array(reader.read_u32::<LE>()?, reader, AssetData::read)?;

        let dependencies = Dependencies::read(reader)?;
        Ok(AssetRegistry {
            version,
            unknown1,
            unknown2,
            unknown3,
            unknown4,
            unknown5,
            idk,
            names,
            magic_start,
            unknown7,
            unknown8,
            unknown9,
            unknown10,
            texts,
            numberless_names,
            names_idk,
            assets,
            export_paths,
            ansi_strings,
            wide_strings,
            pairs,
            magic_end,
            asset_data,
            dependencies,
        })
    }
}
impl<W: Write> Writable<W> for AssetRegistry {
    fn write(&self, writer: &mut W) -> Result<()> {
        self.version.write(writer)?;

        writer.write_u32::<LE>(self.unknown1)?;
        writer.write_u32::<LE>(self.names.len() as u32)?;
        writer.write_u32::<LE>(self.unknown2)?;
        writer.write_u16::<LE>(self.unknown3)?;
        writer.write_u16::<LE>(self.unknown4)?;
        writer.write_u32::<LE>(self.unknown5)?;

        write_array(writer, &self.idk, |w, i| Ok(w.write_u64::<LE>(*i)?))?;
        write_array(writer, &self.names, |w, i| {
            Ok(w.write_u16::<BE>(i.as_bytes().len() as u16)?)
        })?;

        write_array(writer, &self.names, |w, i| {
            w.write_all(i.as_bytes())?;
            Ok(())
        })?;

        writer.write_u32::<LE>(self.magic_start)?;

        writer.write_u32::<LE>(self.numberless_names.len() as u32)?;
        writer.write_u32::<LE>(self.names_idk.len() as u32)?;
        writer.write_u32::<LE>(self.assets.len() as u32)?;
        writer.write_u32::<LE>(self.export_paths.len() as u32)?;
        writer.write_u32::<LE>(self.texts.len() as u32)?;
        writer.write_u32::<LE>(self.ansi_strings.len() as u32)?;
        writer.write_u32::<LE>(self.wide_strings.len() as u32)?;
        writer.write_u32::<LE>(self.unknown7)?;
        writer.write_u32::<LE>(self.unknown8)?;
        writer.write_u32::<LE>(self.pairs.len() as u32)?;
        writer.write_u32::<LE>(self.unknown9)?;
        writer.write_u32::<LE>(self.unknown10)?;

        write_array(writer, &self.texts, |w, i| {
            w.write_u32::<LE>(i.as_bytes().len() as u32 + 1)?;
            w.write_all(i.as_bytes())?;
            w.write_u8(0)?;
            Ok(())
        })?;

        write_array(writer, &self.numberless_names, |w, i| i.write(w))?;
        write_array(writer, &self.names_idk, |w, i| i.write(w))?;
        write_array(writer, &self.assets, |w, i| i.write(w))?;
        write_array(writer, &self.export_paths, |w, i| i.write(w))?;

        let mut offset = 0;
        for i in &self.ansi_strings {
            writer.write_u32::<LE>(offset)?;
            offset += i.as_bytes().len() as u32 + 1;
        }

        let mut offset = 0;
        for i in &self.wide_strings {
            writer.write_u32::<LE>(offset)?;
            offset += i.chars().count() as u32 + 1;
        }

        write_array(writer, &self.ansi_strings, |w, i| {
            w.write_all(i.as_bytes())?;
            w.write_u8(0)?;
            Ok(())
        })?;

        write_array(writer, &self.wide_strings, |w, i| {
            for c in i.chars() {
                w.write_u16::<LE>(c as u16)?;
            }
            w.write_u16::<LE>(0)?;
            Ok(())
        })?;

        write_array(writer, &self.pairs, |w, i| i.write(w))?;

        writer.write_u32::<LE>(self.magic_end)?;

        writer.write_u32::<LE>(self.asset_data.len() as u32)?;
        write_array(writer, &self.asset_data, |w, i| i.write(w))?;

        self.dependencies.write(writer)?;

        Ok(())
    }
}
