use std::{convert::TryFrom, os::windows::process::CommandExt, path::Path, process::Command};

#[derive(Debug)]
pub enum InjectHelperError {
    WriteBinaryToFile(std::io::Error),
    ExecuteBinary(std::io::Error),
    InjectError(ExitStatus),
}

#[derive(Debug)]
pub enum ExitStatus {
    InjectFailed,
    InvalidParams,
    OpenProcessFail,
    UnlikelyFail,
    Unknown(i32),
}

impl TryFrom<i32> for ExitStatus {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            -1 => Ok(Self::InjectFailed),
            -2 => Ok(Self::InvalidParams),
            -3 => Ok(Self::OpenProcessFail),
            -4 => Ok(Self::UnlikelyFail),
            _ => Err(()),
        }
    }
}

/// Tries to inject the graphics hook into the specified process.
pub fn inject_graphics_hook(pid: u32, anti_cheat_compatible: bool) -> Result<(), InjectHelperError> {
    // Write the binaries to disk
    //
    if !Path::new("inject-helper.exe").exists() {
        std::fs::write("inject-helper.exe", include_bytes!("../bin/inject-helper64.exe"))
            .map_err(InjectHelperError::WriteBinaryToFile)?;
    }

    if !Path::new("graphics-hook64.dll").exists() {
        std::fs::write("graphics-hook64.dll", include_bytes!("../bin/graphics-hook64.dll"))
            .map_err(InjectHelperError::WriteBinaryToFile)?;
    }

    // Run the injector
    //
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    // const DETACHED_PROCESS: u32 = 0x00000008;

    let exit_status = Command::new("inject-helper.exe")
        .args(&[
            "graphics-hook64.dll",
            (anti_cheat_compatible as u8).to_string().as_str(),
            pid.to_string().as_str(),
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(InjectHelperError::ExecuteBinary)?;

    if exit_status.success() {
        Ok(())
    } else {
        Err(InjectHelperError::InjectError(
            exit_status
                .code()
                .map(|code| ExitStatus::try_from(code).unwrap_or(ExitStatus::Unknown(code)))
                .unwrap_or(ExitStatus::Unknown(0)),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject() {
        let result = inject_graphics_hook(std::process::id(), false);
        println!("{:?}", result);
    }
}
