# aarch64
- boot.S
clear out all bss, make sure FDT address gets saved into FDT_POINTER
- kmain
starts early bootup tasks, pivot to arch_kinit
- arch_kinit
parse everything needed for architecture specific operation

# scheduler
- CPUs bind to scheduler
