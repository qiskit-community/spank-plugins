cmake_minimum_required(VERSION 3.10)
project(spank_ibm_qrun C)

#
# For finding packages:
#
include(CheckIncludeFiles)
include(FindPackageHandleStandardArgs)

find_path(SLURM_INCLUDE_DIR NAMES slurm/slurm.h)
find_library(SLURM_LIBRARY NAMES libslurm.so)
find_package_handle_standard_args(SLURM DEFAULT_MSG SLURM_LIBRARY SLURM_INCLUDE_DIR)
if (NOT SLURM_FOUND)
  message(FATAL_ERROR "SLURM library could not be found")
else ()
  set(SLURM_LIBRARIES ${SLURM_LIBRARY})
  set(SLURM_INCLUDE_DIRS ${SLURM_INCLUDE_DIR})

  # Updated to search multiple paths for SLURM extensions directory
  find_path(SLURM_MODULES_DIR NAMES slurm
          PATHS
          /usr/lib/slurm
          /usr/lib64/slurm
          /usr/lib/x86_64-linux-gnu/slurm
          /usr/local/lib/slurm
          DOC "Directory containing SLURM extensions."
  )

  if (NOT SLURM_MODULES_DIR)
    message(WARNING "SLURM extensions directory could not be found. Using fallback: /usr/lib/slurm")
    set(SLURM_MODULES_DIR "/usr/lib/slurm")
  endif ()
endif ()

mark_as_advanced(SLURM_LIBRARIES SLURM_INCLUDE_DIRS SLURM_MODULES_DIR)

# Search for spank_ibm_qrun.c in possible locations
find_path(SPANK_IBM_QRUN_SRC NAMES spank_ibm_qrun.c
        PATHS
        ${CMAKE_CURRENT_SOURCE_DIR}
        ${CMAKE_CURRENT_SOURCE_DIR}/../plugins/spank_ibm_qrun
        DOC "Path to spank_ibm_qrun.c source file"
)

if (SPANK_IBM_QRUN_SRC)
  set(SPANK_IBM_QRUN_FILE "${SPANK_IBM_QRUN_SRC}/spank_ibm_qrun.c")
  add_library(spank_ibm_qrun MODULE ${SPANK_IBM_QRUN_FILE})
else ()
  message(FATAL_ERROR "Could not find spank_ibm_qrun.c in current directory or ../plugins/spank_ibm_qrun")
endif ()

include_directories(BEFORE ${SLURM_INCLUDE_DIRS})
set_target_properties(spank_ibm_qrun PROPERTIES PREFIX "" SUFFIX "" OUTPUT_NAME "spank_ibm_qrun.so")
