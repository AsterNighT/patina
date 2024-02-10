use std::process::Command;
#[cfg(not(debug_assertions))]
use std::process::Stdio;
#[cfg(not(debug_assertions))]
use std::os::windows::process::CommandExt;
use anyhow::Result;

// pub type DWORD = cty::c_ulong;

// #[link(name = "winring0", kind = "static")]
// extern "stdcall" {
//     /// Read Model Specific Register
//     /// EDX:EAX = MSR[index]
//     fn Rdmsr(index: DWORD, eax: *mut DWORD, edx: *mut DWORD) -> bool;
//     /// Write Model Specific Register
//     /// EDX:EAX = MSR[index]
//     fn Wrmsr(index: DWORD, eax: DWORD, edx: DWORD) -> bool;
//     fn HelloWorld(id:DWORD) -> DWORD;
//     fn winring0_init() -> cty::c_int;
//     fn winring0_deinit() -> cty::c_int;
// }

// pub fn hello_world(id: DWORD) -> DWORD {
//     unsafe { HelloWorld(id) }
// }

// pub fn initiliaze_winring0() -> Result<()> {
//     let error = unsafe { winring0_init() };
//     if error == 0 {
//         Ok(())
//     } else {
//         Err(anyhow::anyhow!(
//             "Failed to initialize WinRing0, dll status {}",
//             error
//         ))
//     }
// }

// pub fn deinitiliaze_winring0() -> Result<()> {
//     let error = unsafe { winring0_deinit() };
//     if error == 0 {
//         Ok(())
//     } else {
//         Err(anyhow::anyhow!(
//             "Failed to deinitialize WinRing0, dll status {}",
//             error
//         ))
//     }
// }

// pub fn rdmsr(index: DWORD) -> Result<u64> {
//     let mut eax: DWORD = 0;
//     let mut edx: DWORD = 0;
//     if unsafe { Rdmsr(index, &mut eax, &mut edx) } {
//         Ok(((edx as u64) << 32) | eax as u64)
//     } else {
//         Err(anyhow::anyhow!("Failed to read msr"))
//     }
// }

// pub fn wrmsr(index: DWORD, value: u64) -> Result<()> {
//     let eax = value as DWORD;
//     let edx = (value >> 32) as DWORD;
//     if unsafe { Wrmsr(index, eax, edx) } {
//         Ok(())
//     } else {
//         Err(anyhow::anyhow!("Failed to write msr"))
//     }
// }

/// Anything above got stuck in sc manager, no luck.
/// No idea how msr-utility did it, use it instead.
#[cfg(not(debug_assertions))]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn wrmsr(index: u32, value: u64) -> Result<()> {
    let mut command = Command::new("msr-cmd.exe");
    command
        .arg("-l")
        .arg("write")
        .arg(format!("0x{:x}", index))
        .arg(format!("0x{:016x}", value));
    println!("{:?}", command);
    #[cfg(not(debug_assertions))]
    command.stdout(Stdio::null()).stderr(Stdio::null()).creation_flags(CREATE_NO_WINDOW);
    let ret = command.status()?;
    if ret.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Failed to write msr: return code {}", ret))
    }
}

pub fn rdmsr(index: u32) -> Result<u64> {
    let mut command = Command::new("msr-cmd.exe");
    command
        .arg("-ld")
        .arg("read")
        .arg(format!("0x{:x}", index));
    #[cfg(not(debug_assertions))]
    command.stderr(Stdio::null()).creation_flags(CREATE_NO_WINDOW);
    let output = command.output()?;
    if output.status.success() {
        let output = String::from_utf8(output.stdout)?;
        let output = output.trim();
        let output = output.split_whitespace().last().unwrap();
        let output = u64::from_str_radix(&output[2..output.len()], 16)?;
        Ok(output)
    } else {
        Err(anyhow::anyhow!("Failed to read msr: return code {}", output.status))
    }
}
