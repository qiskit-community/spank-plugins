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

# Switches:
#   -DUSE_LOCAL_QRMI=ON   Use local QRMI checkout at QRMI_ROOT.
#   -DUSE_LOCAL_QRMI=OFF  Fetch/build QRMI via ExternalProject (default).
option(USE_LOCAL_QRMI "Use local QRMI checkout instead of fetching from GitHub" OFF)

# Switch:
#   -DQRMI_ROOT=/path/to/qrmi   Use local QRMI code.
#   unset/empty QRMI_ROOT       Fetch QRMI via ExternalProject on GitHub (default).
set(QRMI_ROOT "" CACHE PATH "Path to local qrmi checkout; leave empty to fetch from GitHub")

# -------------------------
# QRMI build type local vs GitHub
# -------------------------
# We use ExternalProject to build QRMI both locally and from GitHub
include(ExternalProject)
if(NOT "${QRMI_ROOT}" STREQUAL "")
  MESSAGE(STATUS "Using local QRMI at ${QRMI_ROOT}")
  set(QRMI_SOURCE_DIR "${QRMI_ROOT}")
  ExternalProject_Add(QRMI
    SOURCE_DIR "${QRMI_SOURCE_DIR}"
    PREFIX "${CMAKE_BINARY_DIR}/deps"
    CONFIGURE_COMMAND ""
    BUILD_COMMAND cargo build --release
    BUILD_IN_SOURCE 1
    INSTALL_COMMAND ""
    BUILD_BYPRODUCTS
      "${QRMI_SOURCE_DIR}/target/release/libqrmi.a"
      "${QRMI_SOURCE_DIR}/qrmi.h"
  )
else()
  MESSAGE(STATUS "Fetching QRMI from GitHub")
  set(QRMI_SOURCE_DIR "${CMAKE_BINARY_DIR}/deps/src/QRMI")
  ExternalProject_Add(QRMI
    GIT_REPOSITORY https://github.com/qiskit-community/qrmi.git
    GIT_TAG main
    PREFIX ${CMAKE_BINARY_DIR}/deps
    CONFIGURE_COMMAND ""
    BUILD_COMMAND cargo build --release
    BUILD_IN_SOURCE 1
    INSTALL_COMMAND ""
    BUILD_BYPRODUCTS
      "${QRMI_SOURCE_DIR}/target/release/libqrmi.a"
      "${QRMI_SOURCE_DIR}/qrmi.h"
  )
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
# Build spank_qrmi module
# -----------------------
add_library (spank_qrmi MODULE spank_qrmi.c buf.c strbuf.c)
add_dependencies(spank_qrmi QRMI)
set_target_properties (spank_qrmi PROPERTIES PREFIX "" SUFFIX "" OUTPUT_NAME "spank_qrmi.so")

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
    PRIVATE ${SSL_LIB} ${CRYPTO_LIB} dl pthread m ${QRMI_SOURCE_DIR}/target/release/libqrmi.a
)
target_include_directories(spank_qrmi
    PRIVATE ${QRMI_SOURCE_DIR}
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