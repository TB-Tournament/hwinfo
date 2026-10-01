/// x86 处理器序列号（CPUID PSN）。该功能在 Pentium III 之后基本被关掉，没有就不返回。
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) fn processor_serial() -> Option<String> {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::__cpuid;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::__cpuid;

    unsafe {
        let basic = __cpuid(0);
        if basic.eax < 3 {
            return None;
        }
        let leaf1 = __cpuid(1);
        // CPUID.01H:EDX[18] = Processor Serial Number。
        if leaf1.edx & (1 << 18) == 0 {
            return None;
        }
        let leaf3 = __cpuid(3);
        if leaf3.edx == 0 && leaf3.ecx == 0 {
            return None;
        }
        Some(format!(
            "{:08X}{:08X}{:08X}",
            leaf1.eax, leaf3.edx, leaf3.ecx
        ))
    }
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub(crate) fn processor_serial() -> Option<String> {
    None
}
