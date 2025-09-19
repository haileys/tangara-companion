sources = [
    'main.rs',
]

cargo_script = find_program(join_paths(meson.project_source_root(), 'script/meson-aux-cargo'))
cargo_release = custom_target(
  'cargo-build',
  build_by_default: true,
  input: sources,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: get_option('bindir'),
  command: [
    cargo_script,
    meson.project_build_root(),
    meson.project_source_root(),
    '@OUTPUT@',
    get_option('debug').to_string(),
    meson.project_name(),
  ]
)
