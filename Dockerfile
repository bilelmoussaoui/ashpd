FROM ghcr.io/gtk-rs/gtk4-rs/gtk4:latest

# Install build dependencies
RUN sudo dnf install -y \
    pipewire-devel \
    clang-devel \
    gstreamer1-devel \
    gstreamer1-plugins-base-devel \
    libshumate-devel \
    meson \
    ninja-build \
    git \
    sassc \
    && sudo dnf clean all

# Build and install libadwaita from git main
RUN git clone https://gitlab.gnome.org/GNOME/libadwaita.git /tmp/libadwaita \
    && cd /tmp/libadwaita \
    && meson setup _build \
        --prefix=/usr \
        --buildtype=release \
        -Dintrospection=disabled \
        -Dvapi=false \
        -Dtests=false \
        -Dexamples=false \
        -Dgtk:media-gstreamer=disabled \
        -Dgtk:x11-backend=false \
        -Dgtk:broadway-backend=false \
        -Dgtk:introspection=disabled \
        -Dgtk:documentation=false \
        -Dgtk:build-tests=false \
        -Dgtk:build-examples=false \
        -Dgtk:build-demos=false \
        -Dgtk:build-testsuite=false \
    && meson compile -C _build \
    && sudo meson install -C _build \
    && cd / \
    && rm -rf /tmp/libadwaita
