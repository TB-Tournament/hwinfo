#ifndef HWINFO_H
#define HWINFO_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#if defined(_WIN32)
#define HWINFO_API __declspec(dllimport)
#else
#define HWINFO_API
#endif

/** ABI 版本。结构体或返回值语义变化时递增。 */
#define HWINFO_ABI_VERSION 2u

/** 结构体里每个字符串的容量，含结尾的 NUL。 */
#define HWINFO_STR_MAX 256u

#define HWINFO_OK 0
/** 指针为空。 */
#define HWINFO_ERR_NULL (-1)
/** 缓冲区放不下（含 NUL）。`out_len` 会写成不含 NUL 的字节数。 */
#define HWINFO_ERR_SMALL (-2)
/** 当前系统读不到这个字段。 */
#define HWINFO_ERR_UNAVAILABLE (-3)
/** 库内部异常，字段未写入。 */
#define HWINFO_ERR_INTERNAL (-4)

#define HWINFO_HAS_CPU_NAME 0x1
#define HWINFO_HAS_BOARD_NAME 0x2
#define HWINFO_HAS_BOARD_SERIAL 0x4
#define HWINFO_HAS_COMPUTER_NAME 0x8
#define HWINFO_HAS_CPU_SERIAL 0x10

/**
 * 一次取齐的结果。字符串为 UTF-8，以 NUL 结尾。
 * 读不到的字段是空字符串。超过 255 字节的值会被截断；完整内容用单独的函数取。
 *
 * 主板序列号：
 * - Windows / Linux：SMBIOS Type 2（Base Board）
 * - macOS：整机平台序列号（IOPlatformSerialNumber）
 *
 * CPU 序列号：
 * - macOS：SoC unique-chip-id，按设备树字节序输出为大写十六进制
 * - Windows / Linux：SMBIOS Type 4 的序列号；x86 上若 CPUID PSN 开启则用该 96 位序列号
 * 现代 x86 大多关闭了 PSN，固件也常把 Type 4 留空，这时字段为空。
 */
typedef struct hwinfo_machine {
    char cpu_name[HWINFO_STR_MAX];
    char board_name[HWINFO_STR_MAX];
    char board_serial[HWINFO_STR_MAX];
    char computer_name[HWINFO_STR_MAX];
    char cpu_serial[HWINFO_STR_MAX];
} hwinfo_machine;

#if defined(__STDC_VERSION__) && __STDC_VERSION__ >= 201112L
_Static_assert(sizeof(hwinfo_machine) == HWINFO_STR_MAX * 5, "hwinfo_machine layout");
#endif

/** 返回 HWINFO_ABI_VERSION。 */
HWINFO_API uint32_t hwinfo_abi_version(void);

/** 返回 `sizeof(hwinfo_machine)`，供调用方核对布局。 */
HWINFO_API size_t hwinfo_machine_size(void);

/**
 * 填写 `out`。返回已填字段的位掩码（HWINFO_HAS_*）。
 * `out == NULL` 时返回 HWINFO_ERR_NULL。
 * 掩码为 0 表示调用成功但五个字段都不可用。
 */
HWINFO_API int32_t hwinfo_query(hwinfo_machine *out);

/**
 * 把字符串写入 `buf`。成功返回 HWINFO_OK。
 * `out_len` 可为空；不为空时写入不含 NUL 的字节长度。
 * `buf == NULL && buf_len == 0` 时只查询长度并返回 HWINFO_ERR_SMALL。
 */
HWINFO_API int32_t hwinfo_cpu_name(char *buf, size_t buf_len, size_t *out_len);
HWINFO_API int32_t hwinfo_cpu_serial(char *buf, size_t buf_len, size_t *out_len);
HWINFO_API int32_t hwinfo_board_name(char *buf, size_t buf_len, size_t *out_len);
HWINFO_API int32_t hwinfo_board_serial(char *buf, size_t buf_len, size_t *out_len);
HWINFO_API int32_t hwinfo_computer_name(char *buf, size_t buf_len, size_t *out_len);

#ifdef __cplusplus
}
#endif

#endif /* HWINFO_H */
