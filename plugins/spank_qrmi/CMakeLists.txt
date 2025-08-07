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

cmake_minimum_required(VERSION 3.10)
set(CMAKE_C_STANDARD 11)
project (spank_qrmi C)

include(ExternalProject)

ExternalProject_Add(QRMI
    GIT_REPOSITORY https://github.com/qiskit-community/qrmi.git
    GIT_TAG main
    PREFIX ${CMAKE_BINARY_DIR}/deps
    CONFIGURE_COMMAND ""
    BUILD_COMMAND cargo build --release
    BUILD_IN_SOURCE 1
    INSTALL_COMMAND ""
)

add_library(qrmi SHARED IMPORTED GLOBAL)
add_dependencies(qrmi qrmi_project)
set_target_properties(qrmi PROPERTIES
    IMPORTED_LOCATION ${CMAKE_BINARY_DIR}/deps/src/QRMI/target/release/libqrmi.a
)

#
# For finding packages:
#
include(CheckIncludeFiles)
include(FindPackageHandleStandardArgs)

if ("${CMAKE_BUILD_TYPE}" STREQUAL "Debug")
  set(LIB_PATH debug)
else()
  set(LIB_PATH release)
endif()
set(PROJECT_ROOT ${CMAKE_CURRENT_SOURCE_DIR}/../..)

find_path(SLURM_INCLUDE_DIR NAMES slurm/slurm.h)
find_library(SLURM_LIBRARY NAMES libslurm.so)
find_package_handle_standard_args(SLURM DEFAULT_MSG SLURM_LIBRARY SLURM_INCLUDE_DIR)
if (NOT SLURM_FOUND)
  MESSAGE(FATAL_ERROR "SLURM library could not be found")
else (NOT SLURM_FOUND)
  set (SLURM_LIBRARIES ${SLURM_LIBRARY})
  set (SLURM_INCLUDE_DIRS ${SLURM_INCLUDE_DIR})
  find_path(SLURM_MODULES_DIR NAMES slurm
          PATHS
          /lib64/slurm
          /usr/lib64/slurm
          /usr/lib/x86_64-linux-gnu/slurm
          DOC "Directory containing SLURM extensions."
  )
  if (NOT SLURM_MODULES_DIR)
    message(FATAL_ERROR "SLURM extensions directory could not be found")
  endif (NOT SLURM_MODULES_DIR)
endif (NOT SLURM_FOUND)
mark_as_advanced (SLURM_LIBRARIES SLURM_INCLUDE_DIRS)
include_directories (BEFORE ${SLURM_INCLUDE_DIRS})

add_library (spank_qrmi MODULE spank_qrmi.c buf.c strbuf.c)
set_target_properties (spank_qrmi PROPERTIES PREFIX "" SUFFIX "" OUTPUT_NAME "spank_qrmi.so")
target_link_libraries(spank_qrmi PRIVATE qrmi)
target_link_libraries(spank_qrmi PUBLIC "-lssl")
target_link_libraries(spank_qrmi PUBLIC "-lcrypto")
target_link_libraries(spank_qrmi PUBLIC "-ldl")
target_link_libraries(spank_qrmi PUBLIC "-lpthread")
target_link_libraries(spank_qrmi PUBLIC "-lm")
target_include_directories(spank_qrmi PRIVATE ${CMAKE_BINARY_DIR}/deps/src/QRMI)
