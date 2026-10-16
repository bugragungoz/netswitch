//! Windows Firewall through its COM interface (INetFwPolicy2), the same store `wf.msc`,
//! `netsh advfirewall` and PowerShell's NetSecurity cmdlets use.
//!
//! netSwitch only writes and removes block rules named the AppIntBlocker way (see `rules`).

use super::{SysError, SysResult};
use crate::blocklist::FirewallRule;
use crate::rules::{application_name, file_name, rule_name, RuleDirection, PREFIX};
use std::collections::BTreeSet;
use std::path::Path;
use windows::core::{Interface, BSTR};
use windows::Win32::Foundation::{RPC_E_CHANGED_MODE, S_OK, VARIANT_TRUE};
use windows::Win32::NetworkManagement::WindowsFirewall::{
    INetFwPolicy2, INetFwRule, INetFwRules, NetFwPolicy2, NetFwRule, NET_FW_ACTION_BLOCK, NET_FW_PROFILE2_ALL,
    NET_FW_RULE_DIR_IN, NET_FW_RULE_DIR_OUT,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
};
use windows::Win32::System::Ole::IEnumVARIANT;
use windows::Win32::System::Variant::{VariantClear, VARIANT, VT_DISPATCH};

/// What netSwitch writes in a rule's description, so a person reading `wf.msc` knows where it came from.
const DESCRIPTION: &str = "Blocked by netSwitch. Remove it in netSwitch, AppIntBlocker or here.";

/// COM for the current thread, released when dropped.
struct Com {
    uninit: bool,
}

impl Com {
    fn init() -> SysResult<Self> {
        let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if hr == RPC_E_CHANGED_MODE {
            // The thread already runs COM in another mode; it is usable as it is.
            return Ok(Self { uninit: false });
        }
        hr.ok()?;
        Ok(Self { uninit: true })
    }
}

impl Drop for Com {
    fn drop(&mut self) {
        if self.uninit {
            unsafe { CoUninitialize() };
        }
    }
}

fn rules_collection() -> SysResult<INetFwRules> {
    let policy: INetFwPolicy2 = unsafe { CoCreateInstance(&NetFwPolicy2, None, CLSCTX_INPROC_SERVER)? };
    Ok(unsafe { policy.Rules()? })
}

/// Calls `f` for every rule whose name starts with the AppIntBlocker prefix.
fn each_own_rule(rules: &INetFwRules, mut f: impl FnMut(&INetFwRule, String)) -> SysResult<()> {
    let enumerator: IEnumVARIANT = unsafe { rules._NewEnum()? }.cast()?;
    loop {
        let mut items = [VARIANT::default()];
        let mut fetched = 0u32;
        let hr = unsafe { enumerator.Next(&mut items, &mut fetched) };
        if hr != S_OK || fetched == 0 {
            break;
        }
        // SAFETY: Next filled the VARIANT; for VT_DISPATCH the union holds an IDispatch.
        let rule = unsafe {
            let inner = &items[0].Anonymous.Anonymous;
            if inner.vt == VT_DISPATCH {
                (*inner.Anonymous.pdispVal)
                    .as_ref()
                    .and_then(|d| d.cast::<INetFwRule>().ok())
            } else {
                None
            }
        };
        unsafe {
            let _ = VariantClear(&mut items[0]);
        }
        let Some(rule) = rule else { continue };
        let Ok(name) = (unsafe { rule.Name() }) else { continue };
        let name = name.to_string();
        if name.starts_with(PREFIX) {
            f(&rule, name);
        }
    }
    Ok(())
}

/// The AppIntBlocker-named rules, enabled or not, blocking or not.
pub fn read_rules() -> SysResult<Vec<FirewallRule>> {
    let _com = Com::init()?;
    let rules = rules_collection()?;
    let mut out = Vec::new();
    each_own_rule(&rules, |rule, name| unsafe {
        let program = rule
            .ApplicationName()
            .ok()
            .map(|b| b.to_string())
            .filter(|p| !p.is_empty());
        let direction = if rule.Direction().ok() == Some(NET_FW_RULE_DIR_IN) {
            RuleDirection::Inbound
        } else {
            RuleDirection::Outbound
        };
        let block = rule.Action().ok() == Some(NET_FW_ACTION_BLOCK);
        let enabled = rule.Enabled().ok() == Some(VARIANT_TRUE);
        out.push(FirewallRule {
            name,
            program,
            direction,
            block,
            enabled,
        });
    })?;
    Ok(out)
}

/// Blocks a program, inbound and outbound, under the given application name. Rules that already
/// exist are left as they are.
pub fn block(program: &str, application: &str) -> SysResult<()> {
    if !Path::new(program).is_absolute() {
        return Err(SysError::Other(format!("not a full path: {program}")));
    }
    let _com = Com::init()?;
    let rules = rules_collection()?;
    let application = application_name(application);
    for direction in RuleDirection::BOTH {
        let name = rule_name(&application, file_name(program), direction);
        if unsafe { rules.Item(&BSTR::from(name.as_str())) }.is_ok() {
            continue;
        }
        unsafe {
            let rule: INetFwRule = CoCreateInstance(&NetFwRule, None, CLSCTX_INPROC_SERVER)?;
            rule.SetName(&BSTR::from(name.as_str()))?;
            rule.SetDescription(&BSTR::from(DESCRIPTION))?;
            rule.SetApplicationName(&BSTR::from(program))?;
            rule.SetAction(NET_FW_ACTION_BLOCK)?;
            rule.SetDirection(match direction {
                RuleDirection::Inbound => NET_FW_RULE_DIR_IN,
                RuleDirection::Outbound => NET_FW_RULE_DIR_OUT,
            })?;
            rule.SetProfiles(NET_FW_PROFILE2_ALL.0)?;
            rule.SetEnabled(VARIANT_TRUE)?;
            rules.Add(&rule)?;
        }
    }
    Ok(())
}

/// Removes every AppIntBlocker-named rule that matches, and returns how many went.
fn remove_where(matches: impl Fn(&str, Option<&str>) -> bool) -> SysResult<u32> {
    let _com = Com::init()?;
    let rules = rules_collection()?;
    let mut names = BTreeSet::new();
    each_own_rule(&rules, |rule, name| {
        let program = unsafe { rule.ApplicationName() }.ok().map(|b| b.to_string());
        if matches(&name, program.as_deref()) {
            names.insert(name);
        }
    })?;
    let mut removed = 0;
    for name in names {
        // Remove takes one rule per call; rules can share a name.
        let bstr = BSTR::from(name.as_str());
        while unsafe { rules.Item(&bstr) }.is_ok() {
            unsafe { rules.Remove(&bstr)? };
            removed += 1;
            if removed > 10_000 {
                return Err(SysError::Other("the firewall did not remove the rule".into()));
            }
        }
    }
    Ok(removed)
}

/// Gives a program its access back: removes the block rules that name it.
pub fn unblock_program(program: &str) -> SysResult<u32> {
    remove_where(|_, p| p.is_some_and(|p| p.eq_ignore_ascii_case(program)))
}

/// Gives an application its access back: removes the rules with its name.
pub fn unblock_application(application: &str) -> SysResult<u32> {
    remove_where(|name, _| crate::rules::application_of(name).is_some_and(|a| a.eq_ignore_ascii_case(application)))
}

/// Removes every AppIntBlocker-named rule, from netSwitch, AppIntBlocker and its window alike.
pub fn unblock_all() -> SysResult<u32> {
    remove_where(|_, _| true)
}

/// Saves the whole firewall policy with `netsh advfirewall export`. `netsh advfirewall import`
/// restores it.
pub fn export_policy(file: &Path) -> SysResult<()> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let output = std::process::Command::new("netsh")
        .args(["advfirewall", "export"])
        .arg(file)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| SysError::Other(format!("netsh could not start: {e}")))?;
    if output.status.success() && file.exists() {
        Ok(())
    } else {
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Err(SysError::Other(if text.is_empty() {
            "netsh advfirewall export failed".into()
        } else {
            text
        }))
    }
}
