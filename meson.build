project(
  'riscv',
  ['rust', 'c'],
  meson_version: '>=1.6.1',
  default_options: {'rust_std': '2021'}
)

link_script = meson.source_root() + '/link.ld'

bitflags = dependency('bitflags-2-rs')

sources = [
  'src/lib.rs',
  'src/kprint.rs',
  'src/kalloc/mod.rs',
  'src/kalloc/bump.rs',
  'src/sbi/mod.rs',
  'src/sbi/console.rs'
]

rustflags = ['-C', 'panic=abort']

if get_option('qemu').enabled()
  rustflags += ['--cfg', 'qemu']  
endif

libkernel = static_library(
  'kernel',
  sources,
  rust_crate_type: 'staticlib',
  rust_args: rustflags,
  dependencies: [bitflags]
)

kernel = executable(
  'kernel',
  ['boot.s'],
  link_with: [libkernel],
  link_args: ['-T', link_script, '-nostartfiles']
)
