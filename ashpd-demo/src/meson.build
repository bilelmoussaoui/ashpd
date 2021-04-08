global_conf = configuration_data()
global_conf.set_quoted('APP_ID', application_id)
global_conf.set_quoted('PKGDATADIR', pkgdatadir)
global_conf.set_quoted('PROFILE', profile)
global_conf.set_quoted('VERSION', version + version_suffix)
global_conf.set_quoted('GETTEXT_PACKAGE', gettext_package)
global_conf.set_quoted('LOCALEDIR', localedir)
config = configure_file(
  input: 'config.rs.in',
  output: 'config.rs',
  configuration: global_conf
)
# Copy the config.rs output to the source directory.
run_command(
  'cp',
  meson.build_root() / 'src' / 'config.rs',
  meson.source_root() / 'src' / 'config.rs',
  check: true
)

sources = files(
  'portals/desktop/account.rs',
  'portals/desktop/background.rs',
  'portals/desktop/camera.rs',
  'portals/desktop/device.rs',
  'portals/desktop/email.rs',
  'portals/desktop/location.rs',
  'portals/desktop/mod.rs',
  'portals/desktop/network_monitor.rs',
  'portals/desktop/notification.rs',
  'portals/desktop/open_uri.rs',
  'portals/desktop/screenshot.rs',
  'portals/desktop/screencast.rs',
  'portals/desktop/wallpaper.rs',
  'widgets/color_widget.rs',
  'widgets/gst_paintable.rs',
  'widgets/mod.rs',
  'application.rs',
  'config.rs',
  'main.rs',
  'sidebar_row.rs',
  'window.rs',
)

custom_target(
  'cargo-build',
  build_by_default: true,
  input: sources,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: bindir,
  depends: resources,
  command: [
    cargo_script,
    meson.build_root(),
    meson.source_root(),
    '@OUTPUT@',
    profile,
    meson.project_name(),
  ]
)
