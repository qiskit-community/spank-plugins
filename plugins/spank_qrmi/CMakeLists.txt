#
# This code is part of Qiskit.
#
# (C) Copyright IBM 2025
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

if ("${CMAKE_BUILD_TYPE}" STREQUAL "Debug")
  set(LIB_PATH debug)
else()
  set(LIB_PATH release)
endif()

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

#
# For finding packages:
#
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
  MESSAGE(FATAL_ERROR "SLURM library could not be found")
else()
  set(SLURM_INCLUDE_DIRS ${SLURM_INCLUDE_DIR})
endif()
mark_as_advanced (SLURM_INCLUDE_DIRS)
include_directories (BEFORE ${SLURM_INCLUDE_DIRS})

add_library (spank_qrmi MODULE spank_qrmi.c buf.c strbuf.c)
add_dependencies(spank_qrmi QRMI)
set_target_properties (spank_qrmi PROPERTIES PREFIX "" SUFFIX "" OUTPUT_NAME "spank_qrmi.so")
target_link_libraries(spank_qrmi
    PRIVATE ssl crypto dl pthread m ${CMAKE_BINARY_DIR}/deps/src/QRMI/target/release/libqrmi.a
)
target_include_directories(spank_qrmi
    PRIVATE ${CMAKE_BINARY_DIR}/deps/src/QRMI
)
target_compile_options(spank_qrmi
    PRIVATE -Wall -Wextra -Werror -O2 -pedantic -Wconversion -Wwrite-strings -Wfloat-equal
)
