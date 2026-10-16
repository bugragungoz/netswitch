//! Program paths and descriptions of processes.

use windows::core::{HSTRING, PCWSTR, PWSTR};
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};

/// The full path of a process's program, or None for the System process, a process that ended,
/// or one this user may not read.
pub fn program_path(pid: u32) -> Option<String> {
    if pid == 0 || pid == 4 {
        return None;
    }
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buffer = [0u16; 1024];
        let mut len = buffer.len() as u32;
        let result = QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, PWSTR(buffer.as_mut_ptr()), &mut len);
        let _ = CloseHandle(handle);
        result.ok()?;
        Some(String::from_utf16_lossy(&buffer[..len as usize]))
    }
}

/// The FileDescription of a program's version resource ("Google Chrome"), if it has one.
pub fn file_description(path: &str) -> Option<String> {
    let wide = HSTRING::from(path);
    unsafe {
        let size = GetFileVersionInfoSizeW(PCWSTR(wide.as_ptr()), None);
        if size == 0 {
            return None;
        }
        let mut data = vec![0u8; size as usize];
        GetFileVersionInfoW(PCWSTR(wide.as_ptr()), None, size, data.as_mut_ptr().cast()).ok()?;

        // The first language and code page the resource lists, then the usual English fallback.
        let mut languages = Vec::new();
        let mut ptr = std::ptr::null_mut();
        let mut len = 0u32;
        let translation = HSTRING::from("\\VarFileInfo\\Translation");
        if VerQueryValueW(data.as_ptr().cast(), PCWSTR(translation.as_ptr()), &mut ptr, &mut len).as_bool() && len >= 4
        {
            let pairs = std::slice::from_raw_parts(ptr as *const u16, (len / 2) as usize);
            for [language, code_page] in pairs.as_chunks::<2>().0 {
                languages.push(format!("{language:04x}{code_page:04x}"));
            }
        }
        languages.push("040904b0".to_string());
        languages.push("040904e4".to_string());

        for language in languages {
            let key = HSTRING::from(format!("\\StringFileInfo\\{language}\\FileDescription"));
            let mut value = std::ptr::null_mut();
            let mut chars = 0u32;
            if VerQueryValueW(data.as_ptr().cast(), PCWSTR(key.as_ptr()), &mut value, &mut chars).as_bool() && chars > 1
            {
                let text =
                    String::from_utf16_lossy(std::slice::from_raw_parts(value as *const u16, chars as usize - 1));
                let text = text.trim_matches(char::from(0)).trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
        None
    }
}

/// True when this process runs with administrator rights (an elevated token).
pub fn is_elevated() -> bool {
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION::default();
        let mut len = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut len,
        )
        .is_ok();
        let _ = CloseHandle(token);
        ok && elevation.TokenIsElevated != 0
    }
}
