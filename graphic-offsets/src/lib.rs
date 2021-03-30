use serde::Deserialize;
use std::{os::windows::process::CommandExt, path::Path, process::Command};

#[derive(Debug)]
pub enum GraphicOffsetsError {
    WriteBinaryToFile(std::io::Error),
    ExecuteBinary(std::io::Error),
    ParseOutput(toml::de::Error),
}

#[doc(hidden)]
#[repr(C)]
#[derive(Deserialize)]
pub struct ParsedGraphicOffsets {
    pub d3d8: D3D8,
    pub d3d9: D3D9,
    pub dxgi: DXGI,
}

#[repr(C)]
#[derive(Deserialize, Default, Debug)]
pub struct GraphicOffsets {
    pub d3d8: D3D8,
    pub d3d9: D3D9,
    pub dxgi: DXGI,
    pub ddraw: DDraw,
}

#[repr(C)]
#[derive(Deserialize, Default, Debug)]
pub struct D3D8 {
    pub present: u32,
}

#[repr(C)]
#[derive(Deserialize, Default, Debug)]
pub struct D3D9 {
    pub present: u32,
    pub present_ex: u32,
    pub present_swap: u32,
    pub d3d9_clsoff: u32,
    pub is_d3d9ex_clsoff: u32,
}

#[allow(clippy::upper_case_acronyms)]
#[repr(C)]
#[derive(Deserialize, Default, Debug)]
pub struct DXGI {
    pub present: u32,
    pub present1: u32,
    pub resize: u32,
}

#[repr(C)]
#[derive(Deserialize, Default, Debug)]
pub struct DDraw {
    pub surface_create: u32,
    pub surface_restore: u32,
    pub surface_release: u32,
    pub surface_unlock: u32,
    pub surface_blt: u32,
    pub surface_flip: u32,
    pub surface_set_palette: u32,
    pub palette_set_entries: u32,
}

/// Loads the graphic offsets and returns them.
///
/// # How this works.
///
/// TODO: Explain how it's done.
pub fn load_graphic_offsets() -> Result<GraphicOffsets, GraphicOffsetsError> {
    // Write the binary to the file
    //
    if !Path::new("get-graphic-offsets.exe").exists() {
        std::fs::write(
            "get-graphic-offsets.exe",
            include_bytes!("../bin/get-graphics-offsets64.exe"),
        )
        .map_err(GraphicOffsetsError::WriteBinaryToFile)?;
    }

    // Execute the binary
    //
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    // const DETACHED_PROCESS: u32 = 0x00000008;

    let output = Command::new("./get-graphic-offsets.exe")
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(GraphicOffsetsError::ExecuteBinary)?;

    // Parse the output. We need to do this with a separate structure, because the
    // sizes need to match. Wrapping the `ddraw` with an Option, will add 4 more
    // bytes that we don't need.
    //
    let parsed = toml::from_str::<ParsedGraphicOffsets>(&*String::from_utf8_lossy(&*output.stdout))
        .map_err(GraphicOffsetsError::ParseOutput)?;

    Ok(GraphicOffsets {
        d3d8: parsed.d3d8,
        d3d9: parsed.d3d9,
        dxgi: parsed.dxgi,
        ddraw: Default::default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sizes() {
        assert_eq!(core::mem::size_of::<D3D8>(), 4);
        assert_eq!(core::mem::size_of::<D3D9>(), 20);
        assert_eq!(core::mem::size_of::<DXGI>(), 12);
        assert_eq!(core::mem::size_of::<DDraw>(), 32);
        assert_eq!(core::mem::size_of::<GraphicOffsets>(), 68);
    }

    #[test]
    fn test_load() {
        assert!(load_graphic_offsets().is_ok());
    }
}
