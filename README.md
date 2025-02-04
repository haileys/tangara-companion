# Tangara Companion

## Flashing via CLI
If you don't want to use the GUI application, you can easily flash firmware to your Tangara if you have a [Rust toolchain installed](https://www.rust-lang.org/tools/install). Download a Tangara firmware release from https://codeberg.org/cool-tech-zone/tangara-fw/releases then, in a shell in this repository, run:

```sh
cargo run -p tangara-cli flash /path/to/tangarafw-v1.x.y.tra
```

You can install the CLI tool by running:

```sh
cargo install --path crates/tangara-cli
```

Then to flash:

```sh
tangara flash /path/to/tangarafw-v1.x.y.tra
```

## Developing

### Tips

* If you're adding new svg assets or anything styled with particular colours, check that it looks good in both light and dark modes! Use the environment variables `ADW_DEBUG_COLOR_SCHEME=prefer-dark` and `ADW_DEBUG_COLOR_SCHEME=prefer-light`.

## Building

### Building on Linux

The Linux build uses flatpak to distribute the binary.  You will need `flatpak-builder` installed, and you will need to make sure you have the flathub remote ref set up for your user:

    ```sh-session
    $ flatpak remote-add --user --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
    ```

Then you can build and install with:

    ```sh-session
    $ script/build-linux-flatpak
    ```

The flatpak file will be in dist/[version] and you can `flatpak install` it from there.

### Cross compiling for Windows

This is a bit of a process to setup initially, but the DX is great once all the pieces are in place. Here's how I do it:

0. Install [xwin](https://jake-shadle.github.io/xwin/):

    ```sh-session
    $ cargo install xwin
    ```

0. Download the Windows SDKs and put them in `~/cross/windows-kits/xwin`. I put them here so I can use them from other projects.

    ```sh-session
    $ mkdir -p ~/cross/windows-kits
    $ cd ~/cross/windows-kits
    ~/cross/windows-kits $ xwin --accept-license splat --output xwin
    ```

0. Build a copy of the gtk4 libs on a Windows machine using the [`tangara-companion` branch of my fork of gvsbuild](https://github.com/haileys/gvsbuild/tree/tangara-companion). There are some circular dependencies between some of the projects in the gtk4 stack, so you'll need to run the build a few times in a specific order. Here's what I find works:

    ```sh-session
    > poetry run gvsbuild build --build-dir gtk-build gtk4
    > poetry run gvsbuild build --build-dir gtk-build librsvg
    > poetry run gvsbuild build --build-dir gtk-build --clean-built cairo gtk4
    > poetry run gvsbuild build --build-dir gtk-build libadwaita
    ```

0. Install [InnoSetup](https://jrsoftware.org/isdl.php) in your Wine environment. This will install to `C:\Program Files (x86)\Inno Setup 6`, which is currently hardcoded by the build script. We'll automate this at some point.

0. You're ready to run the build!

    ```sh-session
    $ script/build-windows
    ```

### Cross compiling for macOS

0. Clone [osxcross](https://github.com/tpoechtrager/osxcross) and follow the instructions to download an Xcode SDK from Apple

    ```sh-session
    $ git clone https://github.com/tpoechtrager/osxcross
    ```

0. Build the macOS cross kits with osxcross:

    ```sh-session
    osxcross $ export SDK_VERSION=14
    osxcross $ export TARGET_DIR=~/cross/macos-kits/osxcross
    osxcross $ ./build.sh
    osxcross $ ./build_compiler_rt.sh
    ```

   Follow the instructions given by `build_compiler.rt` to install the files to clang. You can also run the subsequent step under the `bwrap` tool bind mounting the right dirs in place instead of copying random stuff into your /usr prefix.

0. Build a copy of the gtk4 libs using the [`macos` branch of my gtk-builder tool](https://github.com/haileys/gtk-builder/tree/macos).

    ```sh-session
    gtk-builder $ export TARGET_DIR=~/cross/macos-kits/gtk/x86_64
    gtk-builder $ export RECIPE=macos-cross
    gtk-builder $ export RECIPE_ARCH=x86_64
    gtk-builder $ ./build all
    ```

0. You're ready to run the build!

    ```sh-session
    $ script/build-macos
    ```
