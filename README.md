## Prerequisites

### Windows
Install LLVM: `winget install LLVM.LLVM`
Install GStreamer: https://gstreamer.freedesktop.org/download/#windows 
- install both runtime and development installer

- add gstreamer to path: ` $env:PATH="C:\Program Files\gstreamer\1.0\msvc_x86_64\bin;$env:PATH"` (TODO: app silently crashes without this)
Install pkg-config: `choco install pkgconfiglite`

point pkg-config to the gstreamer files: `setx PKG_CONFIG_PATH "C:\Program Files\gstreamer\1.0\msvc_x86_64\lib\pkgconfig"`

### macOS
Install GStreamer via Homebrew:
```bash
brew install gstreamer gst-plugins-base gst-plugins-good gst-plugins-bad gst-plugins-ugly gst-libav
```

The build script will automatically configure library paths. If you still encounter issues, you may need to set the `DYLD_LIBRARY_PATH` environment variable:
```bash
export DYLD_LIBRARY_PATH="/opt/homebrew/lib:$DYLD_LIBRARY_PATH"  # Apple Silicon
# or
export DYLD_LIBRARY_PATH="/usr/local/lib:$DYLD_LIBRARY_PATH"     # Intel
```