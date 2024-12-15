#!/usr/bin/ruby
require "fileutils"

ARCH="x86_64"
MACOS_KITS = ENV.fetch("MACOS_KITS")
TOOL_PREFIX = "#{ARCH}-apple-darwin23-"

def usage
  $stderr.puts "usage: install-dylibs.rb <bundle> -- <roots>"
  exit 1
end

def tool(name)
  "#{TOOL_PREFIX}#{name}"
end

def warning(msg)
  $stderr.puts "\e[33;1mwarning: \e[0m\e[97m#{msg}\e[0m"
end

def system!(*cmd)
  system(*cmd)
  fail "error running command: #{cmd.join(" ")}" unless $?.success?
end

def read_exe(path)
  dylibs = []
  rpaths = []

  IO.popen([tool("otool"), "-l", path], "r") do |io|
    io.read
      .split(/(?=^Load command \d+$)/m)
      .grep(/\ALoad command/)
      .each do |text|
        case text
        when /cmd LC_LOAD_DYLIB/
          text =~ /^\s+name (\S+)/m or fail "can't parse LC_LOAD_DYLIB command"
          dylibs << $1
        when /cmd LC_RPATH/
          text =~ /^\s+path (\S+)/m or fail "can't parse LC_RPATH command"
          rpaths << $1
        end
      end
  end

  fail "error reading exe with otool: #{path}" unless $?.success?

  { dylibs: dylibs, rpaths: rpaths }
end

def delete_rpaths(path)
  exe = read_exe(path)
  return if exe[:rpaths].empty?

  cmd = [tool("install_name_tool")]
  exe[:rpaths].each do |rpath|
    cmd << "-delete_rpath" << rpath
  end

  cmd << path

  system!(*cmd)
end

def add_rpath(path, lib:)
  system!(tool("install_name_tool"), "-add_rpath", lib, path)
end

class Image
  attr_reader :path, :fix_imports

  def initialize(path)
    @path = path
    @fix_imports = []
  end

  def info
    @info ||= read_exe(source_path)
  end

  def source_path
    path.gsub(%r{\A@rpath/}, "#{MACOS_KITS}/gtk/#{ARCH}/lib/")
  end

  def rpaths
    info[:rpaths]
  end

  def imports
    info[:dylibs]
  end
end

class Bundle
  attr_reader :lib, :roots, :images

  def initialize(lib_dir)
    @lib = lib_dir
    @images = {}
    @roots = []
  end

  def root(path)
    path = File.realpath("#{lib}/#{path}")
    walk(path)
    roots << path
  end

  def walk(path)
    # skip if we've seen already
    if images.key?(path)
      return
    end

    image = Image.new(path)
    images[path] = image

    image.imports.each do |import|
      # ignore system deps
      next if import.start_with?("/usr/lib/")
      next if import.start_with?("/System/Library")

      if import !~ %r{@rpath/}
        raise "unknown dependency #{import} in #{path}"
      end

      walk(import)
    end
  end
end

# process arguments
lib_dir = ARGV.shift or usage
ARGV.shift == "--" or usage
roots = ARGV

bundle = Bundle.new(lib_dir)
roots.each do |path|
  bundle.root(path)
end

# install all deps and clean rpaths
bundle.images.each do |path, dep|
  next unless %r{\A@rpath/(?<lib>.*)} =~ path

  puts "installing #{lib}"

  dest_path = "#{lib_dir}/#{lib}"
  FileUtils.cp(dep.source_path, dest_path)
  delete_rpaths(dest_path)
end

# set rpath on roots
bundle.roots.each do |path|
  puts "adding rpath for #{path}"
  add_rpath(path, lib: "@executable_path/../lib")
end
