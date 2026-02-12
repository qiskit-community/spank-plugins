#
# This code is part of Qiskit.
#
# (C) Copyright IBM, Pasqal 2025, 2026
#
# This program and the accompanying materials are made available under the
# terms of the GNU General Public License version 3, as published by the
# Free Software Foundation.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <[https://www.gnu.org/licenses/gpl-3.0.txt]
#

cmake_minimum_required(VERSION 3.15)
set(CMAKE_C_STANDARD 11)
project (spank_qrmi C)

if(NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE Release CACHE STRING "Build type" FORCE)
endif()

# Switch:
#   -DQRMI_ROOT=/path/to/qrmi   Use local QRMI code.
#   unset/empty QRMI_ROOT       Fetch QRMI via ExternalProject on GitHub (default).
set(QRMI_ROOT "" CACHE PATH "Path to local qrmi checkout; leave empty to fetch from GitHub")

# -------------------------
# QRMI dependency selection
# -------------------------
if(NOT "${QRMI_ROOT}" STREQUAL "")
  MESSAGE(STATUS "Using local QRMI at ${QRMI_ROOT}")

  # Map CMake build type to Cargo profile.
  if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(QRMI_CARGO_ARGS "")
    set(QRMI_LIB_DIR "${QRMI_ROOT}/target/debug")
  else()
    set(QRMI_CARGO_ARGS "--release")
    set(QRMI_LIB_DIR "${QRMI_ROOT}/target/release")
  endif()

  set(QRMI_SO "${QRMI_LIB_DIR}/libqrmi.so")
  set(QRMI_H  "${QRMI_ROOT}/qrmi.h")

  # A real build target (COMMAND-based), similar to ExternalProject behavior.
  # "ALL" ensures it is part of the default build graph.
  add_custom_target(QRMI ALL
    COMMAND cargo build ${QRMI_CARGO_ARGS}
    WORKING_DIRECTORY "${QRMI_ROOT}"
    COMMENT "Building QRMI with Cargo"
    VERBATIM
  )

  # Imported library consuming the local artifact.
  add_library(qrmi SHARED IMPORTED GLOBAL)
  set_target_properties(qrmi PROPERTIES
    IMPORTED_LOCATION "${QRMI_SO}"
    INTERFACE_INCLUDE_DIRECTORIES "${QRMI_ROOT}"
  )

  # Ensure build order: run Cargo before linking anything against qrmi.
  add_dependencies(qrmi QRMI)

  # Fail fast on missing header.
  if(NOT EXISTS "${QRMI_H}")
    MESSAGE(FATAL_ERROR "QRMI header not found at ${QRMI_H}. Check QRMI_ROOT=${QRMI_ROOT}")
  endif()

  # Runtime loader path for the module to find libqrmi.so.
  set(QRMI_RPATH "${QRMI_LIB_DIR}")

  set(QRMI_TARGET qrmi)

else()
  MESSAGE(STATUS "Fetching QRMI from GitHub")

  include(ExternalProject)

  ExternalProject_Add(QRMI
    GIT_REPOSITORY https://github.com/qiskit-community/qrmi.git
    GIT_TAG main
    PREFIX ${CMAKE_BINARY_DIR}/deps
    CONFIGURE_COMMAND ""
    BUILD_COMMAND cargo build --release
    BUILD_IN_SOURCE 1
    INSTALL_COMMAND ""
    BUILD_BYPRODUCTS
        ${CMAKE_BINARY_DIR}/deps/src/QRMI/target/release/libqrmi.a
        ${CMAKE_BINARY_DIR}/deps/src/QRMI/qrmi.h
)

  add_library(qrmi STATIC IMPORTED GLOBAL)
  set_target_properties(qrmi PROPERTIES
    IMPORTED_LOCATION "${CMAKE_BINARY_DIR}/deps/src/QRMI/target/release/libqrmi.a"
    INTERFACE_INCLUDE_DIRECTORIES "${CMAKE_BINARY_DIR}/deps/src/QRMI"
  )

  # Ensure build order: ExternalProject QRMI runs before anything links against qrmi.
  add_dependencies(qrmi QRMI)

  set(QRMI_TARGET qrmi)
  set(QRMI_RPATH "")
endif()

# --------------------------------
# Find SLURM headers
# --------------------------------
include(CheckIncludeFiles)
include(FindPackageHandleStandardArgs)

# Check environment variables first
if(DEFINED ENV{SLURM_INCLUDE_DIRS})
  set(SLURM_INCLUDE_DIR $ENV{SLURM_INCLUDE_DIRS})
endif()

if(NOT SLURM_INCLUDE_DIR)
  find_path(SLURM_INCLUDE_DIR NAMES slurm/slurm.h)
endif()

find_package_handle_standard_args(SLURM DEFAULT_MSG SLURM_INCLUDE_DIR)
if (NOT SLURM_FOUND)
  MESSAGE(FATAL_ERROR "SLURM header files could not be found")
else()
  set(SLURM_INCLUDE_DIRS ${SLURM_INCLUDE_DIR})
endif()
mark_as_advanced (SLURM_INCLUDE_DIRS)
include_directories (BEFORE ${SLURM_INCLUDE_DIRS})

# -----------------------
# OpenSSL selection logic
# -----------------------
if(DEFINED ENV{OPENSSL_STATIC})
    set(SSL_LIB "")
    set(CRYPTO_LIB "")
else()
    set(SSL_LIB ssl)
    set(CRYPTO_LIB crypto)
endif()

# -----------------------
# Build spank_qrmi module
# -----------------------
add_library(spank_qrmi MODULE spank_qrmi.c buf.c strbuf.c)

set_target_properties(spank_qrmi PROPERTIES
  PREFIX ""
  SUFFIX ""
  OUTPUT_NAME "spank_qrmi.so"
)

target_link_libraries(spank_qrmi
  PRIVATE
    ${SSL_LIB}
    ${CRYPTO_LIB}
    dl
    pthread
    m
    ${QRMI_TARGET}
)

add_dependencies(spank_qrmi ${QRMI_TARGET})

# Ensure libqrmi.so is found at runtime when using local shared library.
if(QRMI_RPATH)
  set_target_properties(spank_qrmi PROPERTIES
    BUILD_RPATH "${QRMI_RPATH}"
  )
endif()

target_compile_options(spank_qrmi
    PRIVATE -Wall -Wextra -Werror -O2 -pedantic -Wconversion -Wwrite-strings -Wfloat-equal
)
