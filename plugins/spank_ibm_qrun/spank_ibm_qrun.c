/*
 * This code is part of Qiskit.
 *
 * (C) Copyright IBM 2025.
 *
 * This code is licensed under the Apache License, Version 2.0. You may
 * obtain a copy of this license in the LICENSE.txt file in the root directory
 * of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
 *
 * Any modifications or derivative works of this code must retain this
 * copyright notice, and modified files need to carry a notice indicating
 * that they have been altered from the originals.
 */
#include <ctype.h>
#include <grp.h>
#include <limits.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "slurm/slurm.h"
#include "slurm/spank.h"

/*
 * All spank plugins must define this macro for the SLURM plugin loader.
 */
SPANK_PLUGIN(spank_ibm_qrun, 1)

/* Qiskit backend name */
#define MAXLEN_BACKEND_NAME 256
static char backend_name[MAXLEN_BACKEND_NAME + 1];

/* Qiskit primitive type */
#define MAXLEN_PROGRAM_ID 256
static char primitive_type[MAXLEN_PROGRAM_ID + 1];

/*
 * @function strncpy_s
 *
 * For security. Copy string src to buffer dst of size dsize.  At most dsize-1
 * chars will be copied. Always NULL terminates (unless dsize == 0).
 */
static char *strncpy_s(char *dst, const char *src, size_t dsize)
{
    if (dsize == 0) {
        return dst;
    }

    strncpy(dst, src, dsize - 1);
    dst[dsize - 1] = '\0';
    return dst;
}

/*
 * @function backend_name_cb
 *
 * Callback function to check --q-backend option value.
 *
 */
static int backend_name_cb(int val, const char *optarg, int remote)
{
    slurm_debug("%s: %s val=%d optarg=%s remote=%d",
            plugin_name, __FUNCTION__, val, optarg, remote);
    strncpy_s(backend_name, optarg, sizeof(backend_name));
    return ESPANK_SUCCESS;
}

/*
 * @function primitive_type_cb
 *
 * Callback function to check --q-primitive option value.
 *
 */
static int primitive_type_cb(int val, const char *optarg, int remote)
{
    slurm_debug("%s: %s val=%d optarg=%s remote=%d",
            plugin_name, __FUNCTION__, val, optarg, remote);
    strncpy_s(primitive_type, optarg, sizeof(primitive_type));
    return ESPANK_SUCCESS;
}

static int dump_spank_items(spank_t spank_ctxt)
{
    uid_t job_id = 0;
    uint32_t step_id = 0;
    int job_argc = 0;
    char **job_argv = NULL;

    if (spank_get_item(spank_ctxt, S_JOB_UID, &job_id) == ESPANK_SUCCESS) {
        slurm_debug("%s: S_JOB_UID [%d]", plugin_name, job_id);
    }
    if (spank_get_item(spank_ctxt, S_JOB_ID, &step_id) == ESPANK_SUCCESS) {
        slurm_debug("%s: S_JOB_ID [%d]", plugin_name, step_id);
    }

    if (spank_get_item(spank_ctxt, S_JOB_ARGV, &job_argc, &job_argv) ==
        ESPANK_SUCCESS) {
        slurm_debug("%s: S_JOB_ARGV argc=%d", plugin_name, job_argc);
        for (int i = 0; i < job_argc; i++) {
            slurm_debug("%s: job_argv[%d] = [%s]",
                    plugin_name, i, job_argv[i]);
        }
    }
    return ESPANK_SUCCESS;
}

static int dump_argv(int argc, char **argv)
{
    for (int i = 0; i < argc; i++) {
        slurm_debug("%s: argv[%d] = [%s]", plugin_name, i, argv[i]);
    }
    return ESPANK_SUCCESS;
}

/*
 * Options available to this spank plugin:
 */
struct spank_option spank_example_options[] = {
    {
        "q-backend",
        "name",
        "Name of Qiskit backend.",
        1, /* argument(backend name) is required */
        0, /* value to return using callback */
        (spank_opt_cb_f)backend_name_cb
    },
    {
        "q-primitive",
        "type",
        "Qiskit primitive type(sampler or estimator).",
        1, /* argument(primitive type name) is required. */
        0, /* value to return using callback */
        (spank_opt_cb_f)primitive_type_cb
    },
    SPANK_OPTIONS_TABLE_END
};

/*
 * @function slurm_spank_init
 *
 * Called just after plugins are loaded. In remote context, this is just after
 * job step is initialized. This function is called before any plugin option
 * processing.
 *
 */
int slurm_spank_init(spank_t spank_ctxt, int argc, char *argv[])
{
    int rc = ESPANK_SUCCESS;
    struct spank_option *opts_to_register = NULL;

    slurm_debug("%s: -> %s argc=%d", plugin_name, __FUNCTION__, argc);
    dump_argv(argc, argv);

    memset(backend_name, '\0', sizeof(backend_name));
    memset(primitive_type, '\0', sizeof(primitive_type));
    /*
     * Get any options registered for this context:
     */
    switch (spank_context()) {
    /* salloc, sbatch */
    case S_CTX_ALLOCATOR:
    /* srun */
    case S_CTX_LOCAL:
    /* slurmstepd */
    case S_CTX_REMOTE:
        opts_to_register = spank_example_options;
        break;

    default:
        break;
    }
    if (opts_to_register) {
        while (opts_to_register->name && (rc == ESPANK_SUCCESS))
            rc = spank_option_register(spank_ctxt, opts_to_register++);
    }

    /*
     * SPANK plugins can query the current list of supported slurm_spank symbols
     * to determine if the current version supports a given plugin hook.
     * This may be useful because the list of plugin symbols may grow in the
     * future.
     */
    slurm_debug("%s Is slurm_spank_task_init() supported ? %d", plugin_name,
        spank_symbol_supported("slurm_spank_task_init"));
    slurm_debug("%s Is slurm_spank_task_exit() supported ? %d", plugin_name,
        spank_symbol_supported("slurm_spank_task_exit"));

    slurm_debug("%s <- %s rc=%d", plugin_name, __FUNCTION__, rc);
    return rc;
}

/*
 * @function slurm_spank_task_init
 *
 * Called for each task just before execve (2). If you are restricting memory
 * with cgroups, memory allocated here will be in the job's cgroup. (remote
 * context only)
 *
 */
int slurm_spank_task_init(spank_t spank_ctxt, int argc, char **argv)
{
    int rc = ESPANK_SUCCESS;
    uint32_t job_id = 0;
    job_info_msg_t *job_info_msg = NULL;

    slurm_debug("%s: -> %s argc=%d remote=%d", plugin_name, __FUNCTION__, argc,
            spank_remote(spank_ctxt));
    dump_argv(argc, argv);
    dump_spank_items(spank_ctxt);

    if (spank_remote(spank_ctxt)) {
        if (strlen(backend_name) > 0) {
            slurm_debug("%s: setenv IBMQRUN_BACKEND=%s",
                plugin_name, backend_name);
            spank_setenv(spank_ctxt, "IBMQRUN_BACKEND", backend_name, 1);
        }

        if (strlen(primitive_type) > 0) {
            slurm_debug("%s: setenv IBMQRUN_PRIMITIVE=%s",
                plugin_name, primitive_type);
            spank_setenv(spank_ctxt, "IBMQRUN_PRIMITIVE", primitive_type, 1);
        }

        if (spank_get_item(spank_ctxt, S_JOB_ID, &job_id) == ESPANK_SUCCESS) { 
            if (slurm_load_job(&job_info_msg, job_id, SHOW_DETAIL) == SLURM_SUCCESS) {
                /* slurm's time limit is represented in minutes */
                uint32_t time_limit_mins = job_info_msg->job_array[0].time_limit;
                /*
                 * minutes to seconds, uint32_t to char*
                 */
                char limit_as_str[11]; /* max uint32_t value is (2147483647) = 10 chars */
                memset(limit_as_str, '\0', sizeof(limit_as_str));
                snprintf(limit_as_str, sizeof(limit_as_str), "%u", time_limit_mins * 60);
                spank_setenv(spank_ctxt, "IBMQRUN_TIMEOUT_SECONDS", limit_as_str, 1);
            }
  	}
    }

    slurm_debug("%s: <- %s rc=%d", plugin_name, __FUNCTION__, rc);
    return rc;
}

/*
 * @function slurm_spank_task_exit
 *
 * Called for each task as its exit status is collected by Slurm. (remote
 * context only)
 */
int slurm_spank_task_exit(spank_t spank_ctxt, int argc, char **argv) 
{
    int rc = ESPANK_SUCCESS;
    int status = 0;

    slurm_debug("%s: -> %s argc=%d", plugin_name, __FUNCTION__, argc);
    dump_argv(argc, argv);

    if (spank_get_item(spank_ctxt, S_TASK_EXIT_STATUS, &status) ==
        ESPANK_SUCCESS) {
        slurm_debug("%s: S_TASK_EXIT_STATUS [%d]", plugin_name, status);
    }

    if (spank_remote(spank_ctxt)) {
        spank_unsetenv(spank_ctxt, "IBMQRUN_BACKEND");
        spank_unsetenv(spank_ctxt, "IBMQRUN_PRIMITIVE");
    }

    slurm_debug("%s: <- %s rc=%d", plugin_name, __FUNCTION__, rc);
    return rc;
}
