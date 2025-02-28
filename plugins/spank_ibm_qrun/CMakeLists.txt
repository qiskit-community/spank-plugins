cmake_minimum_required(VERSION 3.10)
set(CMAKE_C_STANDARD 17)
project (spank_ibm_qrun C)

#
# For finding packages:
#
include(CheckIncludeFiles)
include(FindPackageHandleStandardArgs)

set(TARGET_NAME spank_ibm_qrun)
set(DAAPI_C_LIB
  -L${CMAKE_CURRENT_SOURCE_DIR}/../../commands/qrun/target/release
  -ldirect_access_capi
)

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
          /usr/lib64/slurm
          /usr/lib/x86_64-linux-gnu/slurm
          DOC "Directory containing SLURM extensions."
  )
  if (NOT SLURM_MODULES_DIR)
    message(FATAL_ERROR "SLURM extensions directory could not be found")
  endif (NOT SLURM_MODULES_DIR)
endif (NOT SLURM_FOUND)
mark_as_advanced (SLURM_LIBRARIES SLURM_INCLUDE_DIRS)

add_library (${TARGET_NAME} MODULE spank_ibm_qrun.c)
include_directories (BEFORE ${SLURM_INCLUDE_DIRS})
target_include_directories(${TARGET_NAME} PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/../../commands/qrun/daapi_c/)
target_link_libraries(${TARGET_NAME} PRIVATE ${DAAPI_C_LIB})
target_link_libraries(${TARGET_NAME} PUBLIC "-lssl")
target_link_libraries(${TARGET_NAME} PUBLIC "-lcrypto")
target_link_libraries(${TARGET_NAME} PUBLIC "-ldl")
target_link_libraries(${TARGET_NAME} PUBLIC "-lm")
target_link_libraries(${TARGET_NAME} PUBLIC "-lpthread")
set_target_properties (${TARGET_NAME} PROPERTIES PREFIX "" SUFFIX "" OUTPUT_NAME "spank_ibm_qrun.so")
