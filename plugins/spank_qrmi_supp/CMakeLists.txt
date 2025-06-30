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
project (spank_qrmi_supp C)

#
# For finding packages:
#
include(CheckIncludeFiles)
include(FindPackageHandleStandardArgs)

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

add_library (spank_qrmi_supp MODULE spank_qrmi_supp.c)
include_directories (BEFORE ${SLURM_INCLUDE_DIRS})
set_target_properties (spank_qrmi_supp PROPERTIES PREFIX "" SUFFIX "" OUTPUT_NAME "libspank_qrmi_supp.so")
