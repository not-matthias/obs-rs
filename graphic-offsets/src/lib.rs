use serde::Deserialize;
use std::process::Command;

pub enum GraphicOffsetsError {
    WriteBinaryToFile(std::io::Error),
    ExecuteBinary(std::io::Error),
    ParseOutput(toml::de::Error),
}

#[derive(Deserialize)]
pub struct GraphicOffsets {
    pub d3d8: D3D8,
    pub d3d9: D3D9,
    pub dxgi: DXGI,
}

#[derive(Deserialize)]
pub struct D3D8 {
    pub present: u64,
}

#[derive(Deserialize)]
pub struct D3D9 {
    pub present: u64,
    pub present_ex: u64,
    pub present_swap: u64,
    pub d3d9_clsoff: u64,
    pub is_d3d9ex_clsoff: u64,
}

#[derive(Deserialize)]
pub struct DXGI {
    pub present: u64,
    pub present1: u64,
    pub resize: u64,
}

/// Loads the graphic offsets and returns them.
///
/// # How this works.
///
/// TODO: Explain how it's done.
pub fn load_graphic_offsets() -> Result<GraphicOffsets, GraphicOffsetsError> {
    // Write the binary to the file
    //
    std::fs::write(
        "./get-graphic-offsets.exe",
        include_bytes!("../bin/get-graphics-offsets64.exe"),
    )
    .map_err(|e| GraphicOffsetsError::WriteBinaryToFile(e))?;

    // Execute the binary
    //
    let output = Command::new("./get-graphic-offsets.exe")
        .output()
        .map_err(|e| GraphicOffsetsError::ExecuteBinary(e))?;

    // Parse the output
    //
    toml::from_str(&*String::from_utf8_lossy(&*output.stdout))
        .map_err(|e| GraphicOffsetsError::ParseOutput(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load() {
        assert!(load_graphic_offsets().is_ok());
    }
}
