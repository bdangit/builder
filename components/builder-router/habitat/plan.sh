source "../../../support/ci/builder-base-plan.sh"
pkg_name=builder-router
pkg_origin=habitat
pkg_maintainer="The Habitat Maintainers <humans@habitat.sh>"
pkg_license=('Apache-2.0')
pkg_bin_dirs=(bin)
pkg_deps=(core/glibc core/openssl core/gcc-libs core/zeromq core/libsodium core/libarchive)
pkg_build_deps=(core/protobuf-cpp core/protobuf-rust core/coreutils core/cacerts
  core/rust core/gcc core/git core/pkg-config core/make core/cmake core/go)
pkg_exports=(
  [port]=port
)
pkg_exposes=(port)
bin="bldr-router"
