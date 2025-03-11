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

#include "direct_access_capi.h"

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

/* QRUN job identifier */
#define MAXLEN_JOB_ID 1024
static char qrun_job_id[MAXLEN_JOB_ID+1];

/* Maximum length of URL (comes from De facto standard) */
#define MAXLEN_URL_DEFAULT  2083
/* Maximum length of Service CRN (See IBM Cloud API handbook) */
#define MAXLEN_SERVICE_CRN_DEFAULT  512
/* Maximum length of API Key (See IBM Cloud documentation) */
#define MAXLEN_IAM_APIKEY   128

/* QRUN executable name */
static const char* QRUN_COMMAND = "qrun";

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

/*
 * @function is_qrun_task
 *
 * Returns true if this is qrun task, otherwise false.
 *
 */
static bool is_qrun_task(spank_t spank_ctxt) {
    int argc = 0;
    char **argv = NULL;

    if (spank_get_item(spank_ctxt, S_JOB_ARGV, &argc, &argv) ==
        ESPANK_SUCCESS) {
        if ((argc > 0) && (strncmp(QRUN_COMMAND, argv[0], strlen(QRUN_COMMAND)) == 0)) {
            return true;
        }
    }
    return false;
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
    int pid = (int)getpid();

    slurm_debug("%s(%d): -> %s argc=%d", plugin_name, pid, __FUNCTION__, argc);

    memset(backend_name, '\0', sizeof(backend_name));
    memset(primitive_type, '\0', sizeof(primitive_type));
    memset(qrun_job_id, '\0', sizeof(qrun_job_id));
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

#ifndef ALLOC_RESOURCE_BY_QRUN
        /*
         * Generate QRUN job identifier
         */
        const char* uuid = daapi_uuid_v4_new();
        if (!uuid) {
            slurm_error("%s(%d): failed to generate UUIDv4", plugin_name, pid);
            return SLURM_ERROR;
        }
        strncpy_s(qrun_job_id, uuid, sizeof(qrun_job_id));
        daapi_free_string((char*)uuid);
        slurm_debug("%s(%d): job_id = %s", plugin_name, pid, qrun_job_id);
#endif /* !ALLOC_RESOURCE_BY_QRUN */
        break;

    default:
        break;
    }
    if (opts_to_register) {
        while (opts_to_register->name && (rc == ESPANK_SUCCESS)) {
            rc = spank_option_register(spank_ctxt, opts_to_register++);
        }
    }

    slurm_debug("%s(%d): <- %s rc=%d", plugin_name, pid, __FUNCTION__, rc);
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
    int pid = (int)getpid();

    slurm_debug("%s(%d): -> %s argc=%d remote=%d", plugin_name, pid, __FUNCTION__, argc,
            spank_remote(spank_ctxt));

    if (spank_remote(spank_ctxt)) {
        if (strlen(backend_name) > 0) {
            slurm_debug("%s(%d): setenv IBMQRUN_BACKEND=%s",
                plugin_name, (int)getpid(), backend_name);
            spank_setenv(spank_ctxt, "IBMQRUN_BACKEND", backend_name, 1);
        }

        if (strlen(primitive_type) > 0) {
            slurm_debug("%s(%d): setenv IBMQRUN_PRIMITIVE=%s",
                plugin_name, (int)getpid(), primitive_type);
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

#ifndef ALLOC_RESOURCE_BY_QRUN
        slurm_debug("%s(%d): setenv IBMQRUN_JOB_ID=%s",
                    plugin_name, pid, qrun_job_id);
        spank_setenv(spank_ctxt, "IBMQRUN_JOB_ID", qrun_job_id, 1);
#endif /* !ALLOC_RESOURCE_BY_QRUN */
    }

    slurm_debug("%s(%d): <- %s rc=%d", plugin_name, pid, __FUNCTION__, rc);
    return rc;
}

#ifndef FREE_RESOURCE_BY_QRUN
static int delete_qrun_job(spank_t spank_ctxt, char* job_id) {
    int rc = ESPANK_SUCCESS;
    int pid = (int)getpid();
    char daapi_endpoint[MAXLEN_URL_DEFAULT + 1];
    char iam_endpoint[MAXLEN_URL_DEFAULT + 1];
    char service_crn[MAXLEN_SERVICE_CRN_DEFAULT + 1];
    char iam_apikey[MAXLEN_IAM_APIKEY + 1];

    memset(daapi_endpoint, '\0', sizeof(daapi_endpoint));
    memset(iam_endpoint, '\0', sizeof(iam_endpoint));
    memset(service_crn, '\0', sizeof(service_crn));
    memset(iam_apikey, '\0', sizeof(iam_apikey));

    if (spank_getenv(spank_ctxt,
                     "IBMQRUN_DAAPI_ENDPOINT",
                     daapi_endpoint,
                     MAXLEN_URL_DEFAULT) != ESPANK_SUCCESS) {
        return rc;
    }
    if (spank_getenv(spank_ctxt,
                     "IBMQRUN_IAM_ENDPOINT",
                     iam_endpoint,
                     MAXLEN_URL_DEFAULT) != ESPANK_SUCCESS) {
        return rc;
    }
    if (spank_getenv(spank_ctxt,
                     "IBMQRUN_SERVICE_CRN",
                     service_crn,
                     MAXLEN_SERVICE_CRN_DEFAULT) != ESPANK_SUCCESS) {
        return rc;
    }
    if (spank_getenv(spank_ctxt,
                     "IBMQRUN_IAM_APIKEY",
                     iam_apikey,
                     MAXLEN_IAM_APIKEY) != ESPANK_SUCCESS) {
        return rc;
    }
    struct ClientBuilder *builder = daapi_bldr_new(daapi_endpoint);
    if (!builder) {
        slurm_error("%s(%d): failed to create a Direct Access Client builder",
                    plugin_name, pid);
        return SLURM_ERROR;
    }
    daapi_bldr_enable_iam_auth(builder, iam_apikey, service_crn, iam_endpoint);
    daapi_bldr_set_timeout(builder, 60.0);
    daapi_bldr_set_exponential_backoff_retry(builder, 5, 2, 1, 10);
    struct Client *client = daapi_cli_new(builder);
    if (!client) {
        slurm_error("%s(%d): failed to create a Direct Access Client", plugin_name, pid);
        daapi_free_builder(builder);
        return SLURM_ERROR;
    }

    JobStatus job_status;
    if (daapi_cli_get_job_status(client, job_id, &job_status) == DAAPI_SUCCESS) {
        if (job_status == RUNNING) {
            /*
             * If qrun job is still running, cancel this job and then delete it.
             */
            slurm_info("%s(%d): cancel & delete qrun job(%s)", plugin_name, pid, job_id);
            daapi_cli_cancel_job(client, job_id, true);
        } 
        else {
            slurm_info("%s(%d): delete qrun job(%s)", plugin_name, pid, job_id);
            daapi_cli_delete_job(client, job_id);
        }
    }
    daapi_free_client(client);
    daapi_free_builder(builder);

    return rc;
}
#endif /* !FREE_RESOURCE_BY_QRUN */

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
    int pid = (int)getpid();

    slurm_debug("%s(%d): -> %s argc=%d", plugin_name, pid, __FUNCTION__, argc);

    if (spank_get_item(spank_ctxt, S_TASK_EXIT_STATUS, &status) ==
        ESPANK_SUCCESS) {
        slurm_debug("%s(%d): S_TASK_EXIT_STATUS [%d]", plugin_name, (int)getpid(), status);
    }

    if (spank_remote(spank_ctxt)) {
        spank_unsetenv(spank_ctxt, "IBMQRUN_BACKEND");
        spank_unsetenv(spank_ctxt, "IBMQRUN_PRIMITIVE");
#ifndef FREE_RESOURCE_BY_QRUN
        /* try to delete a job if this task is 'QRUN'. */
        if (is_qrun_task(spank_ctxt) &&
            delete_qrun_job(spank_ctxt, qrun_job_id) != ESPANK_SUCCESS) {
            slurm_error("%s: failed to delete qrun job(%s).", plugin_name, qrun_job_id);
            rc = SLURM_ERROR;
        }
#endif /* !FREE_RESOURCE_BY_QRUN */ 
    }

    slurm_debug("%s(%d): <- %s rc=%d", plugin_name, pid, __FUNCTION__, rc);
    return rc;
}
