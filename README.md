## Prerequisites

Install LLVM: `winget install LLVM.LLVM`
Install GStreamer: https://gstreamer.freedesktop.org/download/#windows 
- install both runtime and development installer

- add gstreamer to path: ` $env:PATH="C:\Program Files\gstreamer\1.0\msvc_x86_64\bin;$env:PATH"`
Install pkg-config: `choco install pkgconfiglite`

point pkg-config to the gstreamer files: `setx PKG_CONFIG_PATH "C:\Program Files\gstreamer\1.0\msvc_x86_64\lib\pkgconfig"`