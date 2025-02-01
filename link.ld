ENTRY(start);

. = 0x80200000;

SECTIONS {
	.text : ALIGN(4K) {
                PROVIDE(text_start = .);
		*(.init);
		*(.text);
                PROVIDE(text_end = .);
	}
	.rodata : ALIGN(4K) {
                PROVIDE(rodata_start = .);
		*(.rodata);
                PROVIDE(rodata_end = .);
	}
	.data : ALIGN(4K) {
                PROVIDE(data_start = .);
		*(.data);
                PROVIDE(data_end = .);
	}        
	.bss : ALIGN(4K) {
		PROVIDE(bss_start = .);
		*(.bss);
		PROVIDE(stack_top = .);
		. += 4M;
		PROVIDE(heap_start = .);
		. += 64M;
		PROVIDE(heap_end = .);
		PROVIDE(global_pointer = .);
		PROVIDE(bss_end = .);
	}
}
