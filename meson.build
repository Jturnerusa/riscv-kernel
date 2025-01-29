project('riscv', ['rust', 'c'])

link_script = meson.source_root() + '/link.ld'

bitflags = dependency('bitflags-2-rs')

subdir('src')

kernel = executable(
  'kernel',
  ['boot.s'],
  link_with: [libkernel],
  link_args: ['-T', link_script, '-nostartfiles']
)
