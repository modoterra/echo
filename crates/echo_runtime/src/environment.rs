use std::env;
use std::ffi::OsStr;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI64, Ordering};

use crate::{
    EchoValue, echo_runtime_string, echo_value_array_append, echo_value_array_new,
    echo_value_array_set, write_runtime_output,
};

pub const PHP_COMPAT_VERSION: &str = "8.2.0";
pub const ZEND_COMPAT_VERSION: &str = "8.2.0";

static HTTP_RESPONSE_CODE: AtomicI64 = AtomicI64::new(0);
static IGNORE_USER_ABORT: AtomicI64 = AtomicI64::new(0);
static CLI_PROCESS_TITLE: Mutex<Option<Vec<u8>>> = Mutex::new(None);

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_getenv(name: EchoValue, _local_only: EchoValue) -> EchoValue {
    if name.is_null() {
        let mut result = echo_value_array_new();

        for (key, value) in env::vars_os() {
            result = echo_value_array_set(
                result,
                echo_runtime_string(os_string_bytes(&key)),
                echo_runtime_string(os_string_bytes(&value)),
            );
        }

        return result;
    }

    let Some(bytes) = name.string_bytes() else {
        return EchoValue::bool(false);
    };
    let Ok(key) = String::from_utf8(bytes) else {
        return EchoValue::bool(false);
    };

    env::var_os(key)
        .map(|value| echo_runtime_string(os_string_bytes(&value)))
        .unwrap_or_else(|| EchoValue::bool(false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_openlog(
    _prefix: EchoValue,
    _flags: EchoValue,
    _facility: EchoValue,
) -> EchoValue {
    EchoValue::bool(true)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_syslog(_priority: EchoValue, _message: EchoValue) -> EchoValue {
    EchoValue::bool(true)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_closelog() -> EchoValue {
    EchoValue::bool(true)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_image_type_to_extension(
    image_type: EchoValue,
    include_dot: EchoValue,
) -> EchoValue {
    let Some(image_type) = image_type.php_int_value() else {
        return EchoValue::bool(false);
    };
    let Some(extension) = image_type_extension(image_type) else {
        return EchoValue::bool(false);
    };

    let mut bytes = Vec::new();
    if include_dot.bool_value().unwrap_or(true) {
        bytes.push(b'.');
    }
    bytes.extend_from_slice(extension);
    echo_runtime_string(bytes)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_image_type_to_mime_type(image_type: EchoValue) -> EchoValue {
    let Some(image_type) = image_type.php_int_value() else {
        return EchoValue::bool(false);
    };
    image_type_mime_type(image_type)
        .map(|mime_type| echo_runtime_string(mime_type.to_vec()))
        .unwrap_or_else(|| EchoValue::bool(false))
}

fn image_type_extension(image_type: i64) -> Option<&'static [u8]> {
    match image_type {
        1 => Some(b"gif"),
        2 => Some(b"jpeg"),
        3 => Some(b"png"),
        4 => Some(b"swf"),
        5 => Some(b"psd"),
        6 => Some(b"bmp"),
        7 | 8 => Some(b"tiff"),
        9 => Some(b"jpc"),
        10 => Some(b"jp2"),
        11 => Some(b"jpx"),
        12 => Some(b"jb2"),
        13 => Some(b"swc"),
        14 => Some(b"iff"),
        15 => Some(b"wbmp"),
        16 => Some(b"xbm"),
        17 => Some(b"ico"),
        18 => Some(b"webp"),
        19 => Some(b"avif"),
        20 => Some(b"heif"),
        _ => None,
    }
}

fn image_type_mime_type(image_type: i64) -> Option<&'static [u8]> {
    match image_type {
        1 => Some(b"image/gif"),
        2 => Some(b"image/jpeg"),
        3 => Some(b"image/png"),
        4 | 13 => Some(b"application/x-shockwave-flash"),
        5 => Some(b"image/psd"),
        6 => Some(b"image/bmp"),
        7 | 8 => Some(b"image/tiff"),
        9 | 11 | 12 => Some(b"application/octet-stream"),
        10 => Some(b"image/jp2"),
        14 => Some(b"image/iff"),
        15 => Some(b"image/vnd.wap.wbmp"),
        16 => Some(b"image/xbm"),
        17 => Some(b"image/vnd.microsoft.icon"),
        18 => Some(b"image/webp"),
        19 => Some(b"image/avif"),
        20 => Some(b"image/heif"),
        _ => None,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_gethostname() -> EchoValue {
    env::var_os("HOSTNAME")
        .and_then(non_empty_os_string_bytes)
        .or_else(|| hostname_file_bytes(Path::new("/proc/sys/kernel/hostname")))
        .or_else(|| hostname_file_bytes(Path::new("/etc/hostname")))
        .map(echo_runtime_string)
        .unwrap_or_else(|| EchoValue::bool(false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_gethostbyname(hostname: EchoValue) -> EchoValue {
    let Some(bytes) = hostname.string_bytes() else {
        return EchoValue::error();
    };
    let lookup_bytes = bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes.as_slice(), |nul| &bytes[..nul]);

    let result = resolve_ipv4_host(lookup_bytes).unwrap_or(lookup_bytes.to_vec());
    echo_runtime_string(result)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_gethostbynamel(hostname: EchoValue) -> EchoValue {
    let Some(bytes) = hostname.string_bytes() else {
        return EchoValue::error();
    };
    let lookup_bytes = bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes.as_slice(), |nul| &bytes[..nul]);

    let Some(addresses) = resolve_ipv4_hosts(lookup_bytes) else {
        return EchoValue::bool(false);
    };

    let mut result = echo_value_array_new();
    for address in addresses {
        result = echo_value_array_append(result, echo_runtime_string(address));
    }
    result
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_gethostbyaddr(ip: EchoValue) -> EchoValue {
    let Some(bytes) = ip.string_bytes() else {
        return EchoValue::error();
    };
    let lookup_bytes = bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes.as_slice(), |nul| &bytes[..nul]);
    let Some(address) = parse_ip_address(lookup_bytes) else {
        return EchoValue::bool(false);
    };

    echo_runtime_string(reverse_host_by_address(address).unwrap_or_else(|| lookup_bytes.to_vec()))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_getprotobyname(name: EchoValue) -> EchoValue {
    let Some(name) = name.string_bytes() else {
        return EchoValue::bool(false);
    };

    protocol_number_by_name(&name)
        .map(EchoValue::int)
        .unwrap_or_else(|| EchoValue::bool(false))
}

fn parse_ip_address(bytes: &[u8]) -> Option<IpAddr> {
    std::str::from_utf8(bytes).ok()?.parse().ok()
}

fn reverse_host_by_address(address: IpAddr) -> Option<Vec<u8>> {
    hosts_file_name_for_address(Path::new("/etc/hosts"), address)
}

fn hosts_file_name_for_address(path: &Path, address: IpAddr) -> Option<Vec<u8>> {
    let contents = std::fs::read_to_string(path).ok()?;
    let address = address.to_string();
    for line in contents.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        if parts.next()? == address {
            return parts.next().map(|name| name.as_bytes().to_vec());
        }
    }
    None
}

fn resolve_ipv4_host(hostname: &[u8]) -> Option<Vec<u8>> {
    resolve_ipv4_hosts(hostname).and_then(|addresses| addresses.into_iter().next())
}

fn resolve_ipv4_hosts(hostname: &[u8]) -> Option<Vec<Vec<u8>>> {
    let hostname = std::str::from_utf8(hostname).ok()?;
    let mut addresses = Vec::new();
    for address in (hostname, 0).to_socket_addrs().ok()? {
        if let IpAddr::V4(address) = address.ip() {
            let bytes = address.to_string().into_bytes();
            if !addresses.contains(&bytes) {
                addresses.push(bytes);
            }
        }
    }
    if addresses.is_empty() {
        None
    } else {
        Some(addresses)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_getprotobynumber(number: EchoValue) -> EchoValue {
    let Some(number) = number.int_value() else {
        return EchoValue::bool(false);
    };

    protocol_name_by_number(number)
        .map(echo_runtime_string)
        .unwrap_or_else(|| EchoValue::bool(false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_getservbyname(service: EchoValue, protocol: EchoValue) -> EchoValue {
    let (Some(service), Some(protocol)) = (service.string_bytes(), protocol.string_bytes()) else {
        return EchoValue::bool(false);
    };

    service_port_by_name(&service, &protocol)
        .map(EchoValue::int)
        .unwrap_or_else(|| EchoValue::bool(false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_getservbyport(port: EchoValue, protocol: EchoValue) -> EchoValue {
    let (Some(port), Some(protocol)) = (port.int_value(), protocol.string_bytes()) else {
        return EchoValue::bool(false);
    };

    service_name_by_port(port, &protocol)
        .map(echo_runtime_string)
        .unwrap_or_else(|| EchoValue::bool(false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_getrusage() -> EchoValue {
    resource_usage_array(process_resource_usage())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_memory_get_usage() -> EchoValue {
    EchoValue::int(current_resident_memory_bytes().unwrap_or(0))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_memory_get_peak_usage() -> EchoValue {
    EchoValue::int(peak_resident_memory_bytes().unwrap_or(0))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_memory_reset_peak_usage() -> EchoValue {
    EchoValue::null()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_getmypid() -> EchoValue {
    EchoValue::int(std::process::id() as i64)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_getmyuid() -> EchoValue {
    proc_status_id("Uid")
        .map(EchoValue::int)
        .unwrap_or_else(|| EchoValue::bool(false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_getmygid() -> EchoValue {
    proc_status_id("Gid")
        .map(EchoValue::int)
        .unwrap_or_else(|| EchoValue::bool(false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_getmyinode() -> EchoValue {
    current_exe_inode()
        .map(EchoValue::int)
        .unwrap_or_else(|| EchoValue::bool(false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_getlastmod() -> EchoValue {
    current_exe_modified_timestamp()
        .map(EchoValue::int)
        .unwrap_or_else(|| EchoValue::bool(false))
}

#[cfg(unix)]
fn current_exe_inode() -> Option<i64> {
    use std::os::unix::fs::MetadataExt;

    let metadata = std::env::current_exe().ok()?.metadata().ok()?;
    i64::try_from(metadata.ino()).ok()
}

#[cfg(not(unix))]
fn current_exe_inode() -> Option<i64> {
    None
}

fn current_exe_modified_timestamp() -> Option<i64> {
    let modified = std::env::current_exe()
        .ok()?
        .metadata()
        .ok()?
        .modified()
        .ok()?;
    modified
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_secs()).ok())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_get_current_user() -> EchoValue {
    env::var_os("USER")
        .or_else(|| env::var_os("LOGNAME"))
        .map(|value| echo_runtime_string(os_string_bytes(&value)))
        .unwrap_or_else(|| echo_runtime_string(Vec::new()))
}

#[cfg(target_os = "linux")]
fn proc_status_id(field: &str) -> Option<i64> {
    let content = std::fs::read_to_string("/proc/self/status").ok()?;
    let prefix = format!("{field}:");
    let line = content.lines().find(|line| line.starts_with(&prefix))?;
    line[prefix.len()..].split_whitespace().next()?.parse().ok()
}

#[cfg(not(target_os = "linux"))]
fn proc_status_id(_field: &str) -> Option<i64> {
    None
}

fn protocol_number_by_name(name: &[u8]) -> Option<i64> {
    parse_protocol_number_by_name(&std::fs::read("/etc/protocols").unwrap_or_default(), name)
        .or_else(|| common_protocol_number_by_name(name))
}

fn protocol_name_by_number(number: i64) -> Option<Vec<u8>> {
    parse_protocol_name_by_number(&std::fs::read("/etc/protocols").unwrap_or_default(), number)
        .or_else(|| common_protocol_name_by_number(number))
}

fn service_port_by_name(service: &[u8], protocol: &[u8]) -> Option<i64> {
    parse_service_port_by_name(
        &std::fs::read("/etc/services").unwrap_or_default(),
        service,
        protocol,
    )
    .or_else(|| common_service_port_by_name(service, protocol))
}

fn service_name_by_port(port: i64, protocol: &[u8]) -> Option<Vec<u8>> {
    parse_service_name_by_port(
        &std::fs::read("/etc/services").unwrap_or_default(),
        port,
        protocol,
    )
    .or_else(|| common_service_name_by_port(port, protocol))
}

fn parse_protocol_number_by_name(content: &[u8], name: &[u8]) -> Option<i64> {
    for line in content.split(|byte| *byte == b'\n') {
        let before_comment = line.split(|byte| *byte == b'#').next().unwrap_or_default();
        let mut fields = before_comment
            .split(|byte| byte.is_ascii_whitespace())
            .filter(|field| !field.is_empty());

        let Some(protocol_name) = fields.next() else {
            continue;
        };
        let Some(number) = fields.next() else {
            continue;
        };

        if protocol_name == name || fields.any(|alias| alias == name) {
            return std::str::from_utf8(number).ok()?.parse().ok();
        }
    }

    None
}

fn parse_protocol_name_by_number(content: &[u8], target_number: i64) -> Option<Vec<u8>> {
    for line in content.split(|byte| *byte == b'\n') {
        let before_comment = line.split(|byte| *byte == b'#').next().unwrap_or_default();
        let mut fields = before_comment
            .split(|byte| byte.is_ascii_whitespace())
            .filter(|field| !field.is_empty());

        let Some(protocol_name) = fields.next() else {
            continue;
        };
        let Some(number) = fields.next() else {
            continue;
        };
        let Ok(number) = std::str::from_utf8(number).ok()?.parse::<i64>() else {
            continue;
        };

        if number == target_number {
            return Some(protocol_name.to_vec());
        }
    }

    None
}

fn parse_service_port_by_name(content: &[u8], service: &[u8], protocol: &[u8]) -> Option<i64> {
    for line in content.split(|byte| *byte == b'\n') {
        let before_comment = line.split(|byte| *byte == b'#').next().unwrap_or_default();
        let mut fields = before_comment
            .split(|byte| byte.is_ascii_whitespace())
            .filter(|field| !field.is_empty());

        let Some(service_name) = fields.next() else {
            continue;
        };
        let Some(port_protocol) = fields.next() else {
            continue;
        };
        let Some(separator) = port_protocol.iter().position(|byte| *byte == b'/') else {
            continue;
        };
        let port = &port_protocol[..separator];
        let service_protocol = &port_protocol[separator + 1..];

        if service_protocol != protocol {
            continue;
        }

        if service_name == service || fields.any(|alias| alias == service) {
            return std::str::from_utf8(port).ok()?.parse().ok();
        }
    }

    None
}

fn parse_service_name_by_port(
    content: &[u8],
    target_port: i64,
    protocol: &[u8],
) -> Option<Vec<u8>> {
    for line in content.split(|byte| *byte == b'\n') {
        let before_comment = line.split(|byte| *byte == b'#').next().unwrap_or_default();
        let mut fields = before_comment
            .split(|byte| byte.is_ascii_whitespace())
            .filter(|field| !field.is_empty());

        let Some(service_name) = fields.next() else {
            continue;
        };
        let Some(port_protocol) = fields.next() else {
            continue;
        };
        let Some(separator) = port_protocol.iter().position(|byte| *byte == b'/') else {
            continue;
        };
        let port = &port_protocol[..separator];
        let service_protocol = &port_protocol[separator + 1..];

        if service_protocol != protocol {
            continue;
        }

        let Ok(port) = std::str::from_utf8(port).ok()?.parse::<i64>() else {
            continue;
        };

        if port == target_port {
            return Some(service_name.to_vec());
        }
    }

    None
}

fn common_protocol_number_by_name(name: &[u8]) -> Option<i64> {
    match name {
        b"icmp" => Some(1),
        b"tcp" => Some(6),
        b"udp" => Some(17),
        b"ipv6" => Some(41),
        _ => None,
    }
}

fn common_protocol_name_by_number(number: i64) -> Option<Vec<u8>> {
    match number {
        1 => Some(b"icmp".to_vec()),
        6 => Some(b"tcp".to_vec()),
        17 => Some(b"udp".to_vec()),
        41 => Some(b"ipv6".to_vec()),
        _ => None,
    }
}

fn common_service_port_by_name(service: &[u8], protocol: &[u8]) -> Option<i64> {
    match (service, protocol) {
        (b"http" | b"www" | b"www-http", b"tcp") => Some(80),
        (b"https", b"tcp") => Some(443),
        (b"domain", b"tcp" | b"udp") => Some(53),
        (b"ssh", b"tcp") => Some(22),
        _ => None,
    }
}

fn common_service_name_by_port(port: i64, protocol: &[u8]) -> Option<Vec<u8>> {
    match (port, protocol) {
        (22, b"tcp") => Some(b"ssh".to_vec()),
        (53, b"tcp" | b"udp") => Some(b"domain".to_vec()),
        (80, b"tcp") => Some(b"http".to_vec()),
        (443, b"tcp") => Some(b"https".to_vec()),
        _ => None,
    }
}

#[derive(Default)]
struct ResourceUsage {
    user_seconds: i64,
    user_microseconds: i64,
    system_seconds: i64,
    system_microseconds: i64,
    max_rss_kb: i64,
}

fn process_resource_usage() -> ResourceUsage {
    let mut usage = proc_self_stat_usage().unwrap_or_default();
    if let Some(max_rss_kb) = proc_self_status_vm_hwm_kb() {
        usage.max_rss_kb = max_rss_kb;
    }
    usage
}

fn resource_usage_array(usage: ResourceUsage) -> EchoValue {
    let mut result = echo_value_array_new();
    for (key, value) in [
        ("ru_oublock", 0),
        ("ru_inblock", 0),
        ("ru_msgsnd", 0),
        ("ru_msgrcv", 0),
        ("ru_maxrss", usage.max_rss_kb),
        ("ru_ixrss", 0),
        ("ru_idrss", 0),
        ("ru_minflt", 0),
        ("ru_majflt", 0),
        ("ru_nsignals", 0),
        ("ru_nvcsw", 0),
        ("ru_nivcsw", 0),
        ("ru_nswap", 0),
        ("ru_utime.tv_usec", usage.user_microseconds),
        ("ru_utime.tv_sec", usage.user_seconds),
        ("ru_stime.tv_usec", usage.system_microseconds),
        ("ru_stime.tv_sec", usage.system_seconds),
    ] {
        result = echo_value_array_set(
            result,
            echo_runtime_string(key.as_bytes().to_vec()),
            EchoValue::int(value),
        );
    }
    result
}

#[cfg(target_os = "linux")]
fn proc_self_stat_usage() -> Option<ResourceUsage> {
    let content = std::fs::read_to_string("/proc/self/stat").ok()?;
    let fields_after_command = content.rsplit_once(") ")?.1;
    let fields: Vec<&str> = fields_after_command.split_whitespace().collect();
    let user_ticks: i64 = fields.get(11)?.parse().ok()?;
    let system_ticks: i64 = fields.get(12)?.parse().ok()?;

    Some(ResourceUsage {
        user_seconds: user_ticks / 100,
        user_microseconds: (user_ticks % 100) * 10_000,
        system_seconds: system_ticks / 100,
        system_microseconds: (system_ticks % 100) * 10_000,
        max_rss_kb: 0,
    })
}

#[cfg(not(target_os = "linux"))]
fn proc_self_stat_usage() -> Option<ResourceUsage> {
    None
}

#[cfg(target_os = "linux")]
fn proc_self_status_vm_hwm_kb() -> Option<i64> {
    let content = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = content.lines().find(|line| line.starts_with("VmHWM:"))?;
    line["VmHWM:".len()..]
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}

#[cfg(not(target_os = "linux"))]
fn proc_self_status_vm_hwm_kb() -> Option<i64> {
    None
}

#[cfg(target_os = "linux")]
fn current_resident_memory_bytes() -> Option<i64> {
    proc_self_status_kb("VmRSS").and_then(|kb| kb.checked_mul(1024))
}

#[cfg(not(target_os = "linux"))]
fn current_resident_memory_bytes() -> Option<i64> {
    None
}

#[cfg(target_os = "linux")]
fn peak_resident_memory_bytes() -> Option<i64> {
    proc_self_status_kb("VmHWM").and_then(|kb| kb.checked_mul(1024))
}

#[cfg(not(target_os = "linux"))]
fn peak_resident_memory_bytes() -> Option<i64> {
    None
}

#[cfg(target_os = "linux")]
fn proc_self_status_kb(field: &str) -> Option<i64> {
    let content = std::fs::read_to_string("/proc/self/status").ok()?;
    let prefix = format!("{field}:");
    let line = content.lines().find(|line| line.starts_with(&prefix))?;
    line[prefix.len()..].split_whitespace().next()?.parse().ok()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_sys_getloadavg() -> EchoValue {
    let Some(loads) = load_average_values() else {
        return EchoValue::bool(false);
    };

    let mut result = echo_value_array_new();
    for load in loads {
        result = echo_value_array_append(result, EchoValue::float(load));
    }
    result
}

#[cfg(target_os = "linux")]
fn load_average_values() -> Option<[f64; 3]> {
    let content = std::fs::read_to_string("/proc/loadavg").ok()?;
    let mut parts = content.split_whitespace();
    Some([
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    ])
}

#[cfg(not(target_os = "linux"))]
fn load_average_values() -> Option<[f64; 3]> {
    None
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_cli_get_process_title() -> EchoValue {
    let Ok(title) = CLI_PROCESS_TITLE.lock() else {
        return EchoValue::null();
    };

    title
        .clone()
        .map(echo_runtime_string)
        .unwrap_or_else(EchoValue::null)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_cli_set_process_title(title: EchoValue) -> EchoValue {
    let Some(bytes) = title.string_bytes() else {
        return EchoValue::bool(false);
    };
    let Ok(mut stored_title) = CLI_PROCESS_TITLE.lock() else {
        return EchoValue::bool(false);
    };

    *stored_title = Some(bytes);
    EchoValue::bool(true)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_phpversion(extension: EchoValue) -> EchoValue {
    if extension.is_null() {
        return echo_runtime_string(PHP_COMPAT_VERSION.as_bytes().to_vec());
    }

    let Some(_extension) = extension.string_bytes() else {
        return EchoValue::bool(false);
    };

    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_php_sapi_name() -> EchoValue {
    echo_runtime_string(b"cli".to_vec())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_phpcredits(_flags: EchoValue) -> EchoValue {
    write_runtime_output(b"PHP Credits\nEcho PHP compatibility runtime\n");
    EchoValue::bool(true)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_zend_version() -> EchoValue {
    echo_runtime_string(ZEND_COMPAT_VERSION.as_bytes().to_vec())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_dl(extension_filename: EchoValue) -> EchoValue {
    let Some(_extension_filename) = extension_filename.string_bytes() else {
        return EchoValue::bool(false);
    };

    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_php_uname(mode: EchoValue) -> EchoValue {
    let mode = mode
        .string_bytes()
        .and_then(|bytes| bytes.first().copied())
        .unwrap_or(b'a')
        .to_ascii_lowercase();

    let system = php_uname_system();
    let node = php_uname_node();
    let release = php_uname_file("/proc/sys/kernel/osrelease", "unknown");
    let version = php_uname_file("/proc/sys/kernel/version", "unknown");
    let machine = std::env::consts::ARCH.as_bytes().to_vec();

    let value = match mode {
        b's' => system,
        b'n' => node,
        b'r' => release,
        b'v' => version,
        b'm' => machine,
        _ => {
            let mut value = Vec::new();
            for (index, part) in [system, node, release, version, machine]
                .into_iter()
                .enumerate()
            {
                if index > 0 {
                    value.push(b' ');
                }
                value.extend(part);
            }
            value
        }
    };

    echo_runtime_string(value)
}

fn php_uname_system() -> Vec<u8> {
    if cfg!(target_os = "linux") {
        b"Linux".to_vec()
    } else if cfg!(target_os = "macos") {
        b"Darwin".to_vec()
    } else if cfg!(target_os = "windows") {
        b"Windows".to_vec()
    } else {
        std::env::consts::OS.as_bytes().to_vec()
    }
}

fn php_uname_node() -> Vec<u8> {
    env::var_os("HOSTNAME")
        .and_then(non_empty_os_string_bytes)
        .or_else(|| hostname_file_bytes(Path::new("/proc/sys/kernel/hostname")))
        .or_else(|| hostname_file_bytes(Path::new("/etc/hostname")))
        .unwrap_or_else(|| b"unknown".to_vec())
}

fn php_uname_file(path: &str, fallback: &str) -> Vec<u8> {
    std::fs::read(path)
        .ok()
        .map(|mut bytes| {
            while matches!(bytes.last(), Some(b'\n' | b'\r')) {
                bytes.pop();
            }
            bytes
        })
        .filter(|bytes| !bytes.is_empty())
        .unwrap_or_else(|| fallback.as_bytes().to_vec())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_extension_loaded(extension: EchoValue) -> EchoValue {
    let Some(_extension) = extension.string_bytes() else {
        return EchoValue::bool(false);
    };

    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_get_loaded_extensions(_zend_extensions: EchoValue) -> EchoValue {
    echo_value_array_new()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_get_extension_funcs(extension: EchoValue) -> EchoValue {
    let Some(_extension) = extension.string_bytes() else {
        return EchoValue::bool(false);
    };

    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_get_cfg_var(option: EchoValue) -> EchoValue {
    let Some(_option) = option.string_bytes() else {
        return EchoValue::bool(false);
    };

    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ini_get(option: EchoValue) -> EchoValue {
    let Some(_option) = option.string_bytes() else {
        return EchoValue::bool(false);
    };

    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ini_get_all(extension: EchoValue, _details: EchoValue) -> EchoValue {
    if extension.is_null() {
        return echo_value_array_new();
    }

    let Some(bytes) = extension.string_bytes() else {
        return EchoValue::bool(false);
    };

    if bytes.is_empty() {
        echo_value_array_new()
    } else {
        EchoValue::bool(false)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ini_parse_quantity(shorthand: EchoValue) -> EchoValue {
    let Some(bytes) = shorthand.string_bytes() else {
        return EchoValue::int(0);
    };

    EchoValue::int(parse_ini_quantity_bytes(&bytes))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_get_include_path() -> EchoValue {
    echo_php_ini_get(echo_runtime_string(b"include_path".to_vec()))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_set_include_path(include_path: EchoValue) -> EchoValue {
    echo_php_ini_set(echo_runtime_string(b"include_path".to_vec()), include_path)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_connection_aborted() -> EchoValue {
    EchoValue::int(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_connection_status() -> EchoValue {
    EchoValue::int(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ignore_user_abort(enable: EchoValue) -> EchoValue {
    if enable.is_null() {
        return EchoValue::int(IGNORE_USER_ABORT.load(Ordering::SeqCst));
    }

    let previous = IGNORE_USER_ABORT.swap(
        enable.bool_value().unwrap_or(false) as i64,
        Ordering::SeqCst,
    );
    EchoValue::int(previous)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_headers_list() -> EchoValue {
    echo_value_array_new()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_headers_sent() -> EchoValue {
    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_http_get_last_response_headers() -> EchoValue {
    EchoValue::null()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_http_clear_last_response_headers() {}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_header(
    header: EchoValue,
    _replace: EchoValue,
    _response_code: EchoValue,
) {
    let Some(_header) = header.string_bytes() else {
        return;
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_header_remove(name: EchoValue) {
    if name.is_null() {
        return;
    }

    let Some(_name) = name.string_bytes() else {
        return;
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_http_response_code(response_code: EchoValue) -> EchoValue {
    if response_code.is_null() {
        let current = HTTP_RESPONSE_CODE.load(Ordering::SeqCst);

        if current == 0 {
            EchoValue::bool(false)
        } else {
            EchoValue::int(current)
        }
    } else if let Some(code) = response_code.int_value() {
        let previous = HTTP_RESPONSE_CODE.swap(code, Ordering::SeqCst);

        if previous == 0 {
            EchoValue::bool(true)
        } else {
            EchoValue::int(previous)
        }
    } else {
        EchoValue::bool(false)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_mail(
    to: EchoValue,
    subject: EchoValue,
    message: EchoValue,
) -> EchoValue {
    if to.string_bytes().is_none()
        || subject.string_bytes().is_none()
        || message.string_bytes().is_none()
    {
        return EchoValue::error();
    }

    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ip2long(ip: EchoValue) -> EchoValue {
    let Some(bytes) = ip.string_bytes() else {
        return EchoValue::error();
    };
    match parse_ipv4_bytes(&bytes) {
        Some(value) => EchoValue::int(value as i64),
        None => EchoValue::bool(false),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_long2ip(ip: EchoValue) -> EchoValue {
    let Some(value) = ip.php_int_value() else {
        return EchoValue::error();
    };
    let bytes = (value as u32).to_be_bytes();
    echo_runtime_string(format!("{}.{}.{}.{}", bytes[0], bytes[1], bytes[2], bytes[3]).into_bytes())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_inet_pton(ip: EchoValue) -> EchoValue {
    let Some(bytes) = ip.string_bytes() else {
        return EchoValue::error();
    };
    let Ok(ip) = std::str::from_utf8(&bytes) else {
        return EchoValue::bool(false);
    };

    match ip.parse::<IpAddr>() {
        Ok(IpAddr::V4(address)) => echo_runtime_string(address.octets().to_vec()),
        Ok(IpAddr::V6(address)) => echo_runtime_string(address.octets().to_vec()),
        Err(_) => EchoValue::bool(false),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_inet_ntop(ip: EchoValue) -> EchoValue {
    let Some(bytes) = ip.string_bytes() else {
        return EchoValue::error();
    };

    match bytes.len() {
        4 => echo_runtime_string(
            Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3])
                .to_string()
                .into_bytes(),
        ),
        16 => {
            let mut octets = [0_u8; 16];
            octets.copy_from_slice(&bytes);
            echo_runtime_string(Ipv6Addr::from(octets).to_string().into_bytes())
        }
        _ => EchoValue::bool(false),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ini_set(option: EchoValue, value: EchoValue) -> EchoValue {
    let Some(_option) = option.string_bytes() else {
        return EchoValue::bool(false);
    };
    let Some(_value) = value.string_bytes() else {
        return EchoValue::bool(false);
    };

    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ini_alter(option: EchoValue, value: EchoValue) -> EchoValue {
    echo_php_ini_set(option, value)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ini_restore(option: EchoValue) {
    let Some(_option) = option.string_bytes() else {
        return;
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_php_ini_loaded_file() -> EchoValue {
    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_php_ini_scanned_files() -> EchoValue {
    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_putenv(assignment: EchoValue) -> EchoValue {
    let Some(bytes) = assignment.string_bytes() else {
        return EchoValue::bool(false);
    };
    let Ok(assignment) = String::from_utf8(bytes) else {
        return EchoValue::bool(false);
    };

    if let Some((name, value)) = assignment.split_once('=') {
        if name.is_empty() {
            return EchoValue::bool(false);
        }

        unsafe {
            env::set_var(name, value);
        }
    } else {
        if assignment.is_empty() {
            return EchoValue::bool(false);
        }

        unsafe {
            env::remove_var(assignment);
        }
    }

    EchoValue::bool(true)
}

fn parse_ipv4_bytes(bytes: &[u8]) -> Option<u32> {
    let mut parts = [0u8; 4];
    let mut part_index = 0;

    for part in bytes.split(|byte| *byte == b'.') {
        if part_index == 4 || part.is_empty() {
            return None;
        }
        let mut value = 0u16;
        for byte in part {
            if !byte.is_ascii_digit() {
                return None;
            }
            value = value
                .checked_mul(10)?
                .checked_add(u16::from(*byte - b'0'))?;
            if value > u8::MAX as u16 {
                return None;
            }
        }
        parts[part_index] = value as u8;
        part_index += 1;
    }

    if part_index != 4 {
        return None;
    }

    Some(u32::from_be_bytes(parts))
}

#[cfg(unix)]
fn os_string_bytes(value: &OsStr) -> Vec<u8> {
    value.as_bytes().to_vec()
}

#[cfg(not(unix))]
fn os_string_bytes(value: &OsStr) -> Vec<u8> {
    value.to_string_lossy().as_bytes().to_vec()
}

fn non_empty_os_string_bytes(value: std::ffi::OsString) -> Option<Vec<u8>> {
    let bytes = os_string_bytes(&value);

    if bytes.is_empty() { None } else { Some(bytes) }
}

fn hostname_file_bytes(path: &Path) -> Option<Vec<u8>> {
    let mut bytes = std::fs::read(path).ok()?;

    while matches!(bytes.last(), Some(b'\n' | b'\r')) {
        bytes.pop();
    }

    if bytes.is_empty() { None } else { Some(bytes) }
}

fn parse_ini_quantity_bytes(bytes: &[u8]) -> i64 {
    let bytes = trim_ascii_start(bytes);
    let Some((sign, after_sign)) = parse_ini_quantity_sign(bytes) else {
        return 0;
    };
    let Some((base, digits)) = parse_ini_quantity_base(after_sign) else {
        return 0;
    };

    let mut value = 0_i64;
    let mut consumed = 0;

    for &byte in digits {
        let Some(digit) = ascii_digit_value(byte) else {
            break;
        };
        if digit >= base {
            break;
        }

        value = value
            .saturating_mul(base as i64)
            .saturating_add(digit as i64);
        consumed += 1;
    }

    if consumed == 0 {
        return 0;
    }

    let mut value = if sign < 0 {
        value.saturating_neg()
    } else {
        value
    };

    if let Some(multiplier) = digits
        .get(consumed)
        .and_then(|byte| ini_quantity_multiplier(*byte))
    {
        value = value.saturating_mul(multiplier);
    }

    value
}

fn trim_ascii_start(mut bytes: &[u8]) -> &[u8] {
    while matches!(
        bytes.first(),
        Some(b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
    ) {
        bytes = &bytes[1..];
    }

    bytes
}

fn parse_ini_quantity_sign(bytes: &[u8]) -> Option<(i8, &[u8])> {
    match bytes.first() {
        Some(b'+') => Some((1, &bytes[1..])),
        Some(b'-') => Some((-1, &bytes[1..])),
        Some(_) => Some((1, bytes)),
        None => None,
    }
}

fn parse_ini_quantity_base(bytes: &[u8]) -> Option<(u8, &[u8])> {
    match bytes {
        [b'0', b'x' | b'X', rest @ ..] => Some((16, rest)),
        [b'0', b'b' | b'B', rest @ ..] => Some((2, rest)),
        [b'0', b'o' | b'O', rest @ ..] => Some((8, rest)),
        [b'0', rest @ ..] => Some((8, rest)),
        [_, ..] => Some((10, bytes)),
        [] => None,
    }
}

fn ascii_digit_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn ini_quantity_multiplier(byte: u8) -> Option<i64> {
    match byte {
        b'k' | b'K' => Some(1024),
        b'm' | b'M' => Some(1024 * 1024),
        b'g' | b'G' => Some(1024 * 1024 * 1024),
        _ => None,
    }
}
