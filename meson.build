project('riscv', ['rust', 'c'])

link_script = meson.source_root() + '/link.ld'

bitflags = dependency('bitflags-2-rs')

sources = [
  'src/lib.rs',
  'src/uart.rs',
  'src/kprint.rs'
]

flags = ['-C', 'panic=abort']

if get_option('qemu').enabled()
  flags += ['--cfg', 'qemu']  
endif

libkernel = static_library('kernel', sources, rust_crate_type: 'staticlib', rust_args: flags)

kernel = executable(
  'kernel',
  ['boot.s'],
  link_with: [libkernel],
  link_args: ['-T', link_script, '-nostartfiles']
)
