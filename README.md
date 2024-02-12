# Tangara Companion

## Developing

### Tips

* If you're adding new svg assets or anything styled with particular colours, check that it looks good in both light and dark modes! Use the environment variables `ADW_DEBUG_COLOR_SCHEME=prefer-dark` and `ADW_DEBUG_COLOR_SCHEME=prefer-light`.

## Building

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
