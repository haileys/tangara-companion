project(
  'tangara-companion', 'rust',
  version: '0.4.2',
  meson_version: '>= 1.3.0',
)

dependency('gio-2.0')
dependency('glib-2.0')
dependency('gtk4', version: '>= 4.10.0')
dependency('libadwaita-1', version: '>= 1.4')

subdir('data')
subdir('src')

meson.add_dist_script(
  'script/meson-aux-dist-vendor',
  meson.project_build_root() / 'meson-dist' / meson.project_name() + '-' + meson.project_version(),
  meson.project_source_root()
)
