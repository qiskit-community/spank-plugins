# This code is part of Qiskit.
#
# (C) Copyright IBM 2025
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

cmake_minimum_required(VERSION 3.10)
set(CMAKE_C_STANDARD 11)
SET(CMAKE_OSX_DEPLOYMENT_TARGET 15.2)
include (FindPackageHandleStandardArgs)

project("test" LANGUAGES C)

if ("${CMAKE_BUILD_TYPE}" STREQUAL "Debug")
  set(LIB_PATH debug)
else()
  set(LIB_PATH release)
endif()

set(PROJECT_ROOT ${CMAKE_CURRENT_SOURCE_DIR}/..)

find_library(SLURM_LIBRARY NAMES libslurm.so)

find_package_handle_standard_args(SLURM DEFAULT_MSG SLURM_LIBRARY)
if (NOT SLURM_FOUND)
  MESSAGE(FATAL_ERROR "SLURM library could not be found")
else (NOT SLURM_FOUND)
  set (SLURM_LIBRARIES ${SLURM_LIBRARY})
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

function(add_example TARGET_NAME C_FILE)
  # source files for the client app
  set(SRC_FILE
          ${CMAKE_CURRENT_SOURCE_DIR}/${C_FILE}
  )

  add_executable(${TARGET_NAME} ${SRC_FILE})

  target_compile_options(${TARGET_NAME}
	  PRIVATE
	  -Wall -Wextra -Werror -O2 -pedantic -Wconversion -Wwrite-strings
  )

  install (TARGETS ${TARGET_NAME} DESTINATION .)
endfunction()

add_example(test src/test.c)
