#!/usr/bin/ruby
require "fileutils"

dylibbundler_log = ENV.fetch("DYLIBBUNDLER_LOG")
LICENSES_DIR = ENV.fetch("LICENSES_DIR")
BREW_CELLAR = ENV.fetch("BREW_CELLAR")

def package_from_path(path)
  if !path.start_with?(BREW_CELLAR)
    fail "linked to non-homebrew package, can't get license info!"
  end

  path_in_cellar = path[BREW_CELLAR.length..]

  if path_in_cellar =~ %r{\A/(.*?)/(.*?)/}
    [$1, $2]
  else
    fail "weird path, dunno what to do! #{path}"
  end
end

LICENSE_FILENAMES = [
  "COPYING",
  "LICENSE",
  "LICENSE.txt",
  "LICENSE.md",
  "LGPL-2.1-or-later.txt",
]

def copy_license(package:, version:)
  package_dir = "#{BREW_CELLAR}/#{package}/#{version}"
  LICENSE_FILENAMES.each do |license_name|
    license_path = "#{package_dir}/#{license_name}"
    if File.exist?(license_path)
      puts "copying #{package} #{version} license from #{license_path}"
      FileUtils.cp(license_path, "#{LICENSES_DIR}/#{package}-#{version}.txt")
      return
    end
  end

  fail "couldn't locate any license under #{package_dir}"
end

dylibs = File.readlines(dylibbundler_log)
  .map { |line| line =~ %r{ \* (.*?\.dylib) from (.*)} && "#{$2}#{$1}" }
  .compact

dylibs
  .map { |path| package_from_path(path) }
  .each { |package, version| copy_license(package: package, version: version) }
