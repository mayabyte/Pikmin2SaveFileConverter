mod treasures;

use std::{convert::TryInto, fs::{File, OpenOptions}, io::{Read, Write}, path::{Path, PathBuf}, str::FromStr};
use itertools::Itertools;
use structopt::StructOpt;
use anyhow::{Result, Error, Context, anyhow};
use treasures::TREASURE_VALUES;

#[derive(Debug, StructOpt)]
#[structopt(name="p2saveconvert", about="Pikmin 2 save file converter")]
struct Args {
    #[structopt(parse(from_os_str))]
    input_file: PathBuf,

    #[structopt(
        parse(from_os_str),
        help = "Where to save the converted save file. If not supplied, it will be saved in the same directory as the input file \
                with the region prepended to the filename."
    )]
    output_file: Option<PathBuf>,

    #[structopt(
        short="r", long="set-region", parse(try_from_str),
        help = "Region to convert the provided save file to."
    )]
    region: Region,

    #[structopt(
        short="p", long="set-pokos", parse(try_from_str),
        help = "Manually set the Poko values for each of the three save files. If not supplied, the poko values will be \
                calculated automatically based on which treasures have been collected. Must supply exactly 3 values."
    )]
    set_pokos: Option<Vec<u32>>
}

fn main() -> Result<()> {
    let args = Args::from_args();
    if let Some(pokos) = &args.set_pokos {
        if pokos.len() != 3 {
            return Err(anyhow!("Must supply exactly 3 values if using --set-pokos."));
        }
    }

    let mut save_file = SaveFile::read(args.input_file.as_path())?;
    if let Some(manual_poko_counts) = &args.set_pokos {
        save_file.set_pokos_manually(&manual_poko_counts);
    }
    else {
        save_file.recalculate_pokos(&args.region)?;
    }
    save_file.set_region(&args.region)?;

    let output_filename = args.output_file.clone().unwrap_or_else(
        || args.input_file.with_file_name(
            format!("{}-{:?}.gci", &args.input_file.file_stem().unwrap().to_string_lossy(), &args.region)
        )
    );
    save_file.write(output_filename.as_path())
}

const SAVE_FILE_LEN_BYTES: usize = 221248;
const FILE_HEADER_MAGIC_STR: &'static [u8; 8] = b"PlVa0003";
const TREASURE_LIST_OFFSET: usize = 0x4CF;
const TREASURE_LIST_LEN: usize = 188;
const ACTUAL_POKO_COUNT_OFFSET: usize = 0x834;
const DISPLAY_POKO_COUNT_OFFSET: usize = 0x2C;
struct SaveFile{
    bytes: [u8; SAVE_FILE_LEN_BYTES]
}

impl SaveFile {
    pub fn read(path: &Path) -> Result<Self> {
        let mut bytes = [0u8; SAVE_FILE_LEN_BYTES];
        let read_bytes = File::open(path)
            .with_context(|| "Couldn't read file at the specified path.")?
            .read(&mut bytes)
            .with_context(|| "Couldn't read file at the specified path.")?;

        if read_bytes != SAVE_FILE_LEN_BYTES {
            Err(anyhow!("Invalid save file supplied (wrong length)."))
        }
        else {
            Ok(Self{ bytes })
        }
    }

    pub fn write(&mut self, path: &Path) -> Result<()> {
        self.recalculate_checksum();

        OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)?
            .write(&self.bytes)?;

        println!("Wrote converted save file to {}", path.to_string_lossy());

        Ok(())
    }

    pub fn recalculate_pokos(&mut self, region: &Region) -> Result<()> {
        for file_num in [0, 1, 2] {
            let original_region = Region::from_byte(self.bytes[3])?;
            if let Some(save_file) = self.save_slot_data(file_num) {
                let mut actual_pokos = i32::from_be_bytes(
                    save_file[ACTUAL_POKO_COUNT_OFFSET .. ACTUAL_POKO_COUNT_OFFSET + 4].try_into()?
                );
                let original_poko_count = actual_pokos;
                let treasures = &save_file[TREASURE_LIST_OFFSET .. TREASURE_LIST_OFFSET + TREASURE_LIST_LEN];

                let original_region_idx = original_region.to_index();
                let new_region_idx = region.to_index();
                for (id, status) in treasures.iter().enumerate() {
                    if *status > 0 {
                        actual_pokos -= TREASURE_VALUES.get(&id).unwrap()[original_region_idx];
                        actual_pokos += TREASURE_VALUES.get(&id).unwrap()[new_region_idx];
                    }
                }

                for i in 0..4 {
                    save_file[ACTUAL_POKO_COUNT_OFFSET + i] = actual_pokos.to_be_bytes()[i];
                    save_file[DISPLAY_POKO_COUNT_OFFSET + i] = actual_pokos.to_be_bytes()[i];
                }
                let new_poko_count = i32::from_be_bytes(
                    save_file[ACTUAL_POKO_COUNT_OFFSET .. ACTUAL_POKO_COUNT_OFFSET + 4].try_into()?
                );

                println!("Recalculated Pokos in slot {}. Old: {}. New: {}", file_num, original_poko_count, new_poko_count);
            }
        }

        Ok(())
    }

    pub fn set_pokos_manually(&mut self, counts: &Vec<u32>) {
        for file_num in [0, 1, 2] {
            if let Some(save_file) = self.save_slot_data(file_num) {
                for i in 0..4 {
                    save_file[ACTUAL_POKO_COUNT_OFFSET + i] = counts[file_num].to_be_bytes()[i];
                }
            }
        }
    }

    pub fn set_region(&mut self, region: &Region) -> Result<()> {
        if self.bytes[3] == region.as_byte() {
            return Err(anyhow!("Save file already contains {:?} save data!", region));
        }
        self.bytes[3] = region.as_byte();
        Ok(())
    }

    fn recalculate_checksum(&mut self) {
        for file_num in 0..=2 {
            if let Some(save_file) = self.save_slot_data(file_num) {
                let save_data = &save_file[..0xBFFC];
                let mut c1: u16 = 0;
                let mut c2: u16 = 0;
                for (b1, b2) in save_data.iter().tuples() {
                    let val = (((*b1 as u32).wrapping_shl(8)).wrapping_add(*b2 as u32)) as u16;
                    c1 = (c1.wrapping_add(val)) & 0xFFFF;
                    c2 = (c2.wrapping_add(val ^ 0xFFFF)) & 0xFFFF;
                }
                if c1 == 0xFFFF {
                    c1 = 0;
                }
                if c2 == 0xFFFF {
                    c2 = 0;
                }

                save_file[0xBFFC+0] = c1.to_be_bytes()[0];
                save_file[0xBFFC+1] = c1.to_be_bytes()[1];
                save_file[0xBFFC+2] = c2.to_be_bytes()[0];
                save_file[0xBFFC+3] = c2.to_be_bytes()[1];
            }
        }
    }

    fn save_slot_data(&mut self, file_num: usize) -> Option<&mut [u8]> {
        for (i, bytes) in self.bytes.windows(9).enumerate() {
            if &bytes[..8] == FILE_HEADER_MAGIC_STR && usize::from(bytes[8]) == file_num {
                return Some(&mut self.bytes[i..i+0xC000]);
            }
        }
        None
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Region {
    US,
    JP,
    PAL
}

impl Region {
    pub fn as_byte(&self) -> u8 {
        match self {
            Region::US => b'E',
            Region::JP => b'J',
            Region::PAL => b'P',
        }
    }

    pub fn to_index(&self) -> usize {
        match self {
            Region::US => 0,
            Region::PAL => 1,
            Region::JP => 2
        }
    }

    pub fn from_byte(byte: u8) -> Result<Self> {
        match byte {
            b'E' => Ok(Region::US),
            b'J' => Ok(Region::JP),
            b'P' => Ok(Region::PAL),
            _ => Err(anyhow!("Invalid region in provided save file!"))
        }
    }
}

impl FromStr for Region {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "j" | "jp" | "jpn" | "ntsc-j" | "ntsc_j" => Ok(Region::JP),
            "u" | "us" | "usa" | "ntsc-u" | "ntsc_u" => Ok(Region::US),
            "p" | "e" | "pal" | "eur" => Ok(Region::PAL),
            _ => Err(anyhow!("Invalid region: {}", s))
        }
    }
}
