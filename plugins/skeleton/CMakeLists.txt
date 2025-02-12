cmake_minimum_required(VERSION 3.10)
project(spank_skeleton C)

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

add_library(spank_skeleton MODULE spank_skeleton.c)
include_directories(BEFORE ${SLURM_INCLUDE_DIRS})
set_target_properties(spank_skeleton PROPERTIES PREFIX "" SUFFIX "" OUTPUT_NAME "spank_skeleton.so")
