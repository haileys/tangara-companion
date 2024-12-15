datadir = get_option('prefix') / get_option('datadir')

application_id = 'zone.cooltech.tangara.Companion'
# Read more <https://mesonbuild.com/Reference-manual.html#install_data>

install_data(
    '@0@.desktop'.format(application_id),
    install_dir: datadir / 'applications',
)

install_data(
    'assets/icon.svg',
    rename: ['@0@.svg'.format(application_id)],
    install_dir: datadir / 'icons' / 'hicolor' / 'scalable' / 'apps',
)

metainfo_xml = '@0@.metainfo.xml'.format(application_id)

gen_metainfo = find_program(
    join_paths(meson.project_source_root(),
        'script/meson-aux-gen-metainfo'))

custom_target(
    metainfo_xml,
    build_by_default: true,
    input: [metainfo_xml],
    output: metainfo_xml,
    install: true,
    install_dir: datadir / 'metainfo',
    command: [
        gen_metainfo,
        meson.project_version(),
        meson.current_source_dir() / metainfo_xml,
        '@OUTPUT@',
    ],
)
