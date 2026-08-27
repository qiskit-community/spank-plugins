#
# This code is part of Qiskit.
#
# (C) Copyright IBM 2026
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
# along with this program.  If not, see
# <[https://www.gnu.org/licenses/gpl-3.0.txt]
#
#
# generate_version_header.cmake
#
# Invoked as a build-time step (via add_custom_command) AFTER the QRMI
# ExternalProject has been fetched and built. Reads QRMI's Cargo.toml
# version and git commit hash, and writes them into a generated C header
# that spank_qrmi.c can #include.
#
# Required variables (passed with -D on the command line):
#   QRMI_SOURCE_DIR  - path to the fetched/local QRMI checkout
#   OUTPUT_FILE      - path to the header file to generate
#   SPANK_QRMI_VERSION - version string of this plugin itself (optional)

if(NOT DEFINED QRMI_SOURCE_DIR)
  message(FATAL_ERROR "QRMI_SOURCE_DIR not set")
endif()
if(NOT DEFINED OUTPUT_FILE)
  message(FATAL_ERROR "OUTPUT_FILE not set")
endif()

set(QRMI_CARGO_TOML "${QRMI_SOURCE_DIR}/Cargo.toml")

# --- Extract version from QRMI's Cargo.toml [package] section ---
set(QRMI_CRATE_VERSION "unknown")
if(EXISTS "${QRMI_CARGO_TOML}")
  file(STRINGS "${QRMI_CARGO_TOML}" CARGO_LINES)
  set(IN_PACKAGE_SECTION FALSE)
  foreach(LINE ${CARGO_LINES})
    if(LINE MATCHES "^\\[package\\]")
      set(IN_PACKAGE_SECTION TRUE)
    elseif(LINE MATCHES "^\\[")
      set(IN_PACKAGE_SECTION FALSE)
    elseif(IN_PACKAGE_SECTION AND LINE MATCHES "^version[ \t]*=[ \t]*\"([^\"]+)\"")
      set(QRMI_CRATE_VERSION "${CMAKE_MATCH_1}")
    endif()
  endforeach()
else()
  message(WARNING "QRMI Cargo.toml not found at ${QRMI_CARGO_TOML}")
endif()

# --- Extract the actual git commit that was checked out ---
set(QRMI_GIT_HASH "unknown")
find_package(Git QUIET)
if(GIT_EXECUTABLE AND EXISTS "${QRMI_SOURCE_DIR}/.git")
  execute_process(
    COMMAND ${GIT_EXECUTABLE} rev-parse --short=12 HEAD
    WORKING_DIRECTORY "${QRMI_SOURCE_DIR}"
    OUTPUT_VARIABLE QRMI_GIT_HASH_RAW
    OUTPUT_STRIP_TRAILING_WHITESPACE
    RESULT_VARIABLE GIT_RESULT
  )
  if(GIT_RESULT EQUAL 0 AND QRMI_GIT_HASH_RAW)
    set(QRMI_GIT_HASH "${QRMI_GIT_HASH_RAW}")
  endif()
endif()

if(NOT DEFINED SPANK_QRMI_VERSION OR SPANK_QRMI_VERSION STREQUAL "")
  set(SPANK_QRMI_VERSION "unknown")
endif()

# --- Write the generated header ---
file(WRITE "${OUTPUT_FILE}"
"/* Auto-generated at build time. Do not edit. */
#ifndef QRMI_VERSION_H
#define QRMI_VERSION_H

#define SPANK_QRMI_PLUGIN_VERSION \"${SPANK_QRMI_VERSION}\"
#define QRMI_CRATE_VERSION \"${QRMI_CRATE_VERSION}\"
#define QRMI_GIT_HASH \"${QRMI_GIT_HASH}\"

#endif /* QRMI_VERSION_H */
")

message(STATUS "Generated ${OUTPUT_FILE}: QRMI_CRATE_VERSION=${QRMI_CRATE_VERSION} QRMI_GIT_HASH=${QRMI_GIT_HASH}")
