use shared::CompState;
use std::{
    collections::HashMap,
    fs,
    io,
    path::Path,
};

pub fn load(path: &Path) -> io::Result<HashMap<u8, CompState>> {
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let data = fs::read_to_string(path)?;
    let map: HashMap<u8, CompState> = serde_json::from_str(&data)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(map)
}

pub fn save(path: &Path, map: &HashMap<u8, CompState>) -> io::Result<()> {
    let json = serde_json::to_string_pretty(map)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let tmp = path.with_file_name(format!(
        "{}.tmp",
        path.file_name().unwrap().to_string_lossy()
    ));
    fs::write(&tmp, json)?;
    fs::rename(&tmp, path)?;
    Ok(())
}