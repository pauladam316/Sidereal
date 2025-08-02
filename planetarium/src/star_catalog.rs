use std::{
    fs::File,
    io::{self, BufReader, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};
use byteorder::{LittleEndian, ReadBytesExt};

/// Epoch flag inferred if either `starn` or `nmag` is negative.
#[derive(Debug, Clone, Copy)]
pub enum Epoch {
    B1950,
    J2000,
}

#[derive(Debug)]
pub struct CatalogHeader {
    pub star0: i32,
    pub star1: i32,
    pub starn: i32,
    pub stnum: i32,
    pub mprop: i32,
    pub nmag: i32,
    pub nbent: i32,
    pub epoch: Epoch,
}

#[derive(Debug)]
pub enum StarId {
    /// stored as a f32
    Real(f32),
    /// stored as an i32
    Integer(i32),
}

/// one star entry
#[derive(Debug)]
pub struct StarEntry {
    pub sequence: i32,           // = raw_id - star0
    pub id: Option<StarId>,      // None if stnum == 0 or stnum < 0
    pub ra: f64,                 // radians
    pub dec: f64,                // radians
    pub spectral_type: String,   // 2‐char ASCII
    pub magnitudes: Vec<f32>,    // each = raw_mag / 100.0
    pub proper_motion_ra:  Option<f32>,
    pub proper_motion_dec: Option<f32>,
    pub radial_velocity:     Option<f64>,
    pub name:                Option<String>, // only if stnum < 0
}

pub fn parse_catalog(path: PathBuf) -> io::Result<(CatalogHeader, Vec<StarEntry>)> {
    let f = File::open(path)?;
    let mut reader = BufReader::new(f);

    // 1) Read header
    let star0 = reader.read_i32::<LittleEndian>()?;
    let star1 = reader.read_i32::<LittleEndian>()?;
    let starn = reader.read_i32::<LittleEndian>()?;
    let stnum = reader.read_i32::<LittleEndian>()?;
    let mprop = reader.read_i32::<LittleEndian>()?;
    let nmag_raw = reader.read_i32::<LittleEndian>()?;
    let nbent = reader.read_i32::<LittleEndian>()?;

    let epoch = if starn < 0 || nmag_raw < 0 {
        Epoch::J2000
    } else {
        Epoch::B1950
    };
    let nmag = nmag_raw.abs();
    let star_count = starn.abs() as usize;

    let header = CatalogHeader {
        star0,
        star1,
        starn,
        stnum,
        mprop,
        nmag,
        nbent,
        epoch,
    };

    // 2) Read each star
    let mut stars = Vec::with_capacity(star_count);
    for _ in 0..star_count {
        stars.push(read_star(&mut reader, &header)?);
    }

    Ok((header, stars))
}

fn read_star<R: Read + Seek>(
    reader: &mut R,
    hdr: &CatalogHeader,
) -> io::Result<StarEntry> {
    // mark start so we can skip to exactly nbent bytes
    let start = reader.stream_position()?;

    // 1) ID field (optional)
    let raw_id = match hdr.stnum {
        0 => None,
        n if n > 0 && n != 4 => {
            // Real*4 catalog number
            Some(StarId::Real(reader.read_f32::<LittleEndian>()?))
        }
        4 => {
            // Integer*4 catalog number
            Some(StarId::Integer(reader.read_i32::<LittleEndian>()?))
        }
        _ => None, // <0 means no ID, name will follow later
    };

    // for sequence number:
    let seq = raw_id.as_ref().map_or(0, |id| match id {
        StarId::Real(f) => (*f as i32) - hdr.star0,
        StarId::Integer(i) => *i - hdr.star0,
    });

    // 2) coordinates
    let ra  = reader.read_f64::<LittleEndian>()?;
    let dec = reader.read_f64::<LittleEndian>()?;

    // 3) spectral type (2 ASCII chars)
    let mut sp_buf = [0u8; 2];
    reader.read_exact(&mut sp_buf)?;
    let spectral_type = String::from_utf8_lossy(&sp_buf).into_owned();

    // 4) magnitudes
    let mut magnitudes = Vec::with_capacity(hdr.nmag as usize);
    for _ in 0..hdr.nmag {
        // stored as Integer*2 = i16; value = mag * 100
        let raw = reader.read_i16::<LittleEndian>()?;
        magnitudes.push(raw as f32 / 100.0);
    }

    // 5) proper motions?
    let (pm_ra, pm_dec) = if hdr.mprop >= 1 {
        let xrpm = reader.read_f32::<LittleEndian>()?;
        let xdpm = reader.read_f32::<LittleEndian>()?;
        (Some(xrpm), Some(xdpm))
    } else {
        (None, None)
    };

    // 6) radial velocity?
    let vel = if hdr.mprop >= 2 {
        Some(reader.read_f64::<LittleEndian>()?)
    } else {
        None
    };

    // 7) object name if stnum < 0
    let name = if hdr.stnum < 0 {
        let name_len = (-hdr.stnum) as usize;
        let mut buf = vec![0u8; name_len];
        reader.read_exact(&mut buf)?;
        Some(String::from_utf8_lossy(&buf).trim_end_matches('\0').to_string())
    } else {
        None
    };

    // 8) skip any padding so we end up exactly nbent bytes past `start`
    let end = reader.stream_position()?;
    let read_bytes = (end - start) as i64;
    let to_skip = hdr.nbent as i64 - read_bytes;
    if to_skip > 0 {
        reader.seek(SeekFrom::Current(to_skip))?;
    }

    Ok(StarEntry {
        sequence:         seq,
        id:               raw_id,
        ra,
        dec,
        spectral_type,
        magnitudes,
        proper_motion_ra: pm_ra,
        proper_motion_dec: pm_dec,
        radial_velocity:   vel,
        name,
    })
}
