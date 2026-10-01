#include "hwinfo.h"

#include <stdio.h>
#include <stdlib.h>

static void print_string(const char *label, int32_t (*fn)(char *, size_t, size_t *)) {
    char buf[512];
    size_t len = 0;
    int32_t rc = fn(buf, sizeof buf, &len);
    if (rc == HWINFO_OK) {
        printf("%s: %s (%zu bytes)\n", label, buf, len);
    } else {
        printf("%s: unavailable (%d)\n", label, rc);
    }
}

int main(void) {
    hwinfo_machine info;
    int32_t mask;

    if (hwinfo_abi_version() != HWINFO_ABI_VERSION) {
        fprintf(stderr, "abi mismatch\n");
        return 1;
    }
    if (hwinfo_machine_size() != sizeof(hwinfo_machine)) {
        fprintf(stderr, "struct size mismatch: lib %zu header %zu\n",
                hwinfo_machine_size(), sizeof(hwinfo_machine));
        return 1;
    }

    mask = hwinfo_query(&info);
    if (mask < 0) {
        fprintf(stderr, "hwinfo_query failed: %d\n", mask);
        return 1;
    }

    printf("mask: 0x%x\n", (unsigned)mask);
    printf("cpu:        %s\n", info.cpu_name);
    printf("cpu serial: %s\n", info.cpu_serial);
    printf("board:      %s\n", info.board_name);
    printf("serial:     %s\n", info.board_serial);
    printf("computer:   %s\n", info.computer_name);

    print_string("cpu fn", hwinfo_cpu_name);
    print_string("cpu serial fn", hwinfo_cpu_serial);
    print_string("board fn", hwinfo_board_name);
    print_string("serial fn", hwinfo_board_serial);
    print_string("computer fn", hwinfo_computer_name);
    return 0;
}
